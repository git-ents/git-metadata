//! Tree-rewriting helpers for fanout maintenance.

use crate::{DEFAULT_FANOUT, Error};

/// Split the hex representation of `oid` into `depth` 2-char prefix segments
/// followed by one leaf segment of the remaining hex characters.
///
/// The returned vector always has length `depth + 1`; callers may rely on
/// at least one element being present.
pub(crate) fn fanout_path(oid: gix::ObjectId, depth: u8) -> Vec<gix::bstr::BString> {
    let hex = oid.to_hex().to_string();
    let hex = hex.as_bytes();
    let depth = depth as usize;
    let mut out = Vec::with_capacity(depth + 1);
    for i in 0..depth {
        out.push(gix::bstr::BString::from(&hex[2 * i..2 * i + 2]));
    }
    out.push(gix::bstr::BString::from(&hex[2 * depth..]));
    out
}

/// Join path segments with `/` to produce a relative path for the tree `Editor`.
fn join_paths(path: &[gix::bstr::BString]) -> gix::bstr::BString {
    let cap = path.iter().map(|s| s.len()).sum::<usize>() + path.len().saturating_sub(1);
    let mut out = gix::bstr::BString::from(Vec::with_capacity(cap));
    for (i, seg) in path.iter().enumerate() {
        if i > 0 {
            out.push(b'/');
        }
        out.extend_from_slice(seg.as_slice());
    }
    out
}

/// Insert `leaf_oid` at `path` under `tree_oid` with mode `leaf_kind`,
/// creating intermediate trees as needed. Returns the new root tree oid.
/// Honors `force` at the leaf and reports collisions against `target`.
pub(crate) fn insert_leaf(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    path: &[gix::bstr::BString],
    leaf_oid: gix::ObjectId,
    leaf_kind: gix::objs::tree::EntryKind,
    force: bool,
    target: gix::ObjectId,
) -> Result<gix::ObjectId, Error> {
    let tree = repo.find_tree(tree_oid)?;
    if !force && tree.lookup_entry(path.iter().cloned())?.is_some() {
        return Err(Error::AlreadyExists(target));
    }
    let mut editor = tree.edit().map_err(|e| Error::Gix(Box::new(e)))?;
    editor
        .upsert(join_paths(path), leaf_kind, leaf_oid)
        .map_err(|e| Error::Gix(Box::new(e)))?;
    editor
        .write()
        .map(|id| id.detach())
        .map_err(|e| Error::Gix(Box::new(e)))
}

/// Remove the entry at `path` under `tree_oid`, pruning intermediate trees
/// that become empty as a result. Returns the new root tree oid. Reports
/// [`Error::NotFound`] if the leaf is absent.
pub(crate) fn remove_leaf(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    path: &[gix::bstr::BString],
    target: gix::ObjectId,
) -> Result<gix::ObjectId, Error> {
    let tree = repo.find_tree(tree_oid)?;
    if tree.lookup_entry(path.iter().cloned())?.is_none() {
        return Err(Error::NotFound(target));
    }
    let mut editor = tree.edit().map_err(|e| Error::Gix(Box::new(e)))?;
    editor
        .remove(join_paths(path))
        .map_err(|e| Error::Gix(Box::new(e)))?;
    editor
        .write()
        .map(|id| id.detach())
        .map_err(|e| Error::Gix(Box::new(e)))
}

/// Read the fanout depth recorded in the `.fanout` blob at the root of
/// `tree_oid`, falling back to [`DEFAULT_FANOUT`] when absent.
pub(crate) fn fanout_from_tree(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
) -> Result<u8, Error> {
    let tree = repo.find_tree(tree_oid)?;
    let hash_hex_len = tree.id.kind().len_in_hex();
    let Some(entry) = tree.find_entry(".fanout") else {
        return Ok(DEFAULT_FANOUT);
    };
    if !entry.mode().is_blob() {
        return Err(Error::InvalidFanoutType {
            kind: entry.mode().as_str().to_string(),
        });
    }
    let blob = repo.find_blob(entry.oid())?;
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

/// Ensure a `.fanout` blob at the root of `tree_oid` records `depth`.
///
/// - If `.fanout` is absent, writes it and returns the new root oid.
/// - If `.fanout` already records `depth`, returns `tree_oid` unchanged.
/// - If `.fanout` records a different depth, returns [`Error::FanoutDepthConflict`].
/// - If `.fanout` exists but is not a blob, returns [`Error::InvalidFanoutType`].
pub(crate) fn ensure_fanout_blob(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    depth: u8,
) -> Result<gix::ObjectId, Error> {
    let tree = repo.find_tree(tree_oid)?;
    if let Some(entry) = tree.find_entry(".fanout") {
        if !entry.mode().is_blob() {
            return Err(Error::InvalidFanoutType {
                kind: entry.mode().as_str().to_string(),
            });
        }
        let blob = repo.find_blob(entry.oid())?;
        let text = std::str::from_utf8(blob.data.trim_ascii())
            .ok()
            .and_then(|s| s.parse::<u8>().ok());
        if let Some(existing) = text {
            if existing == depth {
                return Ok(tree_oid);
            }
            return Err(Error::FanoutDepthConflict {
                existing,
                requested: depth,
            });
        }
    }
    let blob = repo.write_blob(depth.to_string().as_bytes())?.detach();
    let mut editor = tree.edit().map_err(|e| Error::Gix(Box::new(e)))?;
    editor
        .upsert(".fanout", gix::objs::tree::EntryKind::Blob, blob)
        .map_err(|e| Error::Gix(Box::new(e)))?;
    editor
        .write()
        .map(|id| id.detach())
        .map_err(|e| Error::Gix(Box::new(e)))
}

/// Validate the structural integrity of the fanout tree rooted at `tree_oid`.
///
/// Walks all entries at levels `0..depth`. Every entry at an intermediate level
/// (except `.fanout` at the root) must be a tree; the first non-tree entry
/// returns [`Error::FanoutPathConflict`].
pub(crate) fn validate_fanout_tree(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    depth: u8,
) -> Result<(), Error> {
    validate_at_level(repo, tree_oid, depth, 0)
}

/// Recursively validate entries at `level` under `tree_oid`. Every entry at
/// levels below `total_depth` must be a tree; the `.fanout` blob at level 0
/// is skipped.
fn validate_at_level(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    total_depth: u8,
    level: u8,
) -> Result<(), Error> {
    let tree = repo.find_tree(tree_oid)?;
    let decoded = tree.decode()?;
    for entry in &decoded.entries {
        if level == 0 && entry.filename == b".fanout" {
            continue;
        }
        if level < total_depth {
            if !entry.mode.is_tree() {
                return Err(Error::FanoutPathConflict(gix::bstr::BString::from(
                    entry.filename,
                )));
            }
            validate_at_level(repo, entry.oid.to_owned(), total_depth, level + 1)?;
        }
    }
    Ok(())
}
