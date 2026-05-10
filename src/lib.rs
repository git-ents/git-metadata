//! Relate data of any shape to Git objects.

mod error;
mod metadata;
mod repository;

pub use error::Error;
pub use gix::Repository;
pub use metadata::Metadata;
pub use repository::MetadataRepository;

impl MetadataRepository for Repository {
    fn metadata(
        &self,
        _author: gix::actor::SignatureRef<'_>,
        _committer: gix::actor::SignatureRef<'_>,
        _metadatas_ref: Option<&str>,
        _oid: gix::ObjectId,
        _metadata: &gix::ObjectId,
        _force: bool,
    ) -> Result<gix::ObjectId, Error> {
        todo!()
    }

    fn metadata_default_ref(&self) -> Result<String, Error> {
        todo!()
    }

    fn metadata_delete(
        &self,
        _id: gix::ObjectId,
        _metadatas_ref: Option<&str>,
        _author: gix::actor::SignatureRef<'_>,
        _committer: gix::actor::SignatureRef<'_>,
    ) -> Result<(), Error> {
        todo!()
    }

    /// Walks the fanout tree at `metadatas_ref` breadth-first and verifies each
    /// leaf via [`Metadata::new`]. Fails fast on the first invalid leaf.
    fn metadatas(&self, _metadatas_ref: Option<&str>) -> Result<Vec<Metadata>, Error> {
        todo!()
    }

    fn find_metadata(
        &self,
        _metadatas_ref: Option<&str>,
        _id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error> {
        todo!()
    }
}
