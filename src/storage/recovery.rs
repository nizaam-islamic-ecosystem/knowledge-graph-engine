//! Durable recovery journal and restart-recovery primitives for the Knowledge Graph.
//!
//! The recovery journal is intentionally separate from:
//! - `ChangeSet`, which represents semantic KG changes;
//! - `ChangeSetTransaction`, which represents an in-process transaction;
//! - the canonical repository state, which remains the source of truth.
//!
//! A journal entry is a physical recovery record. A prepared entry is not
//! canonical until its matching commit marker has been durably written.
//! Therefore an incomplete prepared transaction is ignored during recovery,
//! while malformed or contradictory journal data is reported explicitly.

use core::fmt;
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::document_store::{Document, DocumentStoreError, FileDocumentStore};
use super::repository::RepositoryRecord;
use super::snapshot::Snapshot;
use crate::versioning::{
    Change, ChangeKind, ChangeSet, ChangeSetId, ChangeTarget, KgVersion, KgVersionId,
};

const RECOVERY_DIRECTORY: &str = "recovery";
const JOURNAL_DOCUMENT: &str = "recovery/journal.jsonl";
const JOURNAL_FORMAT_VERSION: u64 = 1;

/// One durable semantic state transition recorded for recovery.
///
/// This is a physical recovery record. It is deliberately not exposed as an
/// audit record and does not replace the semantic `ChangeSet`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryCommit {
    base_version: KgVersionId,
    version: KgVersion,
    change_set: ChangeSet,
    records: Vec<RepositoryRecord>,
}

impl RecoveryCommit {
    /// Creates one recovery record for a pending canonical state transition.
    ///
    /// The resulting version must reference the supplied Change Set and have
    /// the supplied base version as its parent.
    pub fn new(
        base_version: KgVersionId,
        version: KgVersion,
        change_set: ChangeSet,
        records: Vec<RepositoryRecord>,
    ) -> Result<Self, RecoveryError> {
        if change_set.is_empty() {
            return Err(RecoveryError::InvalidCommit(
                "recovery commit requires a non-empty Change Set".to_owned(),
            ));
        }

        if change_set.base_version() != Some(&base_version) {
            return Err(RecoveryError::InvalidCommit(
                "Change Set base version does not match recovery base version".to_owned(),
            ));
        }

        if version.parent() != Some(&base_version) {
            return Err(RecoveryError::InvalidCommit(
                "KG version parent does not match recovery base version".to_owned(),
            ));
        }

        if version.change_set() != Some(change_set.id()) {
            return Err(RecoveryError::InvalidCommit(
                "KG version does not reference the recovery Change Set".to_owned(),
            ));
        }

        for record in &records {
            if !change_set
                .changes()
                .iter()
                .any(|change| change.target() == record.target())
            {
                return Err(RecoveryError::InvalidCommit(
                    "recovery record target is absent from the Change Set".to_owned(),
                ));
            }
        }

        Ok(Self {
            base_version,
            version,
            change_set,
            records,
        })
    }

    /// Returns the KG version from which the commit starts.
    #[must_use]
    pub fn base_version(&self) -> &KgVersionId {
        &self.base_version
    }

    /// Returns the version produced by this commit.
    #[must_use]
    pub fn version(&self) -> &KgVersion {
        &self.version
    }

    /// Returns the semantic Change Set represented by this recovery record.
    #[must_use]
    pub fn change_set(&self) -> &ChangeSet {
        &self.change_set
    }

    /// Returns changed-target records carried by this recovery commit.
    /// Recovery merges these records into the snapshot state rather than
    /// treating them as a complete replacement for all canonical records.
    #[must_use]
    pub fn records(&self) -> &[RepositoryRecord] {
        &self.records
    }
}

/// The result of reconstructing canonical state from a snapshot and committed
/// recovery entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveredState {
    version: KgVersion,
    records: Vec<RepositoryRecord>,
    replayed_change_sets: Vec<ChangeSet>,
}

impl RecoveredState {
    fn from_snapshot(snapshot: &Snapshot) -> Self {
        Self {
            version: snapshot.version().clone(),
            records: snapshot.records().to_vec(),
            replayed_change_sets: Vec::new(),
        }
    }

    /// Returns the reconstructed current KG version.
    #[must_use]
    pub fn version(&self) -> &KgVersion {
        &self.version
    }

    /// Returns the reconstructed canonical records.
    #[must_use]
    pub fn records(&self) -> &[RepositoryRecord] {
        &self.records
    }

    /// Returns Change Sets replayed after the supplied snapshot.
    #[must_use]
    pub fn replayed_change_sets(&self) -> &[ChangeSet] {
        &self.replayed_change_sets
    }
}

/// Recovery failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryError {
    /// A recovery operation could not access its physical journal.
    Storage(String),

    /// A journal entry is malformed or internally contradictory.
    CorruptJournal(String),

    /// A recovery commit violates the version/change-set contract.
    InvalidCommit(String),

    /// A journal commit cannot be applied to the supplied snapshot because
    /// the versions are not contiguous.
    VersionConflict {
        /// Version at the end of the reconstructed state.
        expected: KgVersionId,
        /// Version required by the journal entry.
        actual: KgVersionId,
    },

    /// The supplied snapshot cannot be used as a recovery starting point.
    InvalidSnapshot(String),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(message) => write!(formatter, "recovery storage failure: {message}"),
            Self::CorruptJournal(message) => {
                write!(formatter, "recovery journal is corrupt: {message}")
            }
            Self::InvalidCommit(message) => {
                write!(formatter, "invalid recovery commit: {message}")
            }
            Self::VersionConflict { expected, actual } => {
                write!(
                    formatter,
                    "recovery version conflict: expected {}, actual {}",
                    expected.as_str(),
                    actual.as_str()
                )
            }
            Self::InvalidSnapshot(message) => {
                write!(formatter, "invalid recovery snapshot: {message}")
            }
        }
    }
}

impl std::error::Error for RecoveryError {}

impl From<DocumentStoreError> for RecoveryError {
    fn from(error: DocumentStoreError) -> Self {
        Self::Storage(error.to_string())
    }
}

/// Physical journal boundary used by the recovery workflow.
///
/// The journal stores prepared and committed records separately. Recovery
/// applies only prepared records that have a matching durable commit marker.
pub trait RecoveryJournal {
    /// Durably records a prepared state transition.
    fn prepare(&mut self, commit: &RecoveryCommit) -> Result<(), RecoveryError>;

    /// Durably marks a prepared state transition as committed.
    fn commit(&mut self, change_set_id: &ChangeSetId) -> Result<(), RecoveryError>;

    /// Reads all journal commits in physical order.
    ///
    /// Prepared entries without a commit marker are intentionally omitted.
    fn committed(&self) -> Result<Vec<RecoveryCommit>, RecoveryError>;
}

/// File-backed append-only recovery journal.
///
/// The journal is stored under:
///
/// `~/.nizaam/recovery/journal.jsonl`
///
/// when opened through [`FileRecoveryJournal::open_default`].
#[derive(Debug)]
pub struct FileRecoveryJournal {
    path: PathBuf,
}

impl FileRecoveryJournal {
    /// Opens or creates the default Nizaam recovery journal.
    pub fn open_default() -> Result<Self, RecoveryError> {
        let root = FileDocumentStore::default_root()?;
        Self::open(root)
    }

    /// Opens or creates the recovery journal below the supplied Nizaam root.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, RecoveryError> {
        let root = root.into();
        let store = FileDocumentStore::new(root.clone()).map_err(RecoveryError::from)?;
        let _lock = store
            .acquire_exclusive_lock()
            .map_err(RecoveryError::from)?;
        let directory = root.join(RECOVERY_DIRECTORY);

        fs::create_dir_all(&directory).map_err(|error: io::Error| {
            RecoveryError::Storage(format!(
                "failed to create recovery directory '{}': {error}",
                directory.display()
            ))
        })?;
        sync_directory(&directory)?;

        Ok(Self {
            path: root.join(JOURNAL_DOCUMENT),
        })
    }

    /// Returns the physical journal path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn append(&self, value: Value) -> Result<(), RecoveryError> {
        let root = self.path.parent().and_then(Path::parent).ok_or_else(|| {
            RecoveryError::Storage("recovery journal root could not be resolved".to_owned())
        })?;
        let store = FileDocumentStore::new(root).map_err(RecoveryError::from)?;
        let _lock = store
            .acquire_exclusive_lock()
            .map_err(RecoveryError::from)?;
        let mut line = serde_json::to_vec(&value)
            .map_err(|error| RecoveryError::Storage(error.to_string()))?;
        line.push(b'\n');

        let was_missing = !self.path.exists();
        #[allow(clippy::suspicious_open_options)]
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| {
                RecoveryError::Storage(format!(
                    "failed to open recovery journal '{}': {error}",
                    self.path.display()
                ))
            })?;

        // A missing newline means the last append may have been interrupted.
        // Discard only that unterminated tail before adding the next record.
        let mut existing = Vec::new();
        file.read_to_end(&mut existing).map_err(|error| {
            RecoveryError::Storage(format!(
                "failed to read recovery journal '{}': {error}",
                self.path.display()
            ))
        })?;
        if !existing.is_empty() && existing.last() != Some(&b'\n') {
            let tail_start = existing
                .iter()
                .rposition(|byte| *byte == b'\n')
                .map_or(0, |position| position + 1);
            let tail = &existing[tail_start..];
            if serde_json::from_slice::<Value>(tail).is_ok() {
                // A complete JSON record whose newline was lost is still valid;
                // terminate it before appending rather than discarding it.
                file.seek(SeekFrom::End(0))
                    .and_then(|_| file.write_all(b"\n"))
                    .map_err(|error| {
                        RecoveryError::Storage(format!(
                            "failed to finish recovery journal line '{}': {error}",
                            self.path.display()
                        ))
                    })?;
                file.sync_all().map_err(|error| {
                    RecoveryError::Storage(format!(
                        "failed to durably finish recovery journal line '{}': {error}",
                        self.path.display()
                    ))
                })?;
            } else {
                // An invalid unterminated final line is an interrupted append.
                // Truncate only that tail; all earlier completed lines remain.
                file.set_len(tail_start as u64).map_err(|error| {
                    RecoveryError::Storage(format!(
                        "failed to truncate incomplete recovery journal tail '{}': {error}",
                        self.path.display()
                    ))
                })?;
                file.sync_all().map_err(|error| {
                    RecoveryError::Storage(format!(
                        "failed to durably truncate recovery journal '{}': {error}",
                        self.path.display()
                    ))
                })?;
            }
        }
        file.seek(SeekFrom::End(0)).map_err(|error| {
            RecoveryError::Storage(format!(
                "failed to seek recovery journal '{}': {error}",
                self.path.display()
            ))
        })?;
        file.write_all(&line)
            .and_then(|_| file.sync_all())
            .map_err(|error| {
                RecoveryError::Storage(format!(
                    "failed to durably append recovery journal '{}': {error}",
                    self.path.display()
                ))
            })?;
        if was_missing {
            sync_directory(&self.path)?;
        }
        Ok(())
    }

    fn read_values(&self) -> Result<Vec<Value>, RecoveryError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let bytes = fs::read(&self.path).map_err(|error| {
            RecoveryError::Storage(format!(
                "failed to read recovery journal '{}': {error}",
                self.path.display()
            ))
        })?;
        let mut values = Vec::new();
        let mut start = 0usize;
        let mut line_number = 0usize;

        for (index, byte) in bytes.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            line_number += 1;
            let line = &bytes[start..index];
            start = index + 1;
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let value = serde_json::from_slice::<Value>(line).map_err(|error| {
                RecoveryError::CorruptJournal(format!(
                    "invalid JSON at journal line {line_number}: {error}"
                ))
            })?;
            values.push(value);
        }

        if start < bytes.len() {
            // Accept a complete JSON value without a final newline. If parsing
            // fails, ignore only this unterminated tail: a later append will
            // truncate it. Completed newline-terminated lines above are strict.
            let tail = &bytes[start..];
            if !tail.iter().all(u8::is_ascii_whitespace)
                && let Ok(value) = serde_json::from_slice::<Value>(tail)
            {
                values.push(value);
            }
            // A malformed unterminated final tail is treated as an
            // interrupted append. Newline-terminated lines above are always
            // parsed strictly and malformed completed entries are rejected.
        }

        Ok(values)
    }
}

impl RecoveryJournal for FileRecoveryJournal {
    fn prepare(&mut self, commit: &RecoveryCommit) -> Result<(), RecoveryError> {
        self.append(json!({
            "format_version": JOURNAL_FORMAT_VERSION,
            "type": "prepare",
            "change_set_id": commit.change_set().id().as_str(),
            "commit": encode_commit(commit),
        }))
    }

    fn commit(&mut self, change_set_id: &ChangeSetId) -> Result<(), RecoveryError> {
        self.append(json!({
            "format_version": JOURNAL_FORMAT_VERSION,
            "type": "commit",
            "change_set_id": change_set_id.as_str(),
        }))
    }

    fn committed(&self) -> Result<Vec<RecoveryCommit>, RecoveryError> {
        let values = self.read_values()?;
        let mut prepared = BTreeMap::<String, RecoveryCommit>::new();
        let mut committed = Vec::new();

        for value in values {
            let format_version = value
                .get("format_version")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    RecoveryError::CorruptJournal(
                        "journal entry is missing format_version".to_owned(),
                    )
                })?;

            if format_version != JOURNAL_FORMAT_VERSION {
                return Err(RecoveryError::CorruptJournal(format!(
                    "unsupported journal format version {format_version}"
                )));
            }

            match value.get("type").and_then(Value::as_str) {
                Some("prepare") => {
                    let change_set_id = required_string(&value, "change_set_id")?.to_owned();

                    if prepared.contains_key(&change_set_id) {
                        return Err(RecoveryError::CorruptJournal(format!(
                            "duplicate prepare entry for Change Set '{change_set_id}'"
                        )));
                    }

                    let commit = decode_commit(required_object(&value, "commit")?)?;

                    if commit.change_set().id().as_str() != change_set_id {
                        return Err(RecoveryError::CorruptJournal(format!(
                            "prepare Change Set identity mismatch for '{change_set_id}'"
                        )));
                    }

                    prepared.insert(change_set_id, commit);
                }
                Some("commit") => {
                    let change_set_id = required_string(&value, "change_set_id")?;

                    let commit = prepared.remove(change_set_id).ok_or_else(|| {
                        RecoveryError::CorruptJournal(format!(
                            "commit marker has no matching prepare entry for Change Set '{change_set_id}'"
                        ))
                    })?;

                    committed.push(commit);
                }
                Some(other) => {
                    return Err(RecoveryError::CorruptJournal(format!(
                        "unknown journal entry type '{other}'"
                    )));
                }
                None => {
                    return Err(RecoveryError::CorruptJournal(
                        "journal entry is missing type".to_owned(),
                    ));
                }
            }
        }

        // Any remaining prepared entries are incomplete transactions. They are
        // intentionally ignored rather than promoted into canonical state.
        Ok(committed)
    }
}

/// Reconstructs canonical state from a stable snapshot and the journal entries
/// committed after that snapshot.
///
/// The journal is replayed only when each entry's base version matches the
/// version currently represented by the reconstructed state.
#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), RecoveryError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            RecoveryError::Storage(format!(
                "failed to sync recovery directory '{}': {error}",
                parent.display()
            ))
        })
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), RecoveryError> {
    Ok(())
}

pub fn recover_from_snapshot<J: RecoveryJournal + ?Sized>(
    snapshot: &Snapshot,
    journal: &J,
) -> Result<RecoveredState, RecoveryError> {
    if snapshot.version().id().as_str().is_empty() {
        return Err(RecoveryError::InvalidSnapshot(
            "snapshot does not contain a valid KG version".to_owned(),
        ));
    }

    let mut state = RecoveredState::from_snapshot(snapshot);

    for commit in journal.committed()? {
        // A full append-only journal may contain the history used to construct
        // this snapshot. Skip that prefix by logical sequence; later entries
        // must still chain from the snapshot/current replay version.
        if commit.version().sequence() < state.version.sequence() {
            continue;
        }
        if commit.version().sequence() == state.version.sequence() {
            if commit.version().id() != state.version.id() {
                return Err(RecoveryError::VersionConflict {
                    expected: state.version.id().clone(),
                    actual: commit.base_version().clone(),
                });
            }
            continue;
        }

        if commit.base_version() != state.version.id()
            || commit.version().sequence() != state.version.sequence().saturating_add(1)
        {
            return Err(RecoveryError::VersionConflict {
                expected: state.version.id().clone(),
                actual: commit.base_version().clone(),
            });
        }

        // Journal records represent the changed targets, not a whole-state
        // replacement. Preserve untouched snapshot records and upsert each
        // record supplied by this committed change.
        for updated_record in commit.records() {
            if let Some(existing) = state
                .records
                .iter_mut()
                .find(|existing| existing.target() == updated_record.target())
            {
                *existing = updated_record.clone();
            } else {
                state.records.push(updated_record.clone());
            }
        }
        state.version = commit.version().clone();
        state.replayed_change_sets.push(commit.change_set().clone());
    }

    Ok(state)
}

fn encode_commit(commit: &RecoveryCommit) -> Value {
    json!({
        "base_version": commit.base_version().as_str(),
        "version": encode_version(commit.version()),
        "change_set": encode_change_set(commit.change_set()),
        "records": commit.records().iter().map(encode_record).collect::<Vec<_>>(),
    })
}

fn decode_commit(value: &serde_json::Map<String, Value>) -> Result<RecoveryCommit, RecoveryError> {
    let base_version = KgVersionId::new(required_string_map(value, "base_version")?)
        .map_err(|_| RecoveryError::CorruptJournal("invalid recovery base version".to_owned()))?;

    let version = decode_version(required_object_map(value, "version")?)?;
    let change_set = decode_change_set(required_object_map(value, "change_set")?)?;

    let records = required_array_map(value, "records")?
        .iter()
        .map(decode_record)
        .collect::<Result<Vec<_>, _>>()?;

    RecoveryCommit::new(base_version, version, change_set, records)
}

fn encode_version(version: &KgVersion) -> Value {
    json!({
        "id": version.id().as_str(),
        "sequence": version.sequence(),
        "parent": version.parent().map(|id| id.as_str()),
        "change_set": version.change_set().map(|id| id.as_str()),
    })
}

fn decode_version(value: &serde_json::Map<String, Value>) -> Result<KgVersion, RecoveryError> {
    let id = KgVersionId::new(required_string_map(value, "id")?)
        .map_err(|_| RecoveryError::CorruptJournal("invalid KG version identity".to_owned()))?;

    let sequence = required_u64_map(value, "sequence")?;

    let parent = optional_string_map(value, "parent")?
        .map(|value| {
            KgVersionId::new(value).map_err(|_| {
                RecoveryError::CorruptJournal("invalid parent version identity".to_owned())
            })
        })
        .transpose()?;

    let change_set = optional_string_map(value, "change_set")?
        .map(|value| {
            ChangeSetId::new(value).map_err(|_| {
                RecoveryError::CorruptJournal("invalid Change Set identity".to_owned())
            })
        })
        .transpose()?;

    Ok(KgVersion::from_parts(id, sequence, parent, change_set))
}

fn encode_change_set(change_set: &ChangeSet) -> Value {
    json!({
        "id": change_set.id().as_str(),
        "base_version": change_set.base_version().map(|id| id.as_str()),
        "changes": change_set.changes().iter().map(|change| json!({
            "kind": change.kind().as_str(),
            "target_kind": change.target().kind(),
            "target_id": change.target().id(),
        })).collect::<Vec<_>>(),
    })
}

fn decode_change_set(value: &serde_json::Map<String, Value>) -> Result<ChangeSet, RecoveryError> {
    let id = ChangeSetId::new(required_string_map(value, "id")?)
        .map_err(|_| RecoveryError::CorruptJournal("invalid Change Set identity".to_owned()))?;

    let base_version = optional_string_map(value, "base_version")?
        .map(|value| {
            KgVersionId::new(value).map_err(|_| {
                RecoveryError::CorruptJournal("invalid Change Set base version".to_owned())
            })
        })
        .transpose()?;

    let changes = required_array_map(value, "changes")?
        .iter()
        .map(|change| {
            let object = change.as_object().ok_or_else(|| {
                RecoveryError::CorruptJournal(
                    "Change Set change entry must be an object".to_owned(),
                )
            })?;

            let kind =
                ChangeKind::from_str(required_string_map(object, "kind")?).ok_or_else(|| {
                    RecoveryError::CorruptJournal("unknown Change Set change kind".to_owned())
                })?;

            let target = ChangeTarget::from_kind_and_id(
                required_string_map(object, "target_kind")?,
                required_string_map(object, "target_id")?,
            )
            .map_err(RecoveryError::CorruptJournal)?;

            Ok(Change::new(kind, target))
        })
        .collect::<Result<Vec<_>, RecoveryError>>()?;

    Ok(ChangeSet::from_parts(id, base_version, changes))
}

fn encode_record(record: &RepositoryRecord) -> Value {
    json!({
        "kind": record.target().kind(),
        "id": record.target().id(),
        "bytes": record.document().as_bytes(),
    })
}

fn decode_record(value: &Value) -> Result<RepositoryRecord, RecoveryError> {
    let object = value.as_object().ok_or_else(|| {
        RecoveryError::CorruptJournal("recovery record must be an object".to_owned())
    })?;

    let target = ChangeTarget::from_kind_and_id(
        required_string_map(object, "kind")?,
        required_string_map(object, "id")?,
    )
    .map_err(RecoveryError::CorruptJournal)?;

    let bytes = required_array_map(object, "bytes")?
        .iter()
        .map(|byte| {
            let value = byte.as_u64().ok_or_else(|| {
                RecoveryError::CorruptJournal(
                    "recovery record byte must be an unsigned integer".to_owned(),
                )
            })?;

            u8::try_from(value).map_err(|_| {
                RecoveryError::CorruptJournal("recovery record byte is outside 0..=255".to_owned())
            })
        })
        .collect::<Result<Vec<_>, RecoveryError>>()?;

    Ok(RepositoryRecord::new(target, Document::new(bytes)))
}

fn required_object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, RecoveryError> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| RecoveryError::CorruptJournal(format!("missing object field '{field}'")))
}

fn required_object_map<'a>(
    value: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, RecoveryError> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| RecoveryError::CorruptJournal(format!("missing object field '{field}'")))
}

fn required_array_map<'a>(
    value: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, RecoveryError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| RecoveryError::CorruptJournal(format!("missing array field '{field}'")))
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, RecoveryError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| RecoveryError::CorruptJournal(format!("missing string field '{field}'")))
}

fn required_string_map<'a>(
    value: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a str, RecoveryError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| RecoveryError::CorruptJournal(format!("missing string field '{field}'")))
}

fn optional_string_map<'a>(
    value: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, RecoveryError> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(RecoveryError::CorruptJournal(format!(
            "invalid optional string field '{field}'"
        ))),
    }
}

fn required_u64_map(
    value: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<u64, RecoveryError> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| RecoveryError::CorruptJournal(format!("missing integer field '{field}'")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-recovery-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos()
        ))
    }

    fn target() -> ChangeTarget {
        ChangeTarget::Entity(EntityId::new("entity-1").expect("valid entity identity"))
    }

    fn commit(base: &KgVersion) -> RecoveryCommit {
        let target = target();
        let mut change_set = ChangeSet::new(Some(base.id().clone()));
        change_set.push(Change::new(ChangeKind::Created, target.clone()));

        let next = KgVersion::next(base, change_set.id().clone());

        RecoveryCommit::new(
            base.id().clone(),
            next,
            change_set,
            vec![RepositoryRecord::new(
                target,
                Document::from_slice(b"entity"),
            )],
        )
        .expect("recovery commit should be valid")
    }

    #[test]
    fn recovery_commit_requires_consistent_versions() {
        let base = KgVersion::initial();
        let mut change_set = ChangeSet::new(Some(base.id().clone()));
        let target = target();
        change_set.push(Change::new(ChangeKind::Created, target));

        let unrelated = KgVersion::initial();

        let error = RecoveryCommit::new(base.id().clone(), unrelated, change_set, Vec::new())
            .expect_err("inconsistent recovery commit must fail");

        assert!(matches!(error, RecoveryError::InvalidCommit(_)));
    }

    #[test]
    fn file_journal_replays_only_committed_entries() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).expect("journal should open");

        let base = KgVersion::initial();
        let prepared = commit(&base);

        journal.prepare(&prepared).expect("prepare should succeed");

        assert!(
            journal
                .committed()
                .expect("journal read should succeed")
                .is_empty()
        );

        journal
            .commit(prepared.change_set().id())
            .expect("commit marker should succeed");

        let committed = journal.committed().expect("journal read should succeed");

        assert_eq!(committed, vec![prepared]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn incomplete_prepare_is_ignored() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).expect("journal should open");

        let prepared = commit(&KgVersion::initial());

        journal.prepare(&prepared).expect("prepare should succeed");

        assert!(
            journal
                .committed()
                .expect("journal read should succeed")
                .is_empty()
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_reconstructs_state_after_snapshot_version() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).expect("journal should open");

        let base = KgVersion::initial();
        let snapshot = Snapshot::new(base.clone(), Vec::new()).expect("snapshot should be valid");

        let prepared = commit(&base);

        journal.prepare(&prepared).expect("prepare should succeed");
        journal
            .commit(prepared.change_set().id())
            .expect("commit marker should succeed");

        let recovered =
            recover_from_snapshot(&snapshot, &journal).expect("recovery should succeed");

        assert_eq!(recovered.version(), prepared.version());
        assert_eq!(recovered.records(), prepared.records());
        assert_eq!(
            recovered.replayed_change_sets(),
            &[prepared.change_set().clone()]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_merges_changed_records_and_preserves_untouched_snapshot_records() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).expect("journal should open");
        let base = KgVersion::initial();
        let untouched = RepositoryRecord::new(
            ChangeTarget::Entity(EntityId::new("untouched").unwrap()),
            Document::from_slice(b"keep"),
        );
        let snapshot = Snapshot::new(base.clone(), vec![untouched.clone()]).unwrap();
        let new_target = ChangeTarget::Entity(EntityId::new("new-record").unwrap());
        let mut changes = ChangeSet::new(Some(base.id().clone()));
        changes.push(Change::new(ChangeKind::Created, new_target.clone()));
        let next = KgVersion::next(&base, changes.id().clone());
        let commit = RecoveryCommit::new(
            base.id().clone(),
            next.clone(),
            changes,
            vec![RepositoryRecord::new(
                new_target,
                Document::from_slice(b"new"),
            )],
        )
        .unwrap();
        journal.prepare(&commit).unwrap();
        journal.commit(commit.change_set().id()).unwrap();

        let recovered = recover_from_snapshot(&snapshot, &journal).unwrap();
        assert_eq!(recovered.version(), &next);
        assert_eq!(recovered.records().len(), 2);
        assert!(recovered.records().iter().any(|item| item == &untouched));
        assert!(
            recovered
                .records()
                .iter()
                .any(|item| item.target().id() == "new-record")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_replaces_a_changed_snapshot_record_without_duplicating_it() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).unwrap();
        let base = KgVersion::initial();
        let target = ChangeTarget::Entity(EntityId::new("existing").unwrap());
        let snapshot_record = RepositoryRecord::new(target.clone(), Document::from_slice(b"old"));
        let snapshot = Snapshot::new(base.clone(), vec![snapshot_record]).unwrap();
        let mut changes = ChangeSet::new(Some(base.id().clone()));
        changes.push(Change::new(ChangeKind::Updated, target.clone()));
        let next = KgVersion::next(&base, changes.id().clone());
        let commit = RecoveryCommit::new(
            base.id().clone(),
            next.clone(),
            changes,
            vec![RepositoryRecord::new(target, Document::from_slice(b"new"))],
        )
        .unwrap();
        journal.prepare(&commit).unwrap();
        journal.commit(commit.change_set().id()).unwrap();

        let recovered = recover_from_snapshot(&snapshot, &journal).unwrap();
        assert_eq!(recovered.version(), &next);
        assert_eq!(recovered.records().len(), 1);
        assert_eq!(recovered.records()[0].document().as_bytes(), b"new");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_from_later_snapshot_skips_journal_prefix_and_replays_later_commit() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).unwrap();
        let v0 = KgVersion::initial();
        let a = ChangeTarget::Entity(EntityId::new("a").unwrap());
        let mut c1 = ChangeSet::new(Some(v0.id().clone()));
        c1.push(Change::new(ChangeKind::Created, a.clone()));
        let v1 = KgVersion::next(&v0, c1.id().clone());
        let commit1 = RecoveryCommit::new(
            v0.id().clone(),
            v1.clone(),
            c1,
            vec![RepositoryRecord::new(
                a.clone(),
                Document::from_slice(b"v1"),
            )],
        )
        .unwrap();
        journal.prepare(&commit1).unwrap();
        journal.commit(commit1.change_set().id()).unwrap();

        let b = ChangeTarget::Entity(EntityId::new("b").unwrap());
        let mut c2 = ChangeSet::new(Some(v1.id().clone()));
        c2.push(Change::new(ChangeKind::Created, b.clone()));
        let v2 = KgVersion::next(&v1, c2.id().clone());
        let commit2 = RecoveryCommit::new(
            v1.id().clone(),
            v2.clone(),
            c2,
            vec![RepositoryRecord::new(
                b.clone(),
                Document::from_slice(b"v2"),
            )],
        )
        .unwrap();
        journal.prepare(&commit2).unwrap();
        journal.commit(commit2.change_set().id()).unwrap();

        let snapshot = Snapshot::new(
            v2.clone(),
            vec![
                RepositoryRecord::new(a.clone(), Document::from_slice(b"v1")),
                RepositoryRecord::new(b.clone(), Document::from_slice(b"v2")),
            ],
        )
        .unwrap();
        let c3_target = ChangeTarget::Entity(EntityId::new("c").unwrap());
        let mut c3 = ChangeSet::new(Some(v2.id().clone()));
        c3.push(Change::new(ChangeKind::Created, c3_target.clone()));
        let v3 = KgVersion::next(&v2, c3.id().clone());
        let commit3 = RecoveryCommit::new(
            v2.id().clone(),
            v3.clone(),
            c3,
            vec![RepositoryRecord::new(
                c3_target,
                Document::from_slice(b"v3"),
            )],
        )
        .unwrap();
        journal.prepare(&commit3).unwrap();
        journal.commit(commit3.change_set().id()).unwrap();

        let recovered = recover_from_snapshot(&snapshot, &journal).unwrap();
        assert_eq!(recovered.version(), &v3);
        assert_eq!(recovered.records().len(), 3);
        assert_eq!(
            recovered.replayed_change_sets(),
            &[commit3.change_set().clone()]
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn incomplete_unterminated_journal_tail_is_ignored_and_repaired_on_append() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).unwrap();
        let first = commit(&KgVersion::initial());
        journal.prepare(&first).unwrap();
        journal.commit(first.change_set().id()).unwrap();
        {
            let mut file = OpenOptions::new()
                .append(true)
                .open(journal.path())
                .unwrap();
            file.write_all(b"{partial").unwrap();
            file.sync_all().unwrap();
        }
        assert_eq!(journal.committed().unwrap(), vec![first.clone()]);

        let next = commit(first.version());
        journal.prepare(&next).unwrap();
        journal.commit(next.change_set().id()).unwrap();
        assert_eq!(journal.committed().unwrap(), vec![first, next]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn complete_unterminated_journal_record_is_preserved_before_next_append() {
        let root = temp_root();
        let mut journal = FileRecoveryJournal::open(&root).unwrap();
        let first = commit(&KgVersion::initial());
        let encoded = serde_json::to_vec(&json!({
            "format_version": JOURNAL_FORMAT_VERSION,
            "type": "prepare",
            "change_set_id": first.change_set().id().as_str(),
            "commit": encode_commit(&first),
        }))
        .unwrap();
        {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(journal.path())
                .unwrap();
            file.write_all(&encoded).unwrap();
            file.sync_all().unwrap();
        }
        // The complete prepare record is parseable but has no newline; appending
        // a commit marker must finish this line and preserve it.
        journal.commit(first.change_set().id()).unwrap();
        assert_eq!(journal.committed().unwrap(), vec![first]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_journal_is_reported() {
        let root = temp_root();
        let journal = FileRecoveryJournal::open(&root).expect("journal should open");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(journal.path())
            .expect("journal file should open");

        writeln!(file, "{{invalid").expect("test corruption should be written");

        let error = journal
            .committed()
            .expect_err("malformed journal must fail recovery");

        assert!(matches!(error, RecoveryError::CorruptJournal(_)));

        let _ = fs::remove_dir_all(root);
    }
}
