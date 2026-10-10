//! Reverse logical lookup boundary over Nizaam Indexing.
//!
//! Reverse lookup resolves an opaque Indexing assigned id back to the
//! source-owned `ObjectReference`. The KG does not implement a physical reverse
//! index here.

use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::reference::ObjectReference;

/// Logical reverse lookup adapter owned by the caller of the KG index module.
///
/// Implementations may delegate to Nizaam Indexing or another approved logical
/// adapter. The trait intentionally exposes no physical storage operations.
pub trait ReverseIndexAccess {
    /// Adapter-specific failure type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Resolves an Indexing assigned id to its source-owned object reference.
    fn lookup_reverse(
        &self,
        assigned_id: &IndexAssignedId,
    ) -> Result<Option<ObjectReference>, Self::Error>;
}

/// Executes one reverse lookup through a supplied adapter.
pub fn lookup_reverse<A>(
    adapter: &A,
    assigned_id: &IndexAssignedId,
) -> Result<Option<ObjectReference>, A::Error>
where
    A: ReverseIndexAccess,
{
    adapter.lookup_reverse(assigned_id)
}
