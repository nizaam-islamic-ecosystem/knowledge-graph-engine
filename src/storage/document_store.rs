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
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Environment variable used to select the physical storage lifecycle policy.
pub const STORAGE_MODE_ENV: &str = "NIZAAM_STORAGE_MODE";

/// Production mode retains diagnostic atomic-write artifacts for this long
/// before removing them on a later repository/journal open or commit. Current-
/// operation staging files are removed immediately after a production write.
pub const PRODUCTION_ARTIFACT_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

/// Avoid a full recursive artifact sweep on every canonical commit.
const PRODUCTION_CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Controls whether filesystem artifacts are retained for inspection or cleaned
/// up according to the production retention policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageMode {
    /// Retain atomic-write artifacts so the physical persistence process can be
    /// inspected under `.nizaam`. Canonical state is still published atomically.
    Development,
    /// Remove current-operation staging files and age out old generated atomic
    /// write artifacts without touching canonical or recovery data.
    Production,
}

impl StorageMode {
    /// Resolves the mode from `NIZAAM_STORAGE_MODE`.
    ///
    /// Missing configuration defaults to development mode so local persistence
    /// remains inspectable. Accepted values are `development` and `production`
    /// (case-insensitive). Invalid values are rejected instead of silently
    /// changing the cleanup policy.
    pub fn from_environment() -> Result<Self, DocumentStoreError> {
        match std::env::var(STORAGE_MODE_ENV) {
            Ok(value) => Self::parse(&value),
            Err(std::env::VarError::NotPresent) => Ok(Self::Development),
            Err(std::env::VarError::NotUnicode(value)) => Err(
                DocumentStoreError::InvalidStorageMode(value.to_string_lossy().into_owned()),
            ),
        }
    }

    fn parse(value: &str) -> Result<Self, DocumentStoreError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            _ => Err(DocumentStoreError::InvalidStorageMode(value.to_owned())),
        }
    }

    const fn retains_artifacts(self) -> bool {
        matches!(self, Self::Development)
    }
}

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
    /// The configured storage mode is invalid.
    InvalidStorageMode(String),
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
            Self::InvalidStorageMode(value) => write!(
                formatter,
                "invalid {} value '{}'; expected 'development' or 'production'",
                STORAGE_MODE_ENV, value
            ),
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
    mode: StorageMode,
    last_cleanup_at: Mutex<Option<SystemTime>>,
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

    /// Opens or creates the default store with an explicit storage mode.
    pub fn open_default_with_mode(mode: StorageMode) -> Result<Self, DocumentStoreError> {
        Self::new_with_mode(Self::default_root()?, mode)
    }

    /// Opens or creates a file-backed document store at `root`, using
    /// `NIZAAM_STORAGE_MODE` or development mode when that variable is absent.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, DocumentStoreError> {
        Self::new_with_mode(root, StorageMode::from_environment()?)
    }

    /// Opens or creates a file-backed document store at `root` with an explicit
    /// lifecycle policy. This is useful for applications that prefer typed
    /// configuration over environment-based configuration.
    pub fn new_with_mode(
        root: impl Into<PathBuf>,
        mode: StorageMode,
    ) -> Result<Self, DocumentStoreError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|error| map_io_error(&root, error))?;
        Ok(Self {
            root,
            mode,
            last_cleanup_at: Mutex::new(None),
        })
    }

    /// Returns the provider root for diagnostics and provider-local setup.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the storage lifecycle policy used by this provider.
    pub fn mode(&self) -> StorageMode {
        self.mode
    }

    /// Removes generated atomic-write artifacts older than the production
    /// retention period under KG-owned `kg/` and `test-runs/` directories.
    /// Canonical KG documents, `.lock`, recovery journals, and unrecognized
    /// files outside those owned directories are never selected for deletion.
    ///
    /// Development mode intentionally performs no cleanup. Repository and
    /// journal lifecycle methods call this while holding the storage lock, as
    /// best-effort maintenance. It is crate-visible so callers cannot sweep
    /// artifacts concurrently with an active repository transaction.
    pub(crate) fn cleanup_stale_artifacts(&self) -> Result<usize, DocumentStoreError> {
        if self.mode != StorageMode::Production {
            return Ok(0);
        }

        let now = SystemTime::now();
        {
            let mut last_cleanup = self
                .last_cleanup_at
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(previous) = *last_cleanup
                && now.duration_since(previous).unwrap_or_default() < PRODUCTION_CLEANUP_INTERVAL
            {
                return Ok(0);
            }
            *last_cleanup = Some(now);
        }

        let mut removed = 0;
        if let Err(error) = self.cleanup_managed_artifacts_at(
            &self.root,
            now,
            PRODUCTION_ARTIFACT_RETENTION,
            &mut removed,
        ) {
            // A failed sweep should be retried on the next lifecycle call
            // rather than waiting for the normal interval.
            let mut last_cleanup = self
                .last_cleanup_at
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *last_cleanup = None;
            return Err(error);
        }
        Ok(removed)
    }

    /// Restricts cleanup to KG's canonical storage directory and the dedicated
    /// test-run namespace. This avoids traversing unrelated top-level Nizaam
    /// directories while preserving a realistic `.nizaam` layout for tests.
    fn cleanup_managed_artifacts_at(
        &self,
        root: &Path,
        now: SystemTime,
        retention: Duration,
        removed: &mut usize,
    ) -> Result<(), DocumentStoreError> {
        let kg_directory = root.join("kg");
        if kg_directory.is_dir() {
            self.cleanup_stale_artifacts_in(&kg_directory, now, retention, removed)?;
        }

        let test_runs_directory = root.join("test-runs");
        if test_runs_directory.is_dir() {
            self.cleanup_test_runs_in(&test_runs_directory, now, retention, removed)?;
        }

        Ok(())
    }

    /// Test-run directories created by the KG Phase 7 verification suite are
    /// retained in development and aged out in production. Other directories
    /// within `test-runs/` are traversed only for generated atomic artifacts.
    fn cleanup_test_runs_in(
        &self,
        directory: &Path,
        now: SystemTime,
        retention: Duration,
        removed: &mut usize,
    ) -> Result<(), DocumentStoreError> {
        let entries = fs::read_dir(directory).map_err(|error| map_io_error(directory, error))?;

        for entry in entries {
            let entry = entry.map_err(|error| map_io_error(directory, error))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| map_io_error(&path, error))?;

            if file_type.is_dir() {
                let is_kg_phase7_run = entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with("kg-phase7-"));
                let modified = entry
                    .metadata()
                    .map_err(|error| map_io_error(&path, error))?
                    .modified()
                    .map_err(|error| map_io_error(&path, error))?;
                let age = now.duration_since(modified).unwrap_or_default();

                if is_kg_phase7_run && age >= retention {
                    fs::remove_dir_all(&path).map_err(|error| map_io_error(&path, error))?;
                    sync_containing_directory(&path)?;
                    *removed += 1;
                } else {
                    self.cleanup_stale_artifacts_in(&path, now, retention, removed)?;
                }
                continue;
            }

            if !file_type.is_file() || !is_generated_atomic_artifact(&entry.file_name()) {
                continue;
            }

            let metadata = entry
                .metadata()
                .map_err(|error| map_io_error(&path, error))?;
            let Ok(modified) = metadata.modified() else {
                continue;
            };
            if now.duration_since(modified).unwrap_or_default() < retention {
                continue;
            }

            match fs::remove_file(&path) {
                Ok(()) => {
                    sync_containing_directory(&path)?;
                    *removed += 1;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(map_io_error(&path, error)),
            }
        }

        Ok(())
    }

    fn cleanup_stale_artifacts_in(
        &self,
        directory: &Path,
        now: SystemTime,
        retention: Duration,
        removed: &mut usize,
    ) -> Result<(), DocumentStoreError> {
        let entries = fs::read_dir(directory).map_err(|error| map_io_error(directory, error))?;

        for entry in entries {
            let entry = entry.map_err(|error| map_io_error(directory, error))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| map_io_error(&path, error))?;

            if file_type.is_dir() {
                self.cleanup_stale_artifacts_in(&path, now, retention, removed)?;
                continue;
            }
            if !file_type.is_file() || !is_generated_atomic_artifact(&entry.file_name()) {
                continue;
            }

            let metadata = entry
                .metadata()
                .map_err(|error| map_io_error(&path, error))?;
            let Ok(modified) = metadata.modified() else {
                continue;
            };
            let age = now.duration_since(modified).unwrap_or_default();
            if age < retention {
                continue;
            }

            match fs::remove_file(&path) {
                Ok(()) => {
                    sync_containing_directory(&path)?;
                    *removed += 1;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(map_io_error(&path, error)),
            }
        }

        Ok(())
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
        let artifact_stem = format!(
            ".{file_name}.{}.{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| DocumentStoreError::OperationFailed(error.to_string()))?
                .as_nanos()
        );
        let staging_path = path.with_file_name(format!("{artifact_stem}.stage"));
        let retained_path = path.with_file_name(format!("{artifact_stem}.tmp"));

        let mut staging_created = false;
        let mut canonical_published = false;
        let write_result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging_path)
                .map_err(|error| map_io_error(&staging_path, error))?;
            staging_created = true;
            file.write_all(bytes)
                .map_err(|error| map_io_error(&staging_path, error))?;
            file.sync_all()
                .map_err(|error| map_io_error(&staging_path, error))?;
            drop(file);

            if self.mode.retains_artifacts() {
                // A retained hard link records the exact fully-written bytes
                // while allowing the stage file to be atomically published.
                // If hard links are unavailable, create the diagnostic copy
                // with create_new so a timestamp collision cannot overwrite
                // another operation's artifact. Diagnostic retention is never
                // allowed to break a canonical write.
                if fs::hard_link(&staging_path, &retained_path).is_err()
                    && let Ok(mut retained) = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&retained_path)
                {
                    let _ = retained.write_all(bytes);
                    let _ = retained.sync_all();
                }
            }

            if create_only {
                // A hard link publishes the fully written staging file without
                // replacing an existing destination, even under concurrent create.
                fs::hard_link(&staging_path, path).map_err(|error| {
                    if error.kind() == io::ErrorKind::AlreadyExists {
                        DocumentStoreError::AlreadyExists(id.clone())
                    } else {
                        map_io_error(path, error)
                    }
                })?;
                canonical_published = true;
                sync_containing_directory(path)?;
                if self.mode == StorageMode::Production {
                    // Canonical publication was already synced successfully. A
                    // failure to remove its extra staging link is cleanup failure,
                    // not a failed canonical commit; a later sweep may retry it.
                    if fs::remove_file(&staging_path).is_ok() {
                        let _ = sync_containing_directory(&staging_path);
                    }
                }
                // In development the complete stage is retained alongside the
                // canonical hard link so the physical publication can be inspected.
            } else {
                // Rename moves the staged inode into canonical position. In
                // development its retained `.tmp` hard link still preserves
                // the exact bytes of this write for later inspection.
                fs::rename(&staging_path, path).map_err(|error| map_io_error(path, error))?;
                canonical_published = true;
                sync_containing_directory(path)?;
            }
            Ok(())
        })();

        if write_result.is_err()
            && self.mode == StorageMode::Production
            && staging_created
            && !canonical_published
        {
            // Only remove a stage created by this operation, and only when it
            // was not published. This avoids deleting a colliding stage owned
            // by another writer and preserves evidence if publication durability
            // could not be confirmed. Canonical and recovery files are untouched.
            let _ = fs::remove_file(&staging_path);
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

/// Recognizes only the exact generated naming convention for atomic-write
/// artifacts: `.<document>.<pid>.<timestamp>.(tmp|stage)`.
fn is_generated_atomic_artifact(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    let Some(rest) = name.strip_prefix('.') else {
        return false;
    };
    let Some((rest, suffix)) = rest.rsplit_once('.') else {
        return false;
    };
    if suffix != "tmp" && suffix != "stage" {
        return false;
    }
    let Some((rest, timestamp)) = rest.rsplit_once('.') else {
        return false;
    };
    let Some((document_name, process_id)) = rest.rsplit_once('.') else {
        return false;
    };

    !document_name.is_empty()
        && process_id.parse::<u32>().is_ok()
        && timestamp.parse::<u128>().is_ok()
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

    #[test]
    fn development_mode_retains_atomic_artifacts_and_canonical_state() {
        let root = temp_root();
        let store = FileDocumentStore::new_with_mode(&root, StorageMode::Development)
            .expect("development store should open");
        let state_id = DocumentId::new("kg/current-state.json").unwrap();

        store
            .create(state_id.clone(), Document::from_slice(b"state-v1"))
            .expect("initial state should be created");
        assert!(matches!(
            store.create(
                state_id.clone(),
                Document::from_slice(b"must-not-overwrite")
            ),
            Err(DocumentStoreError::AlreadyExists(_))
        ));
        assert_eq!(
            store.get(&state_id).unwrap().unwrap().as_bytes(),
            b"state-v1",
            "a failed duplicate create must never change canonical state"
        );
        store
            .replace(&state_id, Document::from_slice(b"state-v2"))
            .expect("state should be replaced atomically");

        assert_eq!(store.mode(), StorageMode::Development);
        assert_eq!(
            store.get(&state_id).unwrap().unwrap().as_bytes(),
            b"state-v2"
        );

        let state_dir = root.join("kg");
        let retained = fs::read_dir(&state_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| is_generated_atomic_artifact(&entry.file_name()))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        assert!(
            retained
                .iter()
                .any(|path| path.extension().is_some_and(|ext| ext == "tmp")),
            "development mode must retain a diagnostic copy for inspection"
        );
        assert!(
            retained
                .iter()
                .any(|path| path.extension().is_some_and(|ext| ext == "stage")),
            "development mode must retain the original staging link after a create"
        );
        assert!(
            retained.iter().all(|path| path.is_file()),
            "retained artifacts must remain readable files"
        );
        assert_eq!(store.cleanup_stale_artifacts().unwrap(), 0);
        assert!(state_dir.join("current-state.json").is_file());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn production_mode_cleans_staging_files_but_preserves_canonical_and_recovery_files() {
        let root = temp_root();
        let store = FileDocumentStore::new_with_mode(&root, StorageMode::Production)
            .expect("production store should open");
        let state_id = DocumentId::new("kg/current-state.json").unwrap();
        store
            .create(state_id.clone(), Document::from_slice(b"state-v1"))
            .expect("initial state should be created");
        store
            .replace(&state_id, Document::from_slice(b"state-v2"))
            .expect("state should be replaced atomically");
        assert!(matches!(
            store.create(
                state_id.clone(),
                Document::from_slice(b"must-not-overwrite")
            ),
            Err(DocumentStoreError::AlreadyExists(_))
        ));

        let state_path = root.join("kg/current-state.json");
        assert_eq!(fs::read(&state_path).unwrap(), b"state-v2");
        assert_eq!(
            fs::read_dir(root.join("kg"))
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| is_generated_atomic_artifact(&entry.file_name()))
                .count(),
            0,
            "production mode must not keep current-operation staging artifacts"
        );

        // Simulate artifacts retained by a previous development run. Zero
        // retention is used only by this unit test to deterministically test
        // the selective cleanup predicate; production uses the 24-hour policy.
        let old_tmp = root.join("kg/.current-state.987.123456.tmp");
        let old_stage = root.join("kg/.current-state.987.123457.stage");
        fs::write(&old_tmp, b"old candidate").unwrap();
        fs::write(&old_stage, b"old stage").unwrap();
        fs::create_dir_all(root.join("recovery")).unwrap();
        fs::write(root.join("recovery/journal.jsonl"), b"durable journal\n").unwrap();
        fs::write(root.join(".lock"), b"").unwrap();
        fs::write(root.join("unrecognized.tmp"), b"keep me").unwrap();
        let old_test_run = root.join("test-runs/kg-phase7-old-run");
        fs::create_dir_all(old_test_run.join("kg")).unwrap();
        fs::write(
            old_test_run.join("kg/.current-state.987.123458.tmp"),
            b"old test-only state",
        )
        .unwrap();

        let mut removed = 0;
        store
            .cleanup_managed_artifacts_at(&root, SystemTime::now(), Duration::ZERO, &mut removed)
            .expect("selective KG cleanup should succeed");
        assert_eq!(removed, 3);
        assert!(!old_tmp.exists());
        assert!(!old_stage.exists());
        assert!(
            state_path.is_file(),
            "canonical state must never be deleted"
        );
        assert!(
            root.join(".lock").is_file(),
            "the advisory lock path must remain"
        );
        assert!(
            root.join("recovery/journal.jsonl").is_file(),
            "durable recovery journal must never be deleted"
        );
        assert!(root.join("unrecognized.tmp").is_file());
        assert!(
            !old_test_run.exists(),
            "production mode must age out old KG-owned test-run directories"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn storage_mode_parser_accepts_only_explicit_supported_modes() {
        assert_eq!(
            StorageMode::parse("development").unwrap(),
            StorageMode::Development
        );
        assert_eq!(
            StorageMode::parse(" PRODUCTION ").unwrap(),
            StorageMode::Production
        );
        assert!(matches!(
            StorageMode::parse("unknown-mode"),
            Err(DocumentStoreError::InvalidStorageMode(_))
        ));
    }
}
