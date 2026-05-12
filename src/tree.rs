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

pub(crate) fn decode_entries(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
) -> Result<Vec<gix::objs::tree::Entry>, Error> {
    let tree = repo.find_tree(tree_oid)?;
    let decoded = tree.decode()?;
    Ok(decoded
        .entries
        .iter()
        .map(|e| gix::objs::tree::Entry {
            mode: e.mode,
            filename: e.filename.into(),
            oid: e.oid.into(),
        })
        .collect())
}

/// Insert the tree `leaf_oid` at `path` under `tree_oid`, creating intermediate
/// trees as needed. Returns the new root tree oid. Honors `force` at the leaf
/// and reports collisions against `target`.
pub(crate) fn insert_leaf(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    path: &[gix::bstr::BString],
    leaf_oid: gix::ObjectId,
    force: bool,
    target: gix::ObjectId,
) -> Result<gix::ObjectId, Error> {
    let mut entries = decode_entries(repo, tree_oid)?;
    let (head, rest) = path.split_first().expect("non-empty path");
    let pos = entries.iter().position(|e| e.filename == *head);
    let tree_mode = gix::objs::tree::EntryKind::Tree.into();

    if rest.is_empty() {
        match pos {
            Some(i) => {
                if !force {
                    return Err(Error::AlreadyExists(target));
                }
                entries[i].mode = tree_mode;
                entries[i].oid = leaf_oid;
            }
            None => entries.push(gix::objs::tree::Entry {
                mode: tree_mode,
                filename: head.clone(),
                oid: leaf_oid,
            }),
        }
    } else {
        let sub = match pos {
            Some(i) if entries[i].mode.is_tree() => entries[i].oid,
            Some(_) => return Err(Error::FanoutPathConflict(head.clone())),
            None => repo.write_object(gix::objs::Tree::empty())?.detach(),
        };
        let new_sub = insert_leaf(repo, sub, rest, leaf_oid, force, target)?;
        match pos {
            Some(i) => {
                entries[i].oid = new_sub;
                entries[i].mode = tree_mode;
            }
            None => entries.push(gix::objs::tree::Entry {
                mode: tree_mode,
                filename: head.clone(),
                oid: new_sub,
            }),
        }
    }

    entries.sort();
    Ok(repo.write_object(&gix::objs::Tree { entries })?.detach())
}

/// Remove the entry at `path` under `tree_oid`, pruning intermediate trees
/// that become empty as a result. Returns the new root tree oid. Reports
/// [`Error::NotFound`] if the leaf is absent and [`Error::FanoutPathConflict`]
/// if a non-tree entry occupies an intermediate path segment along the fanout.
pub(crate) fn remove_leaf(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    path: &[gix::bstr::BString],
    target: gix::ObjectId,
) -> Result<gix::ObjectId, Error> {
    match remove_leaf_inner(repo, tree_oid, path, target)? {
        Some(oid) => Ok(oid),
        None => Ok(repo.write_object(gix::objs::Tree::empty())?.detach()),
    }
}

fn remove_leaf_inner(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    path: &[gix::bstr::BString],
    target: gix::ObjectId,
) -> Result<Option<gix::ObjectId>, Error> {
    let mut entries = decode_entries(repo, tree_oid)?;
    let (head, rest) = path.split_first().expect("non-empty path");

    let pos = entries
        .iter()
        .position(|e| e.filename == *head)
        .ok_or(Error::NotFound(target))?;

    if rest.is_empty() {
        entries.remove(pos);
    } else {
        if !entries[pos].mode.is_tree() {
            return Err(Error::FanoutPathConflict(head.clone()));
        }
        match remove_leaf_inner(repo, entries[pos].oid, rest, target)? {
            Some(new_sub) => entries[pos].oid = new_sub,
            None => {
                entries.remove(pos);
            }
        }
    }

    if entries.is_empty() {
        return Ok(None);
    }
    // TODO investigate if tree construction can be less manual
    entries.sort();
    Ok(Some(
        repo.write_object(&gix::objs::Tree { entries })?.detach(),
    ))
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
pub(crate) fn ensure_fanout_blob(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    depth: u8,
) -> Result<gix::ObjectId, Error> {
    let mut entries = decode_entries(repo, tree_oid)?;
    let blob = repo.write_blob(depth.to_string().as_bytes())?.detach();
    let name: gix::bstr::BString = ".fanout".into();
    match entries.iter().position(|e| e.filename == name) {
        Some(i) => {
            entries[i].mode = gix::objs::tree::EntryKind::Blob.into();
            entries[i].oid = blob;
        }
        None => entries.push(gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            filename: name,
            oid: blob,
        }),
    }
    entries.sort();
    Ok(repo.write_object(&gix::objs::Tree { entries })?.detach())
}
