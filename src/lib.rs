//! Relate data of any shape to Git objects.

/// Errors when interacting with metadata refs in a Git repository.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("metadata not found for object {0}")]
    NotFound(gix::ObjectId),
    #[error("metadata has invalid type {0}")]
    InvalidType(gix::object::Kind),
    #[error("metadata ref has inconsistent fanout depths")]
    InconsistentFanout,
}

/// A `gix::ObjectId` guaranteed to refer to a tree object.
///
/// Only constructible via `From<gix::Tree<'_>>`, providing compile-time proof
/// that the ID was obtained from a verified tree object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeId(gix::ObjectId);

impl<'repo> From<gix::Tree<'repo>> for TreeId {
    fn from(tree: gix::Tree<'repo>) -> Self {
        Self(tree.id)
    }
}

impl From<TreeId> for gix::ObjectId {
    fn from(tree: TreeId) -> Self {
        tree.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata {
    pub tree: TreeId,
}

/// Interact with metadata refs in a Git repository.
pub trait MetadataRepository {
    fn metadata(
        &self,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        metadatas_ref: Option<&str>,
        oid: gix::ObjectId,
        metadata: &str,
        force: bool,
    ) -> Result<gix::ObjectId, Error>;
    fn metadata_default_ref(&self) -> Result<String, Error>;
    fn metadata_delete(
        &self,
        id: gix::ObjectId,
        metadatas_ref: Option<&str>,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
    ) -> Result<(), Error>;
    fn metadatas(
        &self,
        metadatas_ref: Option<&str>,
    ) -> Result<Box<dyn Iterator<Item = gix::ObjectId>>, Error>;
    fn find_metadata(
        &self,
        metadatas_ref: Option<&str>,
        id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error>;
}
