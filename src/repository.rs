use crate::{Error, Metadata};

/// Interact with metadata refs in a Git repository.
///
/// The [`MetadataRepository`] trait is implemented by [`gix::Repository`], but
/// mirrors the `git2::Repository` notes API. When `gix-notes` is designated
/// [usable] by the Gitoxide team, this trait will be updated (with a new major
/// version) to mirror the Gitoxide API.
pub trait MetadataRepository {
    fn metadata(
        &self,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        metadatas_ref: Option<&str>,
        oid: gix::ObjectId,
        metadata: &gix::ObjectId,
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
    fn metadatas(&self, metadatas_ref: Option<&str>) -> Result<Vec<Metadata>, Error>;
    fn find_metadata(
        &self,
        metadatas_ref: Option<&str>,
        id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error>;
}
