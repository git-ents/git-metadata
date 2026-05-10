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
    fn metadatas(&self, metadatas_ref: Option<&str>) -> Result<Vec<Metadata>, Error> {
        let default_ref;
        let metadatas_ref = match metadatas_ref {
            Some(r) => r,
            None => {
                default_ref = self.metadata_default_ref()?;
                &default_ref
            }
        };
        let root = self
            .find_reference(metadatas_ref)
            .map_err(|_| todo!("resolve {metadatas_ref:?} to a tree id"))?
            .id()
            .detach();
        let hash_hex_len = root.kind().len_in_hex();
        let tree = self
            .find_object(root)
            .map_err(|_| Error::NotFound(root))?
            .into_tree();
        let entries = tree
            .traverse()
            .breadthfirst
            .files()
            .map_err(|_| Error::NotFound(root))?;
        let mut out = Vec::new();
        for entry in entries {
            if !entry.mode.is_tree() {
                continue;
            }
            let hex: Vec<u8> = entry
                .filepath
                .iter()
                .copied()
                .filter(|b| *b != b'/')
                .collect();
            if hex.len() != hash_hex_len {
                continue;
            }
            let Ok(id) = gix::ObjectId::from_hex(&hex) else {
                continue;
            };
            out.push(Metadata::new(self, id, entry.oid)?);
        }
        Ok(out)
    }

    fn find_metadata(
        &self,
        _metadatas_ref: Option<&str>,
        _id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error> {
        todo!()
    }
}
