//! Relate data of any shape to Git objects.

mod error;
mod metadata;
mod repository;

pub use error::Error;
pub use metadata::Metadata;
pub use repository::MetadataRepository;

use gix::bstr::ByteSlice;

impl MetadataRepository for gix::Repository {
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
        let metadatas_ref = match metadatas_ref {
            Some(r) => r,
            None => &self.metadata_default_ref()?,
        };

        let tree = self.find_reference(metadatas_ref)?.peel_to_tree()?;

        // TODO
        // We ASSUME the fanout tree uses the same hash kind as the repository.
        // Is this a safe assumption?
        let hash_hex_len = tree.id.kind().len_in_hex();
        let entries = tree.traverse().breadthfirst.files()?;
        let mut out = Vec::new();

        // We use a pre-allocated buffer to avoid allocations during iteration.
        let mut hex: Vec<u8> = Vec::with_capacity(hash_hex_len);

        for entry in entries {
            if !entry.mode.is_tree() {
                continue;
            }

            // Clear the pre-allocated buffer.
            hex.clear();

            let mut hex_only = true;
            // PERF
            // We use `split_str` here because the literal `b"/"` reads more
            // clearly than a predicate. If this loop ever shows up as a hot
            // path, consider switching to `entry.filepath.split(|b| *b == b'/')`:
            // the slice method compiles down to a single-byte compare per
            // element, whereas `split_str` runs a general substring search
            // that is heavier than necessary for a one-byte separator.
            for seg in entry.filepath.split_str(b"/") {
                if !seg.iter().all(u8::is_ascii_hexdigit) {
                    hex_only = false;
                    break;
                }
                hex.extend_from_slice(seg);
            }
            if !hex_only || hex.len() != hash_hex_len {
                continue;
            }
            let id = gix::ObjectId::from_hex(&hex).map_err(|_| Error::InvalidFanoutLeaf {
                path: entry.filepath.clone(),
            })?;
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
