//! Relate data of any shape to Git objects.

mod error;
mod metadata;
mod repository;
mod tree;

#[cfg(test)]
mod tests;

pub use error::Error;
pub use metadata::Metadata;
pub use repository::MetadataRepository;

/// Fanout depth assumed when no `.fanout` blob is present at the root of a
/// metadata tree. Matches the git-notes shape (one 2-byte directory level).
pub const DEFAULT_FANOUT: u8 = 1;

impl MetadataRepository for gix::Repository {
    fn metadata(
        &self,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        metadatas_ref: Option<&str>,
        oid: gix::ObjectId,
        metadata: &gix::ObjectId,
        force: bool,
    ) -> Result<gix::ObjectId, Error> {
        let metadatas_ref = match metadatas_ref {
            Some(r) => r,
            None => &self.metadata_default_ref()?,
        };

        if oid.kind() != self.object_hash() {
            return Err(Error::UnsupportedHashKind(oid, oid.kind()));
        }
        let metadata_header = self
            .try_find_header(*metadata)?
            .ok_or(Error::NotFound(*metadata))?;
        if metadata_header.kind() != gix::object::Kind::Tree {
            return Err(Error::InvalidType(metadata_header.kind()));
        }

        // Refs can target any object; we accept a tree-rooted ref as a
        // bootstrap shape and migrate to commit-rooted on first write.
        let (prior_ref_target, parents, root_tree, depth) =
            match self.try_find_reference(metadatas_ref)? {
                Some(mut r) => {
                    let target = r.peel_to_id()?.detach();
                    let header = self
                        .try_find_header(target)?
                        .ok_or(Error::NotFound(target))?;
                    match header.kind() {
                        gix::object::Kind::Commit => {
                            let commit = self.find_commit(target)?;
                            let tree = commit.tree_id()?.detach();
                            let depth = self.metadata_ref_fanout(Some(metadatas_ref))?;
                            (Some(target), vec![target], tree, depth)
                        }
                        gix::object::Kind::Tree => {
                            let depth = self.metadata_ref_fanout(Some(metadatas_ref))?;
                            (Some(target), Vec::new(), target, depth)
                        }
                        k => return Err(Error::InvalidRootType(k)),
                    }
                }
                None => {
                    let empty = self.write_object(gix::objs::Tree::empty())?.detach();
                    (None, Vec::new(), empty, DEFAULT_FANOUT)
                }
            };

        let path = tree::fanout_path(oid, depth);
        let new_root = tree::insert_leaf(self, root_tree, &path, *metadata, force, oid)?;
        let new_root = tree::ensure_fanout_blob(self, new_root, depth)?;

        let commit = gix::objs::Commit {
            message: "metadata: update".into(),
            tree: new_root,
            author: author.into(),
            committer: committer.into(),
            encoding: None,
            parents: parents.into_iter().collect(),
            extra_headers: Default::default(),
        };
        let commit_id = self.write_object(&commit)?.detach();

        // CAS guard: require the ref to still hold the target we snapshotted,
        // so a concurrent writer's commit isn't silently clobbered.
        let expected = match prior_ref_target {
            Some(prior) => gix::refs::transaction::PreviousValue::ExistingMustMatch(
                gix::refs::Target::Object(prior),
            ),
            None => gix::refs::transaction::PreviousValue::MustNotExist,
        };
        self.reference(metadatas_ref, commit_id, expected, "metadata: update")?;
        Ok(commit_id)
    }

    /// The current implementation is infallible, but the `Result` is reserved
    /// for future configuration-driven defaults (e.g. a repository config key)
    /// that may surface I/O or parse errors.
    fn metadata_default_ref(&self) -> Result<String, Error> {
        Ok("refs/metadata/commits".to_string())
    }

    fn metadata_ref_fanout(&self, metadatas_ref: Option<&str>) -> Result<u8, Error> {
        let default_ref;
        let metadatas_ref = match metadatas_ref {
            Some(r) => r,
            None => {
                default_ref = self.metadata_default_ref()?;
                &default_ref
            }
        };
        let tree = self.find_reference(metadatas_ref)?.peel_to_tree()?;
        let hash_hex_len = tree.id.kind().len_in_hex();
        let Some(entry) = tree.find_entry(".fanout") else {
            return Ok(DEFAULT_FANOUT);
        };
        if !entry.mode().is_blob() {
            return Err(Error::InvalidFanoutType {
                kind: entry.mode().as_str().to_string(),
            });
        }
        let blob = self.find_blob(entry.oid())?;
        let text =
            std::str::from_utf8(blob.data.trim_ascii()).map_err(|_| Error::InvalidFanoutDepth {
                value: gix::bstr::BString::from(blob.data.clone()),
            })?;
        let depth: u8 = text.parse().map_err(|_| Error::InvalidFanoutDepth {
            value: gix::bstr::BString::from(text.as_bytes()),
        })?;
        if !(1..=19).contains(&depth) || (2 * depth as usize) >= hash_hex_len {
            return Err(Error::InvalidFanoutDepth {
                value: gix::bstr::BString::from(text.as_bytes()),
            });
        }
        Ok(depth)
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

    /// Walks the fanout tree at `metadatas_ref` and verifies each leaf via
    /// [`Metadata::new`]. Fails fast on the first invalid leaf.
    ///
    /// The fanout depth (number of 2-hex-character directory levels) is read
    /// from a `.fanout` blob at the root of the tree. The blob must contain a
    /// decimal integer in `1..=19`. If the blob is absent, depth defaults to
    /// `1` (git-notes shape: `ab/cdef...`).
    fn metadatas(&self, metadatas_ref: Option<&str>) -> Result<Vec<Metadata>, Error> {
        let default_ref;
        let metadatas_ref = match metadatas_ref {
            Some(r) => r,
            None => {
                default_ref = self.metadata_default_ref()?;
                &default_ref
            }
        };

        let depth = self.metadata_ref_fanout(Some(metadatas_ref))?;
        let tree = self.find_reference(metadatas_ref)?.peel_to_tree()?;
        let hash_hex_len = tree.id.kind().len_in_hex();

        let prefix_segs = depth as usize;
        let leaf_seg_len = hash_hex_len - 2 * prefix_segs;
        let entries = tree.traverse().breadthfirst.files()?;
        let mut out = Vec::new();
        let mut hex: Vec<u8> = Vec::with_capacity(hash_hex_len);

        for entry in entries {
            if !entry.mode.is_tree() {
                continue;
            }
            hex.clear();
            let mut segs = 0usize;
            let mut shape_ok = true;
            for seg in entry.filepath.split(|b| *b == b'/') {
                segs += 1;
                let want = if segs <= prefix_segs { 2 } else { leaf_seg_len };
                if segs > prefix_segs + 1
                    || seg.len() != want
                    || !seg.iter().all(u8::is_ascii_hexdigit)
                {
                    shape_ok = false;
                    break;
                }
                hex.extend_from_slice(seg);
            }
            if !shape_ok || segs != prefix_segs + 1 {
                continue;
            }
            let id = gix::ObjectId::from_hex(&hex).expect("shape-validated hex");
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
