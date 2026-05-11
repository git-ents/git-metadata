//! Tests for `.fanout` parsing and validation.

use super::helpers::*;
use git_metadata::{Error, MetadataRepository};
use rstest::rstest;

fn write_root_with_fanout_blob_content(repo: &gix::Repository, content: &[u8]) -> gix::ObjectId {
    let fanout = blob(repo, content);
    let mut root = Node::dir();
    root.insert(&[b".fanout" as &[u8]], Node::BlobRef(fanout));
    write_tree(repo, &root)
}

#[rstest]
#[case::zero(b"0")]
#[case::twenty(b"20")]
#[case::ninety_nine(b"99")]
#[case::non_numeric(b"abc")]
#[case::empty(b"")]
#[case::negative(b"-1")]
fn invalid_fanout_depth_errors(#[case] content: &[u8]) {
    let (_dir, repo) = init_repo();
    let root = write_root_with_fanout_blob_content(&repo, content);
    set_ref(&repo, root);

    let err = repo.metadatas(Some(FANOUT_REF)).expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutDepth { .. }),
        "got {err:?}"
    );
}

#[test]
fn fanout_blob_with_trailing_newline_is_accepted() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    let fanout = blob(&repo, b"2\n");
    let mut root = Node::dir();
    root.insert(&[b".fanout" as &[u8]], Node::BlobRef(fanout));
    let hex = hex_of(blob_id);
    let segs = fanout_segments(&hex, 2);
    let seg_refs: Vec<&[u8]> = segs.iter().map(|s| s.as_slice()).collect();
    root.insert(&seg_refs, Node::TreeRef(data));
    let root_id = write_tree(&repo, &root);
    set_ref(&repo, root_id);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}

#[test]
fn fanout_entry_is_tree_errors() {
    let (_dir, repo) = init_repo();
    let fanout_tree = empty_tree(&repo);
    let mut root = Node::dir();
    root.insert(&[b".fanout" as &[u8]], Node::TreeRef(fanout_tree));
    let root_id = write_tree(&repo, &root);
    set_ref(&repo, root_id);

    let err = repo.metadatas(Some(FANOUT_REF)).expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutDepth { .. }),
        "got {err:?}"
    );
}
