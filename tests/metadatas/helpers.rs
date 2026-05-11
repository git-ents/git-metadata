//! Shared helpers for `metadatas` tests.

use std::collections::BTreeMap;

use git_metadata::Metadata;
use gix::bstr::BString;
use gix::objs::tree::Entry;
pub use gix::objs::tree::EntryKind;
use gix::refs::transaction::PreviousValue;

pub const FANOUT_REF: &str = "refs/metadatas/test";

/// Write a tree containing the given leaves. Each leaf is a `(path, kind, oid)`
/// triple; intermediate directories are created automatically. An empty list
/// produces an empty tree.
pub fn write_tree(
    repo: &gix::Repository,
    leaves: Vec<(Vec<BString>, EntryKind, gix::ObjectId)>,
) -> gix::ObjectId {
    let mut groups: BTreeMap<BString, Vec<(Vec<BString>, EntryKind, gix::ObjectId)>> =
        BTreeMap::new();
    let mut entries: Vec<Entry> = Vec::new();
    for (mut path, kind, oid) in leaves {
        assert!(!path.is_empty(), "leaf path must be non-empty");
        let head = path.remove(0);
        if path.is_empty() {
            entries.push(Entry {
                mode: kind.into(),
                filename: head,
                oid,
            });
        } else {
            groups.entry(head).or_default().push((path, kind, oid));
        }
    }
    for (name, children) in groups {
        let subtree = write_tree(repo, children);
        entries.push(Entry {
            mode: EntryKind::Tree.into(),
            filename: name,
            oid: subtree,
        });
    }
    entries.sort();
    let tree = gix::objs::Tree { entries };
    repo.write_object(&tree).expect("write tree").detach()
}

pub fn init_repo() -> (tempfile::TempDir, gix::Repository) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = gix::init(dir.path()).expect("init repo");
    (dir, repo)
}

pub fn set_ref(repo: &gix::Repository, oid: gix::ObjectId) {
    set_ref_named(repo, FANOUT_REF, oid);
}

pub fn set_ref_named(repo: &gix::Repository, name: &str, oid: gix::ObjectId) {
    repo.reference(name, oid, PreviousValue::Any, "set fanout")
        .expect("set ref");
}

pub fn blob(repo: &gix::Repository, bytes: &[u8]) -> gix::ObjectId {
    repo.write_blob(bytes).expect("write blob").detach()
}

pub fn empty_tree(repo: &gix::Repository) -> gix::ObjectId {
    repo.write_object(gix::objs::Tree::empty())
        .expect("write empty tree")
        .detach()
}

/// Split a hex string into `depth` two-byte prefix segments plus one tail
/// segment (the leaf filename).
pub fn fanout_segments(hex: &[u8], depth: usize) -> Vec<BString> {
    let mut out = Vec::with_capacity(depth + 1);
    for i in 0..depth {
        out.push(BString::from(&hex[2 * i..2 * i + 2]));
    }
    out.push(BString::from(&hex[2 * depth..]));
    out
}

/// Build a fanout root tree. If `depth` is `Some(d)`, writes a `.fanout` blob
/// at root containing the decimal `d` and lays out leaves at depth `d`.
/// If `None`, omits the `.fanout` blob and lays out leaves at depth `1`
/// (the default).
pub fn write_fanout(
    repo: &gix::Repository,
    depth: Option<u8>,
    leaves: &[(gix::ObjectId, gix::ObjectId)],
) -> gix::ObjectId {
    let mut entries: Vec<(Vec<BString>, EntryKind, gix::ObjectId)> = Vec::new();
    if let Some(d) = depth {
        let fanout_blob = blob(repo, d.to_string().as_bytes());
        entries.push((vec![".fanout".into()], EntryKind::Blob, fanout_blob));
    }
    let effective_depth = depth.unwrap_or(1) as usize;
    for (id, data) in leaves {
        let hex = hex_of(*id);
        entries.push((
            fanout_segments(&hex, effective_depth),
            EntryKind::Tree,
            *data,
        ));
    }
    write_tree(repo, entries)
}

pub fn hex_of(oid: gix::ObjectId) -> Vec<u8> {
    oid.to_hex().to_string().into_bytes()
}

pub fn expected(repo: &gix::Repository, pairs: &[(gix::ObjectId, gix::ObjectId)]) -> Vec<Metadata> {
    let mut v: Vec<_> = pairs
        .iter()
        .map(|(id, data)| Metadata::new(repo, *id, *data).expect("expected metadata"))
        .collect();
    v.sort_by_key(|m| format!("{m:?}"));
    v
}

pub fn sorted(mut got: Vec<Metadata>) -> Vec<Metadata> {
    got.sort_by_key(|m| format!("{m:?}"));
    got
}
