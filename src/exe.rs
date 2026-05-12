//! CLI runtime layer: a thin, side-effecting wrapper around the
//! [`MetadataRepository`] trait suitable for driving from `main`.
//!
//! Each [`Repo`] method maps one-to-one onto a CLI subcommand. Output is
//! written to a caller-supplied [`Write`] so the harness can capture it for
//! tests; errors bubble up as [`anyhow::Error`] so the CLI can render them
//! uniformly.

#![allow(dead_code)]

use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use git_metadata::MetadataRepository;
use gix::bstr::BString;
use gix::objs::tree::{Entry, EntryKind};

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

    /// Configured metadata ref (e.g. `refs/metadata/commits`).
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

    /// Print one line per target with metadata, formatted `<target> <tree>`.
    pub fn list_targets(&self, out: &mut dyn Write) -> Result<()> {
        let entries = self.inner.metadatas(Some(&self.metadatas_ref))?;
        for m in entries {
            writeln!(out, "{} {}", m.id(), m.data())?;
        }
        Ok(())
    }

    /// Print one line per leaf in the metadata tree attached to `target`,
    /// formatted like `git ls-tree -r`: `<mode> <type> <oid>\t<path>`.
    pub fn ls_tree(&self, target: gix::ObjectId, out: &mut dyn Write) -> Result<()> {
        let tree_id = self
            .inner
            .find_metadata(Some(&self.metadatas_ref), target)?;
        let tree = self.inner.find_tree(tree_id)?;
        for entry in tree.traverse().breadthfirst.files()? {
            if entry.mode.is_tree() {
                continue;
            }
            writeln!(
                out,
                "{:06o} {} {}\t{}",
                entry.mode.value(),
                entry.mode.as_str(),
                entry.oid,
                entry.filepath
            )?;
        }
        Ok(())
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
    /// Successive calls at different paths compose. Returns the commit id
    /// at the new tip of the metadata ref.
    pub fn upsert(
        &self,
        target: gix::ObjectId,
        path: &str,
        kind: EntryKind,
        oid: gix::ObjectId,
        force: bool,
    ) -> Result<gix::ObjectId> {
        todo!()
    }

    /// Copy entries from `src_tree` matching any of `patterns` into
    /// `target`'s metadata tree, optionally nested under `dest_prefix`.
    ///
    /// `src_tree` is walked breadth-first; every non-tree leaf whose path
    /// matches at least one [glob pattern][gix::glob] is reinserted at
    /// either `<orig_path>` (when `dest_prefix` is `None`) or
    /// `<dest_prefix>/<orig_path>`. Entry modes are preserved verbatim.
    /// All matching entries are folded into one commit on the metadata ref.
    /// `force` controls overwriting existing entries at the destination.
    pub fn import(
        &self,
        target: gix::ObjectId,
        src_tree: gix::ObjectId,
        patterns: &[&str],
        dest_prefix: Option<&str>,
        force: bool,
    ) -> Result<gix::ObjectId> {
        todo!()
    }

    /// Remove entries from `target`'s metadata tree by glob pattern.
    ///
    /// With `keep = false`, entries whose path matches any pattern are
    /// removed; with `keep = true`, the predicate is inverted (entries that
    /// match are retained, everything else is removed). When the metadata
    /// tree is left empty, the fanout leaf is deleted entirely.
    pub fn remove(&self, target: gix::ObjectId, patterns: &[&str], keep: bool) -> Result<()> {
        todo!()
    }

    /// Copy `from`'s metadata tree to `to`. Returns the commit id at the new
    /// tip of the metadata ref.
    pub fn copy(
        &self,
        from: gix::ObjectId,
        to: gix::ObjectId,
        force: bool,
    ) -> Result<gix::ObjectId> {
        todo!()
    }

    /// Drop entries whose target oid no longer exists in the object database.
    ///
    /// Returns the number of entries pruned (or that would be pruned, if
    /// `dry_run`). Prints one target oid per line to `out`.
    pub fn prune(&self, dry_run: bool, out: &mut dyn Write) -> Result<usize> {
        todo!()
    }

    /// Print the configured metadata ref to `out` (the `get-ref` subcommand).
    pub fn get_ref(&self, out: &mut dyn Write) -> Result<()> {
        todo!()
    }

    fn committer(&self) -> Result<gix::actor::SignatureRef<'_>> {
        todo!()
    }

    fn current_metadata_tree(&self, target: gix::ObjectId) -> Result<gix::ObjectId> {
        todo!()
    }
}

enum UpsertError {
    Exists,
    NonTreeIntermediate(BString),
    Other(anyhow::Error),
}

enum RemoveError {
    NotFound,
    NonTreeIntermediate(BString),
    Other(anyhow::Error),
}

fn compile_patterns(patterns: &[&str]) -> Result<Vec<gix::glob::Pattern>> {
    todo!()
}

fn split_path(path: &str) -> Result<Vec<BString>> {
    todo!()
}

fn decode_entries(repo: &gix::Repository, tree: gix::ObjectId) -> Result<Vec<Entry>> {
    todo!()
}

fn upsert_path(
    repo: &gix::Repository,
    tree: gix::ObjectId,
    path: &[BString],
    leaf: gix::ObjectId,
    leaf_kind: EntryKind,
    force: bool,
) -> Result<gix::ObjectId, UpsertError> {
    todo!()
}

fn remove_path(
    repo: &gix::Repository,
    tree: gix::ObjectId,
    path: &[BString],
) -> Result<gix::ObjectId, RemoveError> {
    todo!()
}

fn remove_path_inner(
    repo: &gix::Repository,
    tree: gix::ObjectId,
    path: &[BString],
) -> Result<Option<gix::ObjectId>, RemoveError> {
    todo!()
}

fn tree_is_empty(repo: &gix::Repository, tree: gix::ObjectId) -> Result<bool> {
    todo!()
}
