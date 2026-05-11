//! Shared helpers for `metadatas` tests.

use std::collections::BTreeMap;

use git_metadata::Metadata;
use gix::bstr::BString;
use gix::objs::tree::{Entry, EntryKind};
use gix::refs::transaction::PreviousValue;

pub const FANOUT_REF: &str = "refs/metadatas/test";

/// Builder node for synthesizing fanout trees.
pub enum Node {
    /// An intermediate fanout directory. Children are serialized as a tree.
    Dir(BTreeMap<BString, Node>),
    /// A pre-existing tree referenced from this entry (Tree mode).
    TreeRef(gix::ObjectId),
    /// A pre-existing blob referenced from this entry (Blob mode).
    BlobRef(gix::ObjectId),
}

impl Node {
    pub fn dir() -> Self {
        Node::Dir(BTreeMap::new())
    }

    pub fn insert(&mut self, path: &[&[u8]], leaf: Node) {
        match self {
            Node::Dir(map) => {
                let (head, tail) = path.split_first().expect("non-empty path");
                if tail.is_empty() {
                    map.insert(BString::from(*head), leaf);
                } else {
                    map.entry(BString::from(*head))
                        .or_insert_with(Node::dir)
                        .insert(tail, leaf);
                }
            }
            _ => panic!("insert into non-directory node"),
        }
    }
}

pub fn write_tree(repo: &gix::Repository, node: &Node) -> gix::ObjectId {
    let Node::Dir(map) = node else {
        panic!("write_tree called on non-directory");
    };
    let mut entries: Vec<Entry> = map
        .iter()
        .map(|(name, child)| match child {
            Node::Dir(_) => Entry {
                mode: EntryKind::Tree.into(),
                filename: name.clone(),
                oid: write_tree(repo, child),
            },
            Node::TreeRef(oid) => Entry {
                mode: EntryKind::Tree.into(),
                filename: name.clone(),
                oid: *oid,
            },
            Node::BlobRef(oid) => Entry {
                mode: EntryKind::Blob.into(),
                filename: name.clone(),
                oid: *oid,
            },
        })
        .collect();
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
pub fn fanout_segments(hex: &[u8], depth: usize) -> Vec<Vec<u8>> {
    let mut out = Vec::with_capacity(depth + 1);
    for i in 0..depth {
        out.push(hex[2 * i..2 * i + 2].to_vec());
    }
    out.push(hex[2 * depth..].to_vec());
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
    let mut root = Node::dir();
    if let Some(d) = depth {
        let fanout_blob = blob(repo, d.to_string().as_bytes());
        root.insert(&[b".fanout" as &[u8]], Node::BlobRef(fanout_blob));
    }
    let effective_depth = depth.unwrap_or(1) as usize;
    for (id, data) in leaves {
        let hex = hex_of(*id);
        let segs = fanout_segments(&hex, effective_depth);
        let seg_refs: Vec<&[u8]> = segs.iter().map(|s| s.as_slice()).collect();
        root.insert(&seg_refs, Node::TreeRef(*data));
    }
    write_tree(repo, &root)
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
