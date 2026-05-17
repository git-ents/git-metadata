//! Relate data of any shape to Git objects.
//!
//! `gix-metadata` attaches arbitrary tree-shaped data to any Git object,
//! mirroring the storage model used by `git notes` but generalized: where a
//! note is a single blob keyed by the annotated object's id, a metadata entry
//! is a tree keyed the same way. This lets callers attach structured data
//! (multiple files, nested directories) to commits, blobs, trees, or tags
//! without inventing their own ref layout.
//!
//! # Model
//!
//! Entries live under a Git ref (default `refs/metadata/objects`, see
//! [`MetadataRepository::metadata_default_ref`]). The ref points at a commit
//! whose tree is the *fanout tree*: a directory tree that maps an annotated
//! object's hash to a stored metadata tree by splitting the hex id into 2-byte
//! prefix segments. The number of prefix segments is the *fanout depth*, read
//! from a `.fanout` blob at the root of the tree and defaulting to
//! [`DEFAULT_FANOUT`] (the git-notes shape: `ab/cdef…`).
//!
//! See [`MetadataRepository`] for the full description of the fanout layout
//! and the per-method contracts.
//!
//! # Example
//!
//! Attach a metadata tree to a blob and read it back:
//!
//! ```
//! use gix_metadata::MetadataRepository;
//!
//! let dir = tempfile::tempdir().expect("tempdir");
//! let repo = gix::init(dir.path()).expect("init repository");
//!
//! let target = repo.write_blob(b"hello")?.detach();
//! let metadata = repo.write_object(gix::objs::Tree::empty())?.detach();
//!
//! let sig = gix::actor::SignatureRef {
//!     name: "Tester".into(),
//!     email: "t@example.com".into(),
//!     time: "0 +0000".into(),
//! };
//! repo.metadata(sig, sig, None, None, target, &metadata, false, None)?;
//!
//! let entries = repo.metadatas(None)?;
//! assert_eq!(entries.len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod metadata;
mod repository;
mod tree;

pub mod exe;

#[cfg(test)]
mod tests;

pub use error::Error;
pub use metadata::Metadata;
pub use repository::MetadataRepository;

/// Fanout depth assumed when no `.fanout` blob is present at the root of a
/// metadata tree. Matches the git-notes shape (one 2-byte directory level).
pub const DEFAULT_FANOUT: u8 = 1;

fn resolve_ref<'a>(
    repo: &gix::Repository,
    metadatas_ref: Option<&'a str>,
) -> Result<std::borrow::Cow<'a, str>, Error> {
    Ok(match metadatas_ref {
        Some(r) => std::borrow::Cow::Borrowed(r),
        None => std::borrow::Cow::Owned(repo.metadata_default_ref()?),
    })
}

/// Walk the fanout tree at `metadatas_ref` and yield `(target_id, data_id)`
/// for every leaf whose path matches the expected fanout shape. Does not
/// verify that the referenced objects exist or have the expected kinds.
pub(crate) fn raw_entries(
    repo: &gix::Repository,
    metadatas_ref: &str,
) -> Result<Vec<(gix::ObjectId, gix::ObjectId)>, Error> {
    let depth = repo.metadata_ref_fanout(Some(metadatas_ref))?;
    let tree = repo.find_reference(metadatas_ref)?.peel_to_tree()?;
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
            if segs > prefix_segs + 1 || seg.len() != want || !seg.iter().all(u8::is_ascii_hexdigit)
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
        out.push((id, entry.oid));
    }
    Ok(out)
}

impl MetadataRepository for gix::Repository {
    #[allow(clippy::too_many_arguments)]
    fn metadata(
        &self,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        message: Option<&str>,
        metadatas_ref: Option<&str>,
        oid: gix::ObjectId,
        metadata: &gix::ObjectId,
        force: bool,
        initial_depth: Option<u8>,
    ) -> Result<gix::ObjectId, Error> {
        let metadatas_ref = resolve_ref(self, metadatas_ref)?;
        let metadatas_ref = metadatas_ref.as_ref();

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
                    (
                        None,
                        Vec::new(),
                        empty,
                        initial_depth.unwrap_or(DEFAULT_FANOUT),
                    )
                }
            };

        let path = tree::fanout_path(oid, depth);
        let new_root = tree::insert_leaf(
            self,
            root_tree,
            &path,
            *metadata,
            gix::objs::tree::EntryKind::Tree,
            force,
            oid,
        )?;
        let new_root = tree::ensure_fanout_blob(self, new_root, depth)?;

        let commit = gix::objs::Commit {
            message: message.unwrap_or("metadata: update").into(),
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
        self.reference(
            metadatas_ref,
            commit_id,
            expected,
            message.unwrap_or("metadata: update"),
        )?;
        Ok(commit_id)
    }

    /// The current implementation is infallible, but the `Result` is reserved
    /// for future configuration-driven defaults (e.g. a repository config key)
    /// that may surface I/O or parse errors.
    fn metadata_default_ref(&self) -> Result<String, Error> {
        Ok("refs/metadata/objects".to_string())
    }

    fn metadata_ref_fanout(&self, metadatas_ref: Option<&str>) -> Result<u8, Error> {
        let metadatas_ref = resolve_ref(self, metadatas_ref)?;
        let tree = self
            .find_reference(metadatas_ref.as_ref())?
            .peel_to_tree()?;
        tree::fanout_from_tree(self, tree.id)
    }

    fn metadata_delete(
        &self,
        id: gix::ObjectId,
        metadatas_ref: Option<&str>,
        author: gix::actor::SignatureRef<'_>,
        committer: gix::actor::SignatureRef<'_>,
        message: Option<&str>,
    ) -> Result<(), Error> {
        let metadatas_ref = resolve_ref(self, metadatas_ref)?;
        let metadatas_ref = metadatas_ref.as_ref();

        // Snapshot the ref once so depth and target both derive from the
        // same tree, avoiding a torn read against a concurrent writer.
        let target = self.find_reference(metadatas_ref)?.peel_to_id()?.detach();
        let header = self
            .try_find_header(target)?
            .ok_or(Error::NotFound(target))?;
        let (parents, root_tree) = match header.kind() {
            gix::object::Kind::Commit => {
                let commit = self.find_commit(target)?;
                (vec![target], commit.tree_id()?.detach())
            }
            gix::object::Kind::Tree => (Vec::new(), target),
            k => return Err(Error::InvalidRootType(k)),
        };
        let depth = tree::fanout_from_tree(self, root_tree)?;

        let path = tree::fanout_path(id, depth);
        let new_root = tree::remove_leaf(self, root_tree, &path, id)?;

        let commit = gix::objs::Commit {
            message: message.unwrap_or("metadata: delete").into(),
            tree: new_root,
            author: author.into(),
            committer: committer.into(),
            encoding: None,
            parents: parents.into_iter().collect(),
            extra_headers: Default::default(),
        };
        let commit_id = self.write_object(&commit)?.detach();

        let expected = gix::refs::transaction::PreviousValue::ExistingMustMatch(
            gix::refs::Target::Object(target),
        );
        self.reference(
            metadatas_ref,
            commit_id,
            expected,
            message.unwrap_or("metadata: delete"),
        )?;
        Ok(())
    }

    fn metadatas(&self, metadatas_ref: Option<&str>) -> Result<Vec<Metadata>, Error> {
        let metadatas_ref = resolve_ref(self, metadatas_ref)?;
        raw_entries(self, metadatas_ref.as_ref())?
            .into_iter()
            .map(|(id, data)| Metadata::new(self, id, data))
            .collect()
    }

    fn find_metadata(
        &self,
        metadatas_ref: Option<&str>,
        id: gix::ObjectId,
    ) -> Result<gix::ObjectId, Error> {
        let metadatas_ref = resolve_ref(self, metadatas_ref)?;
        let metadatas_ref = metadatas_ref.as_ref();

        let depth = self.metadata_ref_fanout(Some(metadatas_ref))?;
        let path = tree::fanout_path(id, depth);
        let tree = self.find_reference(metadatas_ref)?.peel_to_tree()?;
        let entry = tree
            .lookup_entry(path.iter().cloned())?
            .ok_or(Error::NotFound(id))?;
        if !entry.mode().is_tree() {
            let kind = match entry.mode().kind() {
                gix::objs::tree::EntryKind::Commit => gix::object::Kind::Commit,
                _ => gix::object::Kind::Blob,
            };
            return Err(Error::InvalidType(kind));
        }
        Ok(entry.object_id())
    }

    fn validate_metadata_tree(&self, metadatas_ref: Option<&str>) -> Result<(), Error> {
        let metadatas_ref = resolve_ref(self, metadatas_ref)?;
        let metadatas_ref = metadatas_ref.as_ref();
        let tree = self.find_reference(metadatas_ref)?.peel_to_tree()?;
        let depth = tree::fanout_from_tree(self, tree.id)?;
        tree::validate_fanout_tree(self, tree.id, depth)
    }
}
