//! Integration tests for `Executor` methods: `remove`, `stale`, `copy`,
//! `prune`, `read_blob_at`, and `merge`.

use git_metadata::exe::Executor;
use git_metadata::{Error, MetadataRepository};
use gix::objs::tree::EntryKind;
use rstest::rstest;

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn init_executor() -> (tempfile::TempDir, Executor) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = gix::init(dir.path()).expect("init repo");
    // Append user identity to the repo's local config so `Executor::committer()`
    // succeeds.  We use `gix::init` only to locate `.git/config`; the handle is
    // dropped so that `Executor::open` gets a fresh read of the updated file.
    let config_path = repo.path().join("config");
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&config_path)
        .expect("open git config");
    writeln!(f, "\n[user]\n\tname = Tester\n\temail = tester@example.com").expect("write config");
    drop(repo);
    let exe = Executor::open(Some(dir.path())).expect("open executor");
    (dir, exe)
}

fn blob(exe: &Executor, data: &[u8]) -> gix::ObjectId {
    exe.repo().write_blob(data).expect("write blob").detach()
}

/// Construct an ObjectId from a 40-char hex literal without writing any object.
/// The hex is chosen to be astronomically unlikely to collide with a real object
/// in an otherwise-empty test repo.
fn fake_oid(hex: &str) -> gix::ObjectId {
    assert_eq!(hex.len(), 40);
    gix::ObjectId::from_hex(hex.as_bytes()).expect("valid hex")
}

/// Collect and sort all leaf paths in `target`'s metadata tree.
fn file_paths(exe: &Executor, target: gix::ObjectId) -> Vec<String> {
    let tree_id = exe
        .repo()
        .find_metadata(Some(TEST_REF), target)
        .expect("find metadata");
    let tree = exe.repo().find_tree(tree_id).expect("find tree");
    let mut paths: Vec<String> = tree
        .traverse()
        .breadthfirst
        .files()
        .expect("traverse")
        .into_iter()
        .filter(|e| !e.mode.is_tree())
        .map(|e| e.filepath.to_string())
        .collect();
    paths.sort();
    paths
}

const TEST_REF: &str = "refs/metadatas/test";

// Files inserted before every remove glob test.
const FIXTURE_FILES: &[&str] = &["lib.rs", "main.rs", "readme.md", "src/util.rs"];

// ──────────────────────────────────────────────────────────────────────────────
// remove
// ──────────────────────────────────────────────────────────────────────────────

/// Parameterised over glob patterns and expected post-remove state.
///
/// `src/util.rs` is intentionally included to pin `NO_MATCH_SLASH_LITERAL`
/// behavior: `*.rs` must NOT remove it because `*` cannot cross `/`.
#[rstest]
#[case::exact_name(&["lib.rs"], &["main.rs", "readme.md", "src/util.rs"], true)]
#[case::top_level_glob(&["*.rs"], &["readme.md", "src/util.rs"], true)]
#[case::multi_pattern(&["*.rs", "*.md"], &["src/util.rs"], true)]
#[case::no_match(&["no-such.*"], &["lib.rs", "main.rs", "readme.md", "src/util.rs"], false)]
#[case::empty_patterns(&[], &["lib.rs", "main.rs", "readme.md", "src/util.rs"], false)]
fn remove_glob_behavior(
    #[case] patterns: &[&str],
    #[case] expected_remaining: &[&str],
    #[case] expect_commit: bool,
) {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"target");
    let d = blob(&exe, b"data");

    for name in FIXTURE_FILES {
        exe.upsert(target, name, EntryKind::Blob, d, false, None, None, 1)
            .expect("upsert fixture");
    }

    let result = exe.remove(target, patterns, None, None).expect("remove");
    assert_eq!(result.is_some(), expect_commit);

    let mut expected: Vec<&str> = expected_remaining.to_vec();
    expected.sort();
    assert_eq!(file_paths(&exe, target), expected);
}

#[test]
fn remove_last_file_deletes_metadata_entry_and_returns_none() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"target");
    let data = blob(&exe, b"payload");

    exe.upsert(
        target,
        "only.txt",
        EntryKind::Blob,
        data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert");

    let result = exe
        .remove(target, &["only.txt"], None, None)
        .expect("remove");
    assert!(result.is_none(), "empty tree → None");

    assert!(
        matches!(
            exe.repo().find_metadata(Some(TEST_REF), target),
            Err(Error::NotFound(_))
        ),
        "metadata entry should be deleted"
    );
}

#[test]
fn remove_invalid_glob_errors() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"target");
    let data = blob(&exe, b"payload");
    exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert");

    // The empty byte sequence is not a valid glob in gix.
    let err = exe
        .remove(target, &[""], None, None)
        .expect_err("invalid glob should error");
    assert!(
        format!("{err}").contains("invalid glob"),
        "unexpected error: {err}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// stale
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn stale_returns_empty_when_ref_absent() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    assert!(exe.stale().expect("stale").is_empty());
}

#[test]
fn stale_returns_empty_when_all_targets_exist() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"real target");
    let data = blob(&exe, b"metadata");

    exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert");

    let got = exe.stale().expect("stale");
    assert!(got.is_empty(), "real object should not be stale: {got:?}");
}

#[test]
fn stale_returns_multiple_phantoms_not_real_targets() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);

    let p1 = fake_oid("deadbeef0000000000000000000000000000dead");
    let p2 = fake_oid("cafebabe0000000000000000000000000000cafe");
    let real = blob(&exe, b"real");
    let data = blob(&exe, b"meta");

    for target in [p1, p2, real] {
        exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
            .expect("upsert");
    }

    let mut got = exe.stale().expect("stale");
    got.sort();
    let mut expected = vec![p1, p2];
    expected.sort();
    assert_eq!(got, expected);
}

// ──────────────────────────────────────────────────────────────────────────────
// copy
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn copy_creates_metadata_at_destination() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let src = blob(&exe, b"src");
    let dst = blob(&exe, b"dst");
    let data = blob(&exe, b"payload");

    exe.upsert(
        src,
        "readme.md",
        EntryKind::Blob,
        data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert src");
    exe.copy(src, dst, false).expect("copy");

    assert_eq!(file_paths(&exe, dst), ["readme.md"]);
}

#[test]
fn copy_without_force_errors_when_destination_has_metadata() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let src = blob(&exe, b"src");
    let dst = blob(&exe, b"dst");
    let data = blob(&exe, b"payload");

    exe.upsert(src, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert src");
    exe.upsert(dst, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert dst");

    let err = exe
        .copy(src, dst, false)
        .expect_err("should fail without force");
    assert!(
        matches!(err.downcast_ref::<Error>(), Some(Error::AlreadyExists(_))),
        "expected AlreadyExists, got {err:?}"
    );
}

#[test]
fn copy_from_missing_source_errors_not_found() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let src = fake_oid("deadbeef0000000000000000000000000000dead");
    let dst = blob(&exe, b"dst");

    // Populate the ref so it exists, but `src` has no entry in it.
    let other = blob(&exe, b"other");
    let data = blob(&exe, b"meta");
    exe.upsert(other, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert other");

    let err = exe
        .copy(src, dst, false)
        .expect_err("missing src should error");
    assert!(
        matches!(err.downcast_ref::<Error>(), Some(Error::NotFound(_))),
        "expected NotFound, got {err:?}"
    );
}

#[test]
fn copy_with_force_overwrites_destination() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let src = blob(&exe, b"src");
    let dst = blob(&exe, b"dst");
    let data_src = blob(&exe, b"src-meta");
    let data_dst = blob(&exe, b"dst-meta");

    exe.upsert(src, "f", EntryKind::Blob, data_src, false, None, None, 1)
        .expect("upsert src");
    exe.upsert(dst, "f", EntryKind::Blob, data_dst, false, None, None, 1)
        .expect("upsert dst");
    exe.copy(src, dst, true).expect("copy with force");

    let dst_tree = exe
        .repo()
        .find_metadata(Some(TEST_REF), dst)
        .expect("dst tree");
    let src_tree = exe
        .repo()
        .find_metadata(Some(TEST_REF), src)
        .expect("src tree");
    assert_eq!(dst_tree, src_tree, "dst should have src's metadata tree");
}

// ──────────────────────────────────────────────────────────────────────────────
// prune
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn prune_dry_run_prints_stale_targets_without_removing() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let p1 = fake_oid("cafebabe0000000000000000000000000000cafe");
    let p2 = fake_oid("deadbeef0000000000000000000000000000dead");
    let data = blob(&exe, b"meta");

    for p in [p1, p2] {
        exe.upsert(p, "f", EntryKind::Blob, data, false, None, None, 1)
            .expect("upsert phantom");
    }

    let mut pruned = exe.prune(true).expect("prune dry-run");
    assert_eq!(pruned.len(), 2);

    pruned.sort();
    let mut expected = vec![p1, p2];
    expected.sort();
    assert_eq!(pruned, expected);

    // Dry-run must not remove anything.
    let mut still_stale = exe.stale().expect("stale after dry-run");
    still_stale.sort();
    let mut expected = vec![p1, p2];
    expected.sort();
    assert_eq!(still_stale, expected);
}

#[test]
fn prune_removes_stale_entries_and_returns_count() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let p1 = fake_oid("deadbeef1234567890abcdef1234567890abcdef");
    let p2 = fake_oid("cafebabe1234567890abcdef1234567890abcdef");
    let real = blob(&exe, b"real");
    let data = blob(&exe, b"meta");

    for target in [p1, p2, real] {
        exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
            .expect("upsert");
    }

    let pruned = exe.prune(false).expect("prune");
    assert_eq!(pruned.len(), 2);

    assert!(
        exe.stale().expect("stale after prune").is_empty(),
        "all stale entries should be removed"
    );
    exe.repo()
        .find_metadata(Some(TEST_REF), real)
        .expect("real target metadata should survive prune");
}

// ──────────────────────────────────────────────────────────────────────────────
// read_blob_at
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn read_blob_at_returns_content() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"target");
    let data = blob(&exe, b"hello world");
    exe.upsert(
        target,
        "note.md",
        EntryKind::Blob,
        data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert");

    let got = exe.read_blob_at(target, "note.md").expect("read");
    assert_eq!(got, b"hello world");
}

#[test]
fn read_blob_at_errors_when_path_absent() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"target");
    let data = blob(&exe, b"x");
    exe.upsert(
        target,
        "present",
        EntryKind::Blob,
        data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert");

    let err = exe
        .read_blob_at(target, "missing")
        .expect_err("should error");
    assert!(format!("{err}").contains("no entry"), "got: {err}");
}

#[test]
fn read_blob_at_errors_when_target_has_no_metadata() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let other = blob(&exe, b"other");
    let data = blob(&exe, b"d");
    exe.upsert(other, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert other");

    let target = blob(&exe, b"target");
    let err = exe.read_blob_at(target, "p").expect_err("should error");
    assert!(
        matches!(err.downcast_ref::<Error>(), Some(Error::NotFound(_))),
        "expected NotFound, got {err:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// merge
// ──────────────────────────────────────────────────────────────────────────────

const SIDE_REF: &str = "refs/metadatas/side";

fn tip(exe: &Executor, name: &str) -> gix::ObjectId {
    exe.repo()
        .find_reference(name)
        .expect("find ref")
        .peel_to_id()
        .expect("peel")
        .detach()
}

fn point_ref_at(exe: &Executor, name: &str, oid: gix::ObjectId) {
    exe.repo()
        .reference(
            name,
            oid,
            gix::refs::transaction::PreviousValue::Any,
            "test",
        )
        .expect("set ref");
}

#[test]
fn merge_creates_ref_when_dest_absent() {
    let (_dir, exe) = init_executor();
    let exe_side = Executor::open(Some(exe.repo().path().parent().unwrap()))
        .expect("open")
        .with_ref(SIDE_REF);
    let target = blob(&exe_side, b"t");
    let data = blob(&exe_side, b"d");
    exe_side
        .upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert side");
    let source_tip = tip(&exe_side, SIDE_REF);

    let exe = exe.with_ref(TEST_REF);
    let new_tip = exe.merge(SIDE_REF, None).expect("merge");
    assert_eq!(new_tip, source_tip);
    assert_eq!(tip(&exe, TEST_REF), source_tip);
}

#[test]
fn merge_same_tip_is_noop() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"t");
    let data = blob(&exe, b"d");
    exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert");
    let dest_tip = tip(&exe, TEST_REF);
    point_ref_at(&exe, SIDE_REF, dest_tip);

    let new_tip = exe.merge(SIDE_REF, None).expect("merge");
    assert_eq!(new_tip, dest_tip);
}

#[test]
fn merge_fast_forward_when_base_equals_dest() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"t");
    let data = blob(&exe, b"d");
    exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert base");
    let base = tip(&exe, TEST_REF);
    point_ref_at(&exe, SIDE_REF, base);

    // Advance SIDE_REF past base by upserting through a side executor.
    let exe_side = Executor::open(Some(exe.repo().path().parent().unwrap()))
        .expect("open")
        .with_ref(SIDE_REF);
    let data2 = blob(&exe_side, b"d2");
    exe_side
        .upsert(target, "g", EntryKind::Blob, data2, false, None, None, 1)
        .expect("upsert side");
    let side_tip = tip(&exe_side, SIDE_REF);
    assert_ne!(side_tip, base);

    let new_tip = exe.merge(SIDE_REF, None).expect("merge");
    assert_eq!(new_tip, side_tip);
    assert_eq!(tip(&exe, TEST_REF), side_tip);
}

#[test]
fn merge_already_up_to_date_when_source_is_ancestor() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"t");
    let data = blob(&exe, b"d");
    exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert base");
    let base = tip(&exe, TEST_REF);
    point_ref_at(&exe, SIDE_REF, base);

    let data2 = blob(&exe, b"d2");
    exe.upsert(target, "g", EntryKind::Blob, data2, false, None, None, 1)
        .expect("upsert dest");
    let dest_tip = tip(&exe, TEST_REF);

    let new_tip = exe.merge(SIDE_REF, None).expect("merge");
    assert_eq!(new_tip, dest_tip);
    assert_eq!(tip(&exe, TEST_REF), dest_tip);
}

#[test]
fn merge_three_way_disjoint_paths_succeeds() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"t");
    let base_data = blob(&exe, b"base");
    exe.upsert(
        target,
        "base.txt",
        EntryKind::Blob,
        base_data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert base");
    let base = tip(&exe, TEST_REF);
    point_ref_at(&exe, SIDE_REF, base);

    let exe_side = Executor::open(Some(exe.repo().path().parent().unwrap()))
        .expect("open")
        .with_ref(SIDE_REF);
    let theirs_data = blob(&exe_side, b"theirs");
    exe_side
        .upsert(
            target,
            "theirs.txt",
            EntryKind::Blob,
            theirs_data,
            false,
            None,
            None,
            1,
        )
        .expect("upsert theirs");

    let ours_data = blob(&exe, b"ours");
    exe.upsert(
        target,
        "ours.txt",
        EntryKind::Blob,
        ours_data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert ours");
    let dest_tip = tip(&exe, TEST_REF);

    let new_tip = exe.merge(SIDE_REF, None).expect("merge");
    assert_ne!(new_tip, dest_tip, "should produce a merge commit");
    let mut paths = file_paths(&exe, target);
    paths.retain(|p| !p.ends_with(".fanout"));
    assert_eq!(paths, ["base.txt", "ours.txt", "theirs.txt"]);
}

#[test]
fn merge_conflict_aborts() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"t");
    let base_data = blob(&exe, b"base\n");
    exe.upsert(
        target,
        "conflict.txt",
        EntryKind::Blob,
        base_data,
        false,
        None,
        None,
        1,
    )
    .expect("upsert base");
    let base = tip(&exe, TEST_REF);
    point_ref_at(&exe, SIDE_REF, base);

    let exe_side = Executor::open(Some(exe.repo().path().parent().unwrap()))
        .expect("open")
        .with_ref(SIDE_REF);
    let theirs_data = blob(&exe_side, b"theirs\n");
    exe_side
        .upsert(
            target,
            "conflict.txt",
            EntryKind::Blob,
            theirs_data,
            true,
            None,
            None,
            1,
        )
        .expect("upsert theirs");

    let ours_data = blob(&exe, b"ours\n");
    exe.upsert(
        target,
        "conflict.txt",
        EntryKind::Blob,
        ours_data,
        true,
        None,
        None,
        1,
    )
    .expect("upsert ours");
    let dest_tip_before = tip(&exe, TEST_REF);

    let err = exe.merge(SIDE_REF, None).expect_err("should conflict");
    assert!(
        format!("{err}").contains("conflict"),
        "expected conflict error, got: {err}"
    );
    assert_eq!(
        tip(&exe, TEST_REF),
        dest_tip_before,
        "dest ref must be unchanged on conflict",
    );
}

#[test]
fn merge_source_not_a_commit_errors() {
    let (_dir, exe) = init_executor();
    let exe = exe.with_ref(TEST_REF);
    let target = blob(&exe, b"t");
    let data = blob(&exe, b"d");
    exe.upsert(target, "f", EntryKind::Blob, data, false, None, None, 1)
        .expect("upsert");

    let bad = data.to_string();
    let err = exe.merge(&bad, None).expect_err("should error");
    assert!(format!("{err}").contains("expected a commit"), "got: {err}");
}
