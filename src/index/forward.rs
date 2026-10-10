//! Forward logical lookup boundary over Nizaam Indexing.
//!
//! Forward lookup resolves a source-owned `ObjectReference` to the opaque
//! `IndexAssignedId` exposed by Indexing. The KG does not implement a physical
//! lookup structure here.

use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::reference::ObjectReference;

/// Logical forward lookup adapter owned by the caller of the KG index module.
///
/// Implementations may delegate to Nizaam Indexing or another approved logical
/// adapter. The trait intentionally exposes no physical storage operations.
pub trait ForwardIndexAccess {
    /// Adapter-specific failure type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Resolves a source-owned object reference to its Indexing assigned id.
    fn lookup_forward(
        &self,
        object_reference: &ObjectReference,
    ) -> Result<Option<IndexAssignedId>, Self::Error>;
}

/// Executes one forward lookup through a supplied adapter.
pub fn lookup_forward<A>(
    adapter: &A,
    object_reference: &ObjectReference,
) -> Result<Option<IndexAssignedId>, A::Error>
where
    A: ForwardIndexAccess,
{
    adapter.lookup_forward(object_reference)
}
