use crate::Error;
use crate::tree::*;
use gix::bstr::BString;
use gix::objs::tree::{Entry, EntryKind};
use proptest::prelude::*;
use rstest::rstest;

fn oid(hex: &[u8]) -> gix::ObjectId {
    gix::ObjectId::from_hex(hex).expect("valid hex")
}

fn repo() -> (tempfile::TempDir, gix::Repository) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = gix::init(dir.path()).expect("init repo");
    (dir, repo)
}

fn empty_tree(repo: &gix::Repository) -> gix::ObjectId {
    repo.write_object(gix::objs::Tree::empty())
        .expect("write empty tree")
        .detach()
}

fn entries(repo: &gix::Repository, tree_oid: gix::ObjectId) -> Vec<Entry> {
    crate::tree::decode_entries(repo, tree_oid).expect("decode entries")
}

fn path(segs: &[&str]) -> Vec<BString> {
    segs.iter().map(|s| BString::from(*s)).collect()
}

#[rstest]
#[case::depth_1(1, &["ab", "cdef0123456789abcdef0123456789abcdef01"])]
#[case::depth_2(2, &["ab", "cd", "ef0123456789abcdef0123456789abcdef01"])]
#[case::depth_3(3, &["ab", "cd", "ef", "0123456789abcdef0123456789abcdef01"])]
#[case::depth_19(19, &[
    "ab", "cd", "ef", "01", "23", "45", "67", "89", "ab", "cd",
    "ef", "01", "23", "45", "67", "89", "ab", "cd", "ef", "01",
])]
fn fanout_path_splits_hex(#[case] depth: u8, #[case] want: &[&str]) {
    let id = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let got = fanout_path(id, depth);
    let want: Vec<gix::bstr::BString> = want.iter().map(|s| gix::bstr::BString::from(*s)).collect();
    assert_eq!(got, want);
}

#[rstest]
#[case(1)]
#[case(2)]
#[case(5)]
#[case(19)]
fn fanout_path_round_trips_to_full_hex(#[case] depth: u8) {
    let id = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let path = fanout_path(id, depth);
    let mut joined = Vec::new();
    for seg in &path {
        joined.extend_from_slice(seg);
    }
    assert_eq!(joined, id.to_hex().to_string().as_bytes());
}

#[test]
fn fanout_path_segment_count_is_depth_plus_one() {
    let id = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    for d in 1u8..=19 {
        assert_eq!(fanout_path(id, d).len(), d as usize + 1);
    }
}

#[test]
fn insert_leaf_at_depth_one_creates_intermediate() {
    let (_dir, repo) = repo();
    let root = empty_tree(&repo);
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");

    let new_root = insert_leaf(
        &repo,
        root,
        &path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]),
        leaf,
        false,
        target,
    )
    .expect("insert");

    let root_entries = entries(&repo, new_root);
    assert_eq!(root_entries.len(), 1);
    assert_eq!(root_entries[0].filename, "ab");
    assert!(root_entries[0].mode.is_tree());

    let sub = entries(&repo, root_entries[0].oid);
    assert_eq!(sub.len(), 1);
    assert_eq!(sub[0].filename, "cdef0123456789abcdef0123456789abcdef01");
    assert_eq!(sub[0].oid, leaf);
    assert!(sub[0].mode.is_tree());
}

#[test]
fn insert_leaf_existing_without_force_errors() {
    let (_dir, repo) = repo();
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    let root = insert_leaf(&repo, empty_tree(&repo), &p, leaf, false, target).expect("first");
    let other = repo.write_blob(b"x").expect("blob").detach();
    let err = insert_leaf(&repo, root, &p, other, false, target).expect_err("conflict");
    assert!(matches!(err, Error::AlreadyExists(t) if t == target));
}

#[test]
fn insert_leaf_existing_with_force_replaces() {
    let (_dir, repo) = repo();
    let leaf_a = empty_tree(&repo);
    let leaf_b = repo
        .write_object(gix::objs::Tree {
            entries: vec![Entry {
                mode: EntryKind::Blob.into(),
                filename: "x".into(),
                oid: repo.write_blob(b"x").expect("blob").detach(),
            }],
        })
        .expect("write tree b")
        .detach();
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    let root = insert_leaf(&repo, empty_tree(&repo), &p, leaf_a, false, target).expect("first");
    let new_root = insert_leaf(&repo, root, &p, leaf_b, true, target).expect("force");

    let root_entries = entries(&repo, new_root);
    let sub = entries(&repo, root_entries[0].oid);
    assert_eq!(sub[0].oid, leaf_b);
}

#[test]
fn insert_leaf_into_non_tree_segment_conflicts() {
    let (_dir, repo) = repo();
    let blob = repo.write_blob(b"x").expect("blob").detach();
    let root = repo
        .write_object(gix::objs::Tree {
            entries: vec![Entry {
                mode: EntryKind::Blob.into(),
                filename: "ab".into(),
                oid: blob,
            }],
        })
        .expect("write root")
        .detach();
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let err = insert_leaf(
        &repo,
        root,
        &path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]),
        leaf,
        false,
        target,
    )
    .expect_err("conflict");
    assert!(matches!(err, Error::FanoutPathConflict(s) if s == "ab"));
}

#[test]
fn remove_leaf_prunes_empty_intermediate() {
    let (_dir, repo) = repo();
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    let root = insert_leaf(&repo, empty_tree(&repo), &p, leaf, false, target).expect("insert");
    let new_root = remove_leaf(&repo, root, &p, target).expect("remove");

    assert!(entries(&repo, new_root).is_empty());
}

#[test]
fn remove_leaf_keeps_sibling_intermediate() {
    let (_dir, repo) = repo();
    let leaf = empty_tree(&repo);
    let target_a = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let target_b = oid(b"ab0000000000000000000000000000000000000f");
    let pa = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);
    let pb = path(&["ab", "0000000000000000000000000000000000000f"]);

    let root = insert_leaf(&repo, empty_tree(&repo), &pa, leaf, false, target_a).expect("a");
    let root = insert_leaf(&repo, root, &pb, leaf, false, target_b).expect("b");
    let new_root = remove_leaf(&repo, root, &pa, target_a).expect("remove");

    let root_entries = entries(&repo, new_root);
    assert_eq!(root_entries.len(), 1);
    assert_eq!(root_entries[0].filename, "ab");
    let sub = entries(&repo, root_entries[0].oid);
    assert_eq!(sub.len(), 1);
    assert_eq!(sub[0].filename, "0000000000000000000000000000000000000f");
}

#[test]
fn remove_leaf_missing_errors_not_found() {
    let (_dir, repo) = repo();
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let err = remove_leaf(
        &repo,
        empty_tree(&repo),
        &path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]),
        target,
    )
    .expect_err("missing");
    assert!(matches!(err, Error::NotFound(t) if t == target));
}

#[test]
fn remove_leaf_non_tree_segment_conflicts() {
    let (_dir, repo) = repo();
    let blob = repo.write_blob(b"x").expect("blob").detach();
    let root = repo
        .write_object(gix::objs::Tree {
            entries: vec![Entry {
                mode: EntryKind::Blob.into(),
                filename: "ab".into(),
                oid: blob,
            }],
        })
        .expect("write root")
        .detach();
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let err = remove_leaf(
        &repo,
        root,
        &path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]),
        target,
    )
    .expect_err("conflict");
    assert!(matches!(err, Error::FanoutPathConflict(s) if s == "ab"));
}

#[test]
fn ensure_fanout_blob_writes_depth() {
    let (_dir, repo) = repo();
    let new_root = ensure_fanout_blob(&repo, empty_tree(&repo), 3).expect("ensure");
    let es = entries(&repo, new_root);
    assert_eq!(es.len(), 1);
    assert_eq!(es[0].filename, ".fanout");
    assert!(es[0].mode.is_blob());
    let blob = repo.find_blob(es[0].oid).expect("blob");
    assert_eq!(blob.data.as_slice(), b"3");
}

#[test]
fn ensure_fanout_blob_replaces_existing() {
    let (_dir, repo) = repo();
    let root = ensure_fanout_blob(&repo, empty_tree(&repo), 1).expect("first");
    let new_root = ensure_fanout_blob(&repo, root, 5).expect("replace");
    let es = entries(&repo, new_root);
    assert_eq!(es.len(), 1);
    let blob = repo.find_blob(es[0].oid).expect("blob");
    assert_eq!(blob.data.as_slice(), b"5");
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Every prefix segment is exactly 2 hex chars; the trailing leaf
    /// segment is the remaining `40 - 2 * depth` chars.
    #[test]
    fn fanout_path_prefix_segments_are_two_chars(
        hex in "[0-9a-f]{40}",
        depth in 1u8..=19,
    ) {
        let id = oid(hex.as_bytes());
        let path = fanout_path(id, depth);
        let (leaf, prefix) = path.split_last().expect("depth + 1 segments");
        for seg in prefix {
            prop_assert_eq!(seg.len(), 2);
        }
        prop_assert_eq!(leaf.len(), 40 - 2 * depth as usize);
    }
}
