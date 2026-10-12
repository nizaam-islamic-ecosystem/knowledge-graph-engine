//! Physical document-storage abstraction and the initial file-backed provider.
//!
//! The abstraction remains provider-neutral while the initial Phase 7 provider
//! stores opaque documents on the local filesystem. Higher layers must use the
//! [`DocumentStore`] contract rather than depending on filesystem details.

use std::fmt;

use dirs::home_dir;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Opaque identifier for a persisted storage document.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentId(String);

impl DocumentId {
    /// Creates a safe relative document identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, DocumentStoreError> {
        let value = value.into();
        if value.is_empty() {
            return Err(DocumentStoreError::InvalidDocumentId);
        }

        let path = Path::new(&value);
        if path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(DocumentStoreError::InvalidDocumentId);
        }

        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the identifier and returns its underlying string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for DocumentId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Opaque document contents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document(Vec<u8>);

impl Document {
    /// Creates a document from owned bytes.
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    /// Creates a document from a byte slice.
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    /// Returns the document contents.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the document contents as owned bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Returns whether the document contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the document size in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Errors produced by the physical document-storage boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentStoreError {
    /// The supplied document identifier is invalid.
    InvalidDocumentId,
    /// A document expected to exist was not found.
    NotFound(DocumentId),
    /// A document already exists.
    AlreadyExists(DocumentId),
    /// A document mutation encountered a conflicting state.
    Conflict(DocumentId),
    /// The underlying storage provider is unavailable.
    Unavailable(String),
    /// The operating system home directory could not be resolved.
    HomeDirectoryUnavailable,
    /// The provider rejected the requested operation.
    OperationFailed(String),
}

impl fmt::Display for DocumentStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocumentId => formatter.write_str("document identifier is invalid"),
            Self::NotFound(document_id) => {
                write!(formatter, "document '{document_id}' was not found")
            }
            Self::AlreadyExists(document_id) => {
                write!(formatter, "document '{document_id}' already exists")
            }
            Self::Conflict(document_id) => {
                write!(
                    formatter,
                    "document '{document_id}' has a conflicting state"
                )
            }
            Self::Unavailable(message) => {
                write!(formatter, "document store is unavailable: {message}")
            }
            Self::HomeDirectoryUnavailable => {
                formatter.write_str("operating system home directory is unavailable")
            }
            Self::OperationFailed(message) => {
                write!(formatter, "document store operation failed: {message}")
            }
        }
    }
}

impl std::error::Error for DocumentStoreError {}

/// Lowest-level physical document-store contract.
pub trait DocumentStore {
    /// Reads a document by physical document identifier.
    fn get(&self, id: &DocumentId) -> Result<Option<Document>, DocumentStoreError>;

    /// Creates a new document.
    fn create(&self, id: DocumentId, document: Document) -> Result<(), DocumentStoreError>;

    /// Replaces an existing document atomically from the provider's point of
    /// view.
    fn replace(&self, id: &DocumentId, document: Document) -> Result<(), DocumentStoreError>;

    /// Returns whether a document exists.
    fn contains(&self, id: &DocumentId) -> Result<bool, DocumentStoreError>;

    /// Deletes a document.
    fn delete(&self, id: &DocumentId) -> Result<(), DocumentStoreError>;
}

/// Initial Phase 7 file-backed document provider.
#[derive(Debug)]
pub struct FileDocumentStore {
    root: PathBuf,
}

impl FileDocumentStore {
    /// Returns the default Nizaam storage directory in the current user home.
    ///
    /// The `dirs` crate resolves the platform-specific home directory, so the
    /// resulting location is equivalent to `~/.nizaam` on Linux/macOS and the
    /// corresponding user-home `.nizaam` directory on Windows.
    pub fn default_root() -> Result<PathBuf, DocumentStoreError> {
        home_dir()
            .map(|home| home.join(".nizaam"))
            .ok_or(DocumentStoreError::HomeDirectoryUnavailable)
    }

    /// Opens or creates the default file-backed Nizaam document store.
    ///
    /// This creates the `.nizaam` directory in the current operating-system
    /// user home when it does not already exist.
    pub fn open_default() -> Result<Self, DocumentStoreError> {
        Self::new(Self::default_root()?)
    }

    /// Opens or creates a file-backed document store at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, DocumentStoreError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|error| map_io_error(&root, error))?;
        Ok(Self { root })
    }

    /// Returns the provider root for diagnostics and provider-local setup.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Acquires a provider-local exclusive advisory lock.
    ///
    /// The lock file is persistent by design. The operating system releases the
    /// advisory lock if this process exits unexpectedly, so an abandoned lock
    /// file cannot permanently block later writers.
    pub(crate) fn acquire_exclusive_lock(&self) -> Result<FileStoreLock, DocumentStoreError> {
        let lock_path = self.root.join(".lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| map_io_error(&lock_path, error))?;

        file.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => DocumentStoreError::Conflict(
                DocumentId::new(".lock").expect("static lock document id is valid"),
            ),
            std::fs::TryLockError::Error(error) => map_io_error(&lock_path, error),
        })?;

        Ok(FileStoreLock { _file: file })
    }

    fn path_for(&self, id: &DocumentId) -> PathBuf {
        self.root.join(id.as_str())
    }

    fn write_atomically(
        &self,
        id: &DocumentId,
        path: &Path,
        bytes: &[u8],
        create_only: bool,
    ) -> Result<(), DocumentStoreError> {
        // Repository/index callers hold the provider lock across their full
        // read-modify-write transaction. This primitive remains lock-free to
        // avoid recursive locking; create publication itself is race-safe.
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            // Remember every missing directory so each parent directory is
            // synced after its child entry is created.
            let mut missing = Vec::new();
            let mut ancestor = parent;
            while !ancestor.exists() {
                missing.push(ancestor.to_path_buf());
                ancestor = ancestor.parent().unwrap_or_else(|| Path::new("."));
            }
            fs::create_dir_all(parent).map_err(|error| map_io_error(parent, error))?;
            for directory in missing.iter().rev() {
                sync_containing_directory(directory)?;
            }
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document");
        let temporary_name = format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| DocumentStoreError::OperationFailed(error.to_string()))?
                .as_nanos()
        );
        let temporary_path = path.with_file_name(temporary_name);

        let write_result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary_path)
                .map_err(|error| map_io_error(&temporary_path, error))?;
            file.write_all(bytes)
                .map_err(|error| map_io_error(&temporary_path, error))?;
            file.sync_all()
                .map_err(|error| map_io_error(&temporary_path, error))?;
            drop(file);

            if create_only {
                // A hard link publishes the fully written temporary file without
                // replacing an existing destination, even under concurrent create.
                fs::hard_link(&temporary_path, path).map_err(|error| {
                    if error.kind() == io::ErrorKind::AlreadyExists {
                        DocumentStoreError::AlreadyExists(id.clone())
                    } else {
                        map_io_error(path, error)
                    }
                })?;
                // Sync the directory while the new directory entry is durable,
                // then remove the temporary hard link and sync that removal.
                sync_containing_directory(path)?;
                fs::remove_file(&temporary_path)
                    .map_err(|error| map_io_error(&temporary_path, error))?;
                sync_containing_directory(&temporary_path)?;
            } else {
                fs::rename(&temporary_path, path).map_err(|error| map_io_error(path, error))?;
                sync_containing_directory(path)?;
            }
            Ok(())
        })();

        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }

        write_result
    }
}

impl DocumentStore for FileDocumentStore {
    fn get(&self, id: &DocumentId) -> Result<Option<Document>, DocumentStoreError> {
        let path = self.path_for(id);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(Document::new(bytes))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(map_io_error(&path, error)),
        }
    }

    fn create(&self, id: DocumentId, document: Document) -> Result<(), DocumentStoreError> {
        let path = self.path_for(&id);
        self.write_atomically(&id, &path, document.as_bytes(), true)
    }

    fn replace(&self, id: &DocumentId, document: Document) -> Result<(), DocumentStoreError> {
        let path = self.path_for(id);
        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => {
                self.write_atomically(id, &path, document.as_bytes(), false)
            }
            Ok(_) => Err(DocumentStoreError::OperationFailed(format!(
                "document path '{}' is not a regular file",
                path.display()
            ))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(DocumentStoreError::NotFound(id.clone()))
            }
            Err(error) => Err(map_io_error(&path, error)),
        }
    }

    fn contains(&self, id: &DocumentId) -> Result<bool, DocumentStoreError> {
        let path = self.path_for(id);
        match fs::metadata(&path) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(map_io_error(&path, error)),
        }
    }

    fn delete(&self, id: &DocumentId) -> Result<(), DocumentStoreError> {
        let path = self.path_for(id);
        match fs::remove_file(&path) {
            Ok(()) => sync_containing_directory(&path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(DocumentStoreError::NotFound(id.clone()))
            }
            Err(error) => Err(map_io_error(&path, error)),
        }
    }
}

/// Guard for a provider-local OS advisory lock. Dropping the file handle
/// releases the lock; the persistent `.lock` path is intentionally retained.
#[derive(Debug)]
pub(crate) struct FileStoreLock {
    // Holding the handle keeps the OS advisory lock alive. It is intentionally
    // not read; dropping the handle releases the lock but leaves the lock file.
    _file: File,
}

impl Drop for FileStoreLock {
    fn drop(&mut self) {
        // Intentionally do not unlink `.lock`: other processes may already have
        // opened it. The OS releases the advisory lock when `_file` is dropped.
    }
}

#[cfg(unix)]
fn sync_containing_directory(path: &Path) -> Result<(), DocumentStoreError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| map_io_error(parent, error))
}

#[cfg(not(unix))]
fn sync_containing_directory(_path: &Path) -> Result<(), DocumentStoreError> {
    Ok(())
}

fn map_io_error(path: &Path, error: io::Error) -> DocumentStoreError {
    match error.kind() {
        io::ErrorKind::NotFound => DocumentStoreError::NotFound(
            DocumentId::new(path.to_string_lossy().to_string())
                .unwrap_or_else(|_| DocumentId("unknown".to_owned())),
        ),
        io::ErrorKind::PermissionDenied
        | io::ErrorKind::ReadOnlyFilesystem
        | io::ErrorKind::StorageFull
        | io::ErrorKind::WriteZero => DocumentStoreError::Unavailable(error.to_string()),
        _ => DocumentStoreError::OperationFailed(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-document-store-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos()
        ))
    }

    #[test]
    fn default_root_uses_nizaam_directory_under_home() {
        let home = home_dir().expect("test environment should expose a home directory");
        assert_eq!(
            FileDocumentStore::default_root().expect("home should resolve"),
            home.join(".nizaam")
        );
    }

    #[test]
    fn advisory_lock_is_released_without_deleting_lock_file() {
        let root = temp_root();
        let first_store = FileDocumentStore::new(&root).expect("store should open");
        let second_store = FileDocumentStore::new(&root).expect("second store should open");
        let lock = first_store
            .acquire_exclusive_lock()
            .expect("first lock should acquire");
        assert!(matches!(
            second_store.acquire_exclusive_lock(),
            Err(DocumentStoreError::Conflict(_))
        ));
        drop(lock);
        let next_lock = second_store
            .acquire_exclusive_lock()
            .expect("OS should release lock");
        assert!(
            root.join(".lock").exists(),
            "lock file must remain persistent"
        );
        drop(next_lock);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_create_never_overwrites_the_first_document() {
        let root = temp_root();
        let first_store = FileDocumentStore::new(&root).expect("store should open");
        let second_store = FileDocumentStore::new(&root).expect("second store should open");
        let id = DocumentId::new("document.json").expect("valid id");
        let first_id = id.clone();
        let first = std::thread::spawn(move || {
            first_store.create(first_id, Document::from_slice(b"first"))
        });
        let second_id = id.clone();
        let second = std::thread::spawn(move || {
            second_store.create(second_id, Document::from_slice(b"second"))
        });
        let outcomes = [first.join().unwrap(), second.join().unwrap()];
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|result| matches!(result, Err(DocumentStoreError::AlreadyExists(_))))
                .count(),
            1
        );
        let check = FileDocumentStore::new(&root).expect("store should reopen");
        let bytes = check.get(&id).unwrap().unwrap().into_bytes();
        assert!(bytes == b"first" || bytes == b"second");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn document_id_rejects_path_escape() {
        assert!(DocumentId::new("../escape").is_err());
        assert!(DocumentId::new("/absolute").is_err());
    }

    #[test]
    fn file_provider_round_trips_documents() {
        let root = temp_root();
        let store = FileDocumentStore::new(&root).expect("store should open");
        let id = DocumentId::new("documents/001.json").expect("valid id");

        store
            .create(id.clone(), Document::from_slice(b"hello"))
            .expect("document should be created");

        assert!(store.contains(&id).expect("contains should succeed"));
        assert_eq!(
            store.get(&id).expect("read should succeed"),
            Some(Document::from_slice(b"hello"))
        );

        store
            .replace(&id, Document::from_slice(b"updated"))
            .expect("replace should succeed");
        assert_eq!(
            store.get(&id).expect("read should succeed"),
            Some(Document::from_slice(b"updated"))
        );

        store.delete(&id).expect("delete should succeed");
        assert!(!store.contains(&id).expect("contains should succeed"));

        let _ = fs::remove_dir_all(root);
    }
}
