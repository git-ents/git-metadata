//! Table-driven tests for `MetadataRepository::metadata_ref_fanout`.

use super::helpers::*;
use git_metadata::{DEFAULT_FANOUT, Error, MetadataRepository};
use gix::objs::tree::{Entry, EntryKind};
use rstest::rstest;

fn write_root_with_fanout_entry(repo: &gix::Repository, entry: Entry) -> gix::ObjectId {
    let tree = gix::objs::Tree {
        entries: vec![entry],
    };
    repo.write_object(&tree).expect("write tree").detach()
}

fn write_root_with_fanout_blob(repo: &gix::Repository, content: &[u8]) -> gix::ObjectId {
    let oid = blob(repo, content);
    write_root_with_fanout_entry(
        repo,
        Entry {
            mode: EntryKind::Blob.into(),
            filename: ".fanout".into(),
            oid,
        },
    )
}

#[test]
fn absent_fanout_returns_default() {
    let (_dir, repo) = init_repo();
    let root = empty_tree(&repo);
    set_ref(&repo, root);
    let depth = repo
        .metadata_ref_fanout(Some(FANOUT_REF))
        .expect("fanout depth");
    assert_eq!(depth, DEFAULT_FANOUT);
}

#[test]
fn none_uses_default_ref() {
    let (_dir, repo) = init_repo();
    let root = empty_tree(&repo);
    let default_ref = repo.metadata_default_ref().expect("default ref");
    set_ref_named(&repo, &default_ref, root);
    let depth = repo.metadata_ref_fanout(None).expect("fanout depth");
    assert_eq!(depth, DEFAULT_FANOUT);
}

#[test]
fn missing_ref_errors() {
    let (_dir, repo) = init_repo();
    let err = repo
        .metadata_ref_fanout(Some("refs/metadatas/does-not-exist"))
        .expect_err("must error");
    assert!(matches!(err, Error::Reference(_)), "got {err:?}");
}

#[rstest]
#[case::one(b"1", 1)]
#[case::two(b"2", 2)]
#[case::nineteen(b"19", 19)]
#[case::trailing_newline(b"2\n", 2)]
#[case::leading_whitespace(b"  3", 3)]
#[case::surrounding_whitespace(b" 4\n", 4)]
fn valid_depth(#[case] content: &[u8], #[case] want: u8) {
    let (_dir, repo) = init_repo();
    let root = write_root_with_fanout_blob(&repo, content);
    set_ref(&repo, root);
    let depth = repo
        .metadata_ref_fanout(Some(FANOUT_REF))
        .expect("fanout depth");
    assert_eq!(depth, want);
}

#[rstest]
#[case::zero(b"0")]
#[case::twenty(b"20")]
#[case::ninety_nine(b"99")]
#[case::overflow_u8(b"256")]
#[case::non_numeric(b"abc")]
#[case::empty(b"")]
#[case::negative(b"-1")]
#[case::interior_whitespace(b"1 9")]
#[case::trailing_junk(b"2x")]
#[case::non_utf8(b"\xff\xfe")]
fn invalid_depth(#[case] content: &[u8]) {
    let (_dir, repo) = init_repo();
    let root = write_root_with_fanout_blob(&repo, content);
    set_ref(&repo, root);
    let err = repo
        .metadata_ref_fanout(Some(FANOUT_REF))
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutDepth { .. }),
        "got {err:?}"
    );
}

#[rstest]
#[case::tree(EntryKind::Tree)]
#[case::link(EntryKind::Link)]
#[case::commit(EntryKind::Commit)]
fn non_blob_fanout_errors(#[case] kind: EntryKind) {
    let (_dir, repo) = init_repo();
    // For Tree we point at a real empty tree; for Link/Commit we use an
    // arbitrary sha1 — `is_blob()` rejects before any lookup.
    let oid = match kind {
        EntryKind::Tree => empty_tree(&repo),
        _ => gix::ObjectId::from_hex(b"0123456789abcdef0123456789abcdef01234567").unwrap(),
    };
    let root = write_root_with_fanout_entry(
        &repo,
        Entry {
            mode: kind.into(),
            filename: ".fanout".into(),
            oid,
        },
    );
    set_ref(&repo, root);
    let err = repo
        .metadata_ref_fanout(Some(FANOUT_REF))
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutType { .. }),
        "got {err:?}"
    );
}

#[test]
fn executable_blob_fanout_is_accepted() {
    let (_dir, repo) = init_repo();
    let oid = blob(&repo, b"2");
    let root = write_root_with_fanout_entry(
        &repo,
        Entry {
            mode: EntryKind::BlobExecutable.into(),
            filename: ".fanout".into(),
            oid,
        },
    );
    set_ref(&repo, root);
    let depth = repo
        .metadata_ref_fanout(Some(FANOUT_REF))
        .expect("fanout depth");
    assert_eq!(depth, 2);
}
