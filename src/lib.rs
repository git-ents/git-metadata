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

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata {
    pub id: gix::ObjectId,
    pub tree: gix::ObjectId,
}

// A lazy iterator over [`Metadata`] entries in a metadata ref.
//
//
pub struct MetadataIter<'repo> {
    repo: &'repo gix::Repository,
    stack: Vec<(Vec<u8>, gix::ObjectId)>,
}

impl<'repo> Iterator for MetadataIter<'repo> {
    type Item = Result<Metadata, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// Interact with metadata refs in a Git repository.
///
/// Git Metadata usage is most similar to Git Note usage. For this reason, the
/// `MetadataRepository` trait is _shaped_ like a Git Note API, but with trees.
/// While the `git-metadata` project uses `gix` to interact with Git repositories,
/// the `MetadataRepository` trait is based off of the `git2::Repository` methods
/// for Git Note reading and writing. The `gix-note` crate is currently empty.
/// If `gix-note` stabilizes, this project will likely release a new major version
/// to match the `gix-note` API.
pub trait MetadataRepository {
    fn metadata(
        &self,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        metadatas_ref: Option<&str>,
        oid: gix::ObjectId,
        metadata: &Metadata,
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
    fn metadatas(&self, metadatas_ref: Option<&str>) -> Result<MetadataIter<'_>, Error>;
    fn find_metadata(
        &self,
        metadatas_ref: Option<&str>,
        id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error>;
}
