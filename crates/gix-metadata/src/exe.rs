//! CLI runtime layer: a thin, side-effecting wrapper around the
//! [`MetadataRepository`] trait suitable for driving from `main`.
//!
//! Each [`Executor`] method maps one-to-one onto a CLI subcommand. Methods
//! return structured data; callers own formatting. Errors bubble up as
//! [`anyhow::Error`] so the CLI can render them uniformly.

#![allow(dead_code, unused_variables)]

use std::path::Path;

use anyhow::{Context, Result};
use gix::bstr::BString;
use gix::objs::tree::{Entry, EntryKind};

use crate::{Error as MetadataError, MetadataRepository, tree as helpers};

/// A single leaf entry from a metadata tree, returned by [`Executor::ls_tree`].
pub struct TreeEntry {
    pub mode: gix::objs::tree::EntryMode,
    pub oid: gix::ObjectId,
    pub path: String,
}

/// Handle to an open repository, parameterized over the metadata ref name.
pub struct Executor {
    inner: gix::Repository,
    metadatas_ref: String,
}

impl Executor {
    /// Open the repository at `path`, or discover from the current directory.
    pub fn open(path: Option<&Path>) -> Result<Self> {
        let inner = match path {
            Some(p) => gix::discover(p).with_context(|| format!("opening repo at {p:?}"))?,
            None => gix::discover(".").context("discovering repo from current directory")?,
        };
        let metadatas_ref = inner.metadata_default_ref()?;
        Ok(Self {
            inner,
            metadatas_ref,
        })
    }

    /// Override the metadata ref used by every subsequent operation.
    pub fn with_ref(mut self, r: impl Into<String>) -> Self {
        self.metadatas_ref = r.into();
        self
    }

    /// Configured metadata ref (e.g. `refs/metadata/objects`).
    pub fn metadatas_ref(&self) -> &str {
        &self.metadatas_ref
    }

    /// Resolve a revision string (OID, ref, `HEAD~2`, …) to an object id.
    pub fn resolve_oid(&self, rev: &str) -> Result<gix::ObjectId> {
        let id = self
            .inner
            .rev_parse_single(rev)
            .with_context(|| format!("resolving revision `{rev}`"))?;
        Ok(id.detach())
    }

    /// Return all targets that have metadata.
    pub fn list_targets(&self) -> Result<Vec<crate::Metadata>> {
        if self
            .inner
            .try_find_reference(&self.metadatas_ref)?
            .is_none()
        {
            return Ok(Vec::new());
        }
        Ok(self.inner.metadatas(Some(&self.metadatas_ref))?)
    }

    /// Return all leaf entries in the metadata tree attached to `target`.
    pub fn ls_tree(&self, target: gix::ObjectId) -> Result<Vec<TreeEntry>> {
        if self
            .inner
            .try_find_reference(&self.metadatas_ref)?
            .is_none()
        {
            return Ok(Vec::new());
        }
        let tree_id = self
            .inner
            .find_metadata(Some(&self.metadatas_ref), target)?;
        let tree = self.inner.find_tree(tree_id)?;
        let mut out = Vec::new();
        for entry in tree.traverse().breadthfirst.files()? {
            if entry.mode.is_tree() {
                continue;
            }
            out.push(TreeEntry {
                mode: entry.mode,
                oid: entry.oid,
                path: entry.filepath.to_string(),
            });
        }
        Ok(out)
    }

    /// Plant `oid` at `path` inside `target`'s metadata tree.
    ///
    /// The primitive single-entry insert. `oid` is the pre-written object
    /// id and `kind` selects the entry mode:
    ///
    /// - [`EntryKind::Blob`] / [`EntryKind::BlobExecutable`] / [`EntryKind::Link`]:
    ///   the oid is trusted as-is; the bytes are not fetched or validated.
    ///   For `Link`, the blob's content is the symlink target.
    /// - [`EntryKind::Commit`] (gitlink): the oid is trusted as a commit
    ///   pointer; it is *not* required to exist in this repository, since
    ///   gitlinks model unresolved references (submodule semantics).
    /// - [`EntryKind::Tree`]: the oid is verified to exist and decode as a
    ///   tree, because a bogus tree oid would silently corrupt the parent.
    ///
    /// When `force` is false and an entry already exists at `path`, returns
    /// an error. Successive calls at different paths compose. Returns the
    /// commit id at the new tip of the metadata ref.
    ///
    /// `message` overrides the default commit message; `author` overrides
    /// the configured identity for the commit's author (committer is always
    /// taken from config).
    #[allow(clippy::too_many_arguments)]
    pub fn upsert(
        &self,
        target: gix::ObjectId,
        path: &str,
        kind: EntryKind,
        oid: gix::ObjectId,
        force: bool,
        message: Option<&str>,
        author: Option<gix::actor::SignatureRef<'_>>,
        shard_level: u8,
    ) -> Result<gix::ObjectId> {
        // Only Tree needs validation at this time: a bad blob/link/gitlink oid
        // is a broken leaf (content reads fail) but a bad tree oid breaks
        // traversal of the parent immediately, corrupting every path that
        // passes through it.
        if matches!(kind, EntryKind::Tree) {
            let header = self
                .inner
                .try_find_header(oid)?
                .ok_or_else(|| anyhow::anyhow!("tree object {oid} not found"))?;
            if header.kind() != gix::object::Kind::Tree {
                anyhow::bail!("object {oid} is not a tree (found {:?})", header.kind());
            }
        }

        let subtree = self.metadata_subtree_or_empty(target)?;
        let segs = split_path(path)?;
        let new_subtree =
            helpers::insert_leaf(self.repo(), subtree, &segs, oid, kind, force, target)?;

        let committer = self.committer()?;
        let author = author.unwrap_or(committer);
        self.inner
            .metadata(
                author,
                committer,
                message,
                Some(&self.metadatas_ref),
                target,
                &new_subtree,
                true,
                Some(shard_level),
            )
            .map_err(Into::into)
    }

    /// Remove entries from `target`'s metadata tree by glob pattern.
    ///
    /// Entries whose path matches any pattern are removed. When the metadata
    /// tree is left empty the fanout leaf is deleted entirely and `None` is
    /// returned; otherwise returns the commit id at the new tip of the
    /// metadata ref. See [`upsert`](Self::upsert) for `message` and
    /// `author` semantics.
    pub fn remove(
        &self,
        target: gix::ObjectId,
        patterns: &[&str],
        message: Option<&str>,
        author: Option<gix::actor::SignatureRef<'_>>,
    ) -> Result<Option<gix::ObjectId>> {
        let pats = compile_patterns(patterns)?;
        let subtree_id = self
            .inner
            .find_metadata(Some(&self.metadatas_ref), target)?;
        let subtree = self.inner.find_tree(subtree_id)?;

        let mut matched: Vec<gix::bstr::BString> = Vec::new();
        for entry in subtree.traverse().breadthfirst.files()? {
            if pats.iter().any(|p| {
                p.matches(
                    gix::bstr::BStr::new(&entry.filepath),
                    gix::glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
                )
            }) {
                matched.push(entry.filepath.clone());
            }
        }

        if matched.is_empty() {
            return Ok(None);
        }

        let mut editor = subtree.edit().context("creating tree editor")?;
        for path in &matched {
            editor.remove(path.clone()).context("removing entry")?;
        }
        let new_subtree_id = editor
            .write()
            .map(|id| id.detach())
            .context("writing subtree")?;

        let committer = self.committer()?;
        let author = author.unwrap_or(committer);

        if tree_is_empty(&self.inner, new_subtree_id)? {
            self.inner.metadata_delete(
                target,
                Some(&self.metadatas_ref),
                author,
                committer,
                message,
            )?;
            return Ok(None);
        }

        let commit_id = self.inner.metadata(
            author,
            committer,
            message,
            Some(&self.metadatas_ref),
            target,
            &new_subtree_id,
            true,
            None,
        )?;
        Ok(Some(commit_id))
    }

    /// List targets whose oid no longer exists in the object database.
    ///
    /// Read-only counterpart to [`prune`](Self::prune): returns the same set
    /// of targets `prune` would drop, without modifying the metadata ref.
    pub fn stale(&self) -> Result<Vec<gix::ObjectId>> {
        if self
            .inner
            .try_find_reference(&self.metadatas_ref)?
            .is_none()
        {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for (id, _data) in crate::raw_entries(&self.inner, &self.metadatas_ref)? {
            if self.inner.try_find_header(id)?.is_none() {
                out.push(id);
            }
        }
        Ok(out)
    }

    /// Copy `from`'s metadata tree to `to`.
    ///
    /// `force` controls whether existing entries at the destination are
    /// overwritten. Returns the commit id at the new tip of the metadata ref.
    pub fn copy(
        &self,
        from: gix::ObjectId,
        to: gix::ObjectId,
        force: bool,
    ) -> Result<gix::ObjectId> {
        let subtree = self.inner.find_metadata(Some(&self.metadatas_ref), from)?;
        let committer = self.committer()?;
        self.inner
            .metadata(
                committer,
                committer,
                None,
                Some(&self.metadatas_ref),
                to,
                &subtree,
                force,
                None,
            )
            .map_err(Into::into)
    }

    /// Drop entries whose target oid no longer exists in the object database.
    ///
    /// Returns the pruned (or would-be-pruned) oids. When `dry_run` is true
    /// the metadata ref is not modified.
    pub fn prune(&self, dry_run: bool) -> Result<Vec<gix::ObjectId>> {
        let stale = self.stale()?;
        if !dry_run {
            let committer = self.committer()?;
            for id in &stale {
                self.inner.metadata_delete(
                    *id,
                    Some(&self.metadatas_ref),
                    committer,
                    committer,
                    None,
                )?;
            }
        }
        Ok(stale)
    }

    /// Read the blob content at `path` under `target`'s metadata tree.
    ///
    /// Errors if no metadata tree exists for `target`, if `path` is absent,
    /// or if the entry at `path` is not a blob.
    pub fn read_blob_at(&self, target: gix::ObjectId, path: &str) -> Result<Vec<u8>> {
        let tree_id = self
            .inner
            .find_metadata(Some(&self.metadatas_ref), target)?;
        let tree = self.inner.find_tree(tree_id)?;
        let segs = split_path(path)?;
        let entry = tree
            .lookup_entry(segs.iter().cloned())?
            .ok_or_else(|| anyhow::anyhow!("no entry at {path:?} for {target}"))?;
        if !entry.mode().is_blob() {
            anyhow::bail!(
                "entry at {path:?} is not a blob (mode {})",
                entry.mode().as_str()
            );
        }
        let blob = self.inner.find_blob(entry.object_id())?;
        Ok(blob.data.clone())
    }

    /// Merge `source_rev`'s metadata commit into the current metadata ref.
    ///
    /// Delegates to [`gix::Repository::merge_trees`] for 3-way merging.
    /// Aborts (returns an error listing the conflicting paths) if any
    /// unresolved conflict remains. Returns the new tip of the metadata
    /// ref on success; the returned id equals the prior tip when the
    /// merge was a no-op (already up to date).
    pub fn merge(&self, source_rev: &str, message: Option<&str>) -> Result<gix::ObjectId> {
        let source_id = self
            .inner
            .rev_parse_single(source_rev)
            .with_context(|| format!("resolving source `{source_rev}`"))?
            .detach();
        let source_header = self
            .inner
            .try_find_header(source_id)?
            .ok_or_else(|| anyhow::anyhow!("source object {source_id} not found"))?;
        if source_header.kind() != gix::object::Kind::Commit {
            anyhow::bail!(
                "source `{source_rev}` resolves to a {:?}; expected a commit",
                source_header.kind()
            );
        }

        let dest_ref = self.metadatas_ref.clone();
        let dest_id_opt = match self.inner.try_find_reference(&dest_ref)? {
            Some(mut r) => Some(r.peel_to_id()?.detach()),
            None => None,
        };

        let Some(dest_id) = dest_id_opt else {
            self.inner.reference(
                dest_ref.as_str(),
                source_id,
                gix::refs::transaction::PreviousValue::MustNotExist,
                message.unwrap_or("metadata: merge"),
            )?;
            return Ok(source_id);
        };

        if dest_id == source_id {
            return Ok(dest_id);
        }

        let base_id = self
            .inner
            .merge_base(dest_id, source_id)
            .with_context(|| format!("finding merge base of {dest_id} and {source_id}"))?
            .detach();

        if base_id == source_id {
            return Ok(dest_id);
        }
        if base_id == dest_id {
            self.inner.reference(
                dest_ref.as_str(),
                source_id,
                gix::refs::transaction::PreviousValue::ExistingMustMatch(
                    gix::refs::Target::Object(dest_id),
                ),
                message.unwrap_or("metadata: fast-forward"),
            )?;
            return Ok(source_id);
        }

        let base_tree = self.inner.find_commit(base_id)?.tree_id()?.detach();
        let ours_tree = self.inner.find_commit(dest_id)?.tree_id()?.detach();
        let theirs_tree = self.inner.find_commit(source_id)?.tree_id()?.detach();

        let options = self
            .inner
            .tree_merge_options()
            .context("loading tree merge options")?;
        let labels = gix::merge::blob::builtin_driver::text::Labels {
            ancestor: Some("base".into()),
            current: Some("ours".into()),
            other: Some("theirs".into()),
        };
        let mut outcome = self
            .inner
            .merge_trees(base_tree, ours_tree, theirs_tree, labels, options)
            .context("merging metadata trees")?;

        let unresolved = gix::merge::tree::TreatAsUnresolved::default();
        if outcome.has_unresolved_conflicts(unresolved) {
            let paths: Vec<String> = outcome
                .conflicts
                .iter()
                .filter(|c| c.is_unresolved(unresolved))
                .map(|c| c.ours.location().to_string())
                .collect();
            anyhow::bail!("merge conflict at: {}", paths.join(", "));
        }

        let merged_tree = outcome
            .tree
            .write()
            .context("writing merged tree")?
            .detach();

        let committer = self.committer()?;
        let commit = gix::objs::Commit {
            message: message.unwrap_or("metadata: merge").into(),
            tree: merged_tree,
            author: committer.into(),
            committer: committer.into(),
            encoding: None,
            parents: vec![dest_id, source_id].into_iter().collect(),
            extra_headers: Default::default(),
        };
        let commit_id = self.inner.write_object(&commit)?.detach();
        self.inner.reference(
            dest_ref.as_str(),
            commit_id,
            gix::refs::transaction::PreviousValue::ExistingMustMatch(gix::refs::Target::Object(
                dest_id,
            )),
            message.unwrap_or("metadata: merge"),
        )?;
        Ok(commit_id)
    }

    /// The underlying `gix` repository handle.
    pub fn repo(&self) -> &gix::Repository {
        &self.inner
    }

    fn committer(&self) -> Result<gix::actor::SignatureRef<'_>> {
        match self.inner.committer() {
            Some(Ok(s)) => Ok(s),
            Some(Err(e)) => Err(e.into()),
            None => Err(anyhow::anyhow!("no committer identity configured")),
        }
    }

    fn current_metadata_tree(&self, target: gix::ObjectId) -> Result<gix::ObjectId> {
        self.inner
            .find_metadata(Some(self.metadatas_ref()), target)
            .map_err(Into::into)
    }

    /// Like [`current_metadata_tree`], but returns an empty tree id when no
    /// leaf exists yet for `target` (or the ref hasn't been created).
    ///
    /// [`current_metadata_tree`]: Self::current_metadata_tree
    fn metadata_subtree_or_empty(&self, target: gix::ObjectId) -> Result<gix::ObjectId> {
        if self
            .inner
            .try_find_reference(&self.metadatas_ref)?
            .is_none()
        {
            return Ok(self.inner.write_object(gix::objs::Tree::empty())?.detach());
        }
        match self.inner.find_metadata(Some(&self.metadatas_ref), target) {
            Ok(t) => Ok(t),
            Err(MetadataError::NotFound(_)) => {
                Ok(self.inner.write_object(gix::objs::Tree::empty())?.detach())
            }
            Err(e) => Err(e.into()),
        }
    }
}

enum RemoveError {
    NotFound,
    NonTreeIntermediate(BString),
    Other(anyhow::Error),
}

fn compile_patterns(patterns: &[&str]) -> Result<Vec<gix::glob::Pattern>> {
    patterns
        .iter()
        .map(|p| {
            gix::glob::parse(p.as_bytes())
                .ok_or_else(|| anyhow::anyhow!("invalid glob pattern: {p:?}"))
        })
        .collect()
}

fn split_path(path: &str) -> Result<Vec<BString>> {
    let segs: Vec<BString> = path
        .split('/')
        .map(|s| BString::from(s.as_bytes()))
        .collect();
    for s in &segs {
        let bytes: &[u8] = s.as_ref();
        if bytes.is_empty() || bytes == b"." || bytes == b".." {
            anyhow::bail!("invalid path {path:?}");
        }
    }
    Ok(segs)
}

fn decode_entries(repo: &gix::Repository, tree: gix::ObjectId) -> Result<Vec<Entry>> {
    let t = repo.find_tree(tree)?;
    let decoded = t.decode()?;
    Ok(decoded
        .entries
        .iter()
        .map(|e| Entry {
            mode: e.mode,
            filename: e.filename.into(),
            oid: e.oid.into(),
        })
        .collect())
}

fn remove_path(
    repo: &gix::Repository,
    tree: gix::ObjectId,
    path: &[BString],
    target: gix::ObjectId,
) -> Result<gix::ObjectId, RemoveError> {
    helpers::remove_leaf(repo, tree, path, target).map_err(|e| match e {
        MetadataError::NotFound(_) => RemoveError::NotFound,
        other => RemoveError::Other(other.into()),
    })
}

fn tree_is_empty(repo: &gix::Repository, tree: gix::ObjectId) -> Result<bool> {
    let t = repo.find_tree(tree)?;
    let decoded = t.decode()?;
    Ok(decoded.entries.is_empty())
}
