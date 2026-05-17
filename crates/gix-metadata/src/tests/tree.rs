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
    let tree = repo.find_tree(tree_oid).expect("find tree");
    let decoded = tree.decode().expect("decode tree");
    decoded
        .entries
        .iter()
        .map(|e| Entry {
            mode: e.mode,
            filename: e.filename.into(),
            oid: e.oid.into(),
        })
        .collect()
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
        EntryKind::Tree,
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

    let root = insert_leaf(
        &repo,
        empty_tree(&repo),
        &p,
        leaf,
        EntryKind::Tree,
        false,
        target,
    )
    .expect("first");
    let other = repo.write_blob(b"x").expect("blob").detach();
    let err =
        insert_leaf(&repo, root, &p, other, EntryKind::Tree, false, target).expect_err("conflict");
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

    let root = insert_leaf(
        &repo,
        empty_tree(&repo),
        &p,
        leaf_a,
        EntryKind::Tree,
        false,
        target,
    )
    .expect("first");
    let new_root =
        insert_leaf(&repo, root, &p, leaf_b, EntryKind::Tree, true, target).expect("force");

    let root_entries = entries(&repo, new_root);
    let sub = entries(&repo, root_entries[0].oid);
    assert_eq!(sub[0].oid, leaf_b);
}

#[test]
fn remove_leaf_prunes_empty_intermediate() {
    let (_dir, repo) = repo();
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    let root = insert_leaf(
        &repo,
        empty_tree(&repo),
        &p,
        leaf,
        EntryKind::Tree,
        false,
        target,
    )
    .expect("insert");
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

    let root = insert_leaf(
        &repo,
        empty_tree(&repo),
        &pa,
        leaf,
        EntryKind::Tree,
        false,
        target_a,
    )
    .expect("a");
    let root = insert_leaf(&repo, root, &pb, leaf, EntryKind::Tree, false, target_b).expect("b");
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
fn insert_leaf_blob_at_intermediate_is_overwritten() {
    let (_dir, repo) = repo();
    let squatter = repo.write_blob(b"squat").expect("blob").detach();
    let root = repo
        .write_object(gix::objs::Tree {
            entries: vec![Entry {
                mode: EntryKind::Blob.into(),
                filename: "ab".into(),
                oid: squatter,
            }],
        })
        .expect("write root")
        .detach();
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    // Editor converts the blob squatting at the intermediate to a tree.
    let new_root =
        insert_leaf(&repo, root, &p, leaf, EntryKind::Tree, false, target).expect("insert");

    let root_entries = entries(&repo, new_root);
    assert_eq!(root_entries.len(), 1);
    assert!(root_entries[0].mode.is_tree());
    let sub = entries(&repo, root_entries[0].oid);
    assert_eq!(sub.len(), 1);
    assert_eq!(sub[0].oid, leaf);
}

#[test]
fn remove_leaf_missing_when_intermediate_is_non_tree() {
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
    .expect_err("not found");
    assert!(matches!(err, Error::NotFound(t) if t == target));
}

#[test]
fn validate_clean_tree_passes() {
    let (_dir, repo) = repo();
    let leaf = empty_tree(&repo);
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let root = insert_leaf(
        &repo,
        empty_tree(&repo),
        &path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]),
        leaf,
        EntryKind::Tree,
        false,
        target,
    )
    .expect("insert");
    validate_fanout_tree(&repo, root, 1).expect("valid");
}

#[test]
fn validate_blob_at_intermediate_fails() {
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
    let err = validate_fanout_tree(&repo, root, 1).expect_err("conflict");
    assert!(matches!(err, Error::FanoutPathConflict(s) if s == "ab"));
}

#[test]
fn validate_fanout_blob_skipped_at_root() {
    let (_dir, repo) = repo();
    let root = ensure_fanout_blob(&repo, empty_tree(&repo), 2).expect("fanout");
    validate_fanout_tree(&repo, root, 2).expect("valid — .fanout blob at root is not an error");
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
fn ensure_fanout_blob_same_depth_is_noop() {
    let (_dir, repo) = repo();
    let root = ensure_fanout_blob(&repo, empty_tree(&repo), 3).expect("first");
    let same = ensure_fanout_blob(&repo, root, 3).expect("noop");
    assert_eq!(same, root);
}

#[test]
fn ensure_fanout_blob_depth_conflict_errors() {
    let (_dir, repo) = repo();
    let root = ensure_fanout_blob(&repo, empty_tree(&repo), 1).expect("first");
    let err = ensure_fanout_blob(&repo, root, 5).expect_err("conflict");
    assert!(
        matches!(
            err,
            Error::FanoutDepthConflict {
                existing: 1,
                requested: 5
            }
        ),
        "got {err:?}"
    );
}

#[rstest]
#[case::tree(EntryKind::Tree)]
#[case::link(EntryKind::Link)]
#[case::commit(EntryKind::Commit)]
fn ensure_fanout_blob_non_blob_fanout_errors(#[case] kind: EntryKind) {
    let (_dir, repo) = repo();
    let inner = empty_tree(&repo);
    let root = repo
        .write_object(gix::objs::Tree {
            entries: vec![Entry {
                mode: kind.into(),
                filename: ".fanout".into(),
                oid: inner,
            }],
        })
        .expect("write root")
        .detach();
    let err = ensure_fanout_blob(&repo, root, 2).expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutType { .. }),
        "got {err:?}"
    );
}

#[rstest]
#[case::blob(EntryKind::Blob)]
#[case::blob_executable(EntryKind::BlobExecutable)]
#[case::link(EntryKind::Link)]
#[case::commit(EntryKind::Commit)]
fn insert_leaf_non_tree_kind(#[case] kind: EntryKind) {
    let (_dir, repo) = repo();
    let leaf = repo.write_blob(b"x").expect("blob").detach();
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    let new_root =
        insert_leaf(&repo, empty_tree(&repo), &p, leaf, kind, false, target).expect("insert");

    let root_entries = entries(&repo, new_root);
    assert_eq!(root_entries.len(), 1);
    assert!(root_entries[0].mode.is_tree(), "intermediate must be tree");
    let sub = entries(&repo, root_entries[0].oid);
    assert_eq!(sub.len(), 1);
    assert_eq!(sub[0].mode, kind.into());
    assert_eq!(sub[0].oid, leaf);
}

#[test]
fn insert_leaf_force_changes_kind() {
    let (_dir, repo) = repo();
    let leaf_a = empty_tree(&repo);
    let leaf_b = repo.write_blob(b"x").expect("blob").detach();
    let target = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let p = path(&["ab", "cdef0123456789abcdef0123456789abcdef01"]);

    let root = insert_leaf(
        &repo,
        empty_tree(&repo),
        &p,
        leaf_a,
        EntryKind::Tree,
        false,
        target,
    )
    .expect("first");
    let new_root =
        insert_leaf(&repo, root, &p, leaf_b, EntryKind::Blob, true, target).expect("force");

    let sub = entries(&repo, entries(&repo, new_root)[0].oid);
    assert_eq!(sub[0].mode, EntryKind::Blob.into());
    assert_eq!(sub[0].oid, leaf_b);
}

fn arb_entry_kind() -> impl Strategy<Value = EntryKind> {
    prop_oneof![
        Just(EntryKind::Blob),
        Just(EntryKind::BlobExecutable),
        Just(EntryKind::Link),
        Just(EntryKind::Commit),
        Just(EntryKind::Tree),
    ]
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

    /// Leaf entry mode always matches the requested kind; every intermediate
    /// segment is always a tree regardless of kind.
    #[test]
    fn insert_leaf_mode_matches_kind_and_intermediates_are_trees(
        hex in "[0-9a-f]{40}",
        depth in 1u8..=5u8,
        kind in arb_entry_kind(),
    ) {
        let (_dir, repo) = repo();
        let target = oid(hex.as_bytes());
        let p = fanout_path(target, depth);
        let leaf_oid = repo.write_blob(b"x").expect("blob").detach();

        let new_root =
            insert_leaf(&repo, empty_tree(&repo), &p, leaf_oid, kind, false, target)
                .expect("insert");

        let (leaf_seg, prefix_segs) = p.split_last().expect("non-empty");
        let mut cur = entries(&repo, new_root);
        for seg in prefix_segs {
            let e = cur.iter().find(|e| e.filename == *seg).expect("seg");
            prop_assert!(e.mode.is_tree(), "intermediate {:?} must be tree", seg);
            cur = entries(&repo, e.oid);
        }
        let leaf_entry = cur.iter().find(|e| e.filename == *leaf_seg).expect("leaf");
        prop_assert_eq!(leaf_entry.mode, kind.into());
    }
}
