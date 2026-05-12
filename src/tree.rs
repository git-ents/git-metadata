//! Tree-rewriting helpers for fanout maintenance.

use crate::Error;

/// Split the hex representation of `oid` into `depth` 2-char prefix segments
/// followed by one leaf segment of the remaining hex characters.
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

fn decode_entries(
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
