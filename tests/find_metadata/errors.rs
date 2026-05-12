//! Error-path tests for `MetadataRepository::find_metadata`.

use super::helpers::*;
use git_metadata::{Error, MetadataRepository};

#[test]
fn intermediate_segment_is_blob_yields_conflict() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let hex = hex_of(target);
    let head: gix::bstr::BString = hex[0..2].into();

    let squatter = blob(&repo, b"squat");
    let root = write_tree(
        &repo,
        vec![
            (vec![".fanout".into()], EntryKind::Blob, blob(&repo, b"2")),
            (vec![head.clone()], EntryKind::Blob, squatter),
        ],
    );
    set_ref(&repo, root);

    let err = repo
        .find_metadata(Some(FANOUT_REF), target)
        .expect_err("must error");
    assert!(
        matches!(&err, Error::FanoutPathConflict(p) if p == &head),
        "got {err:?}"
    );
}

#[test]
fn leaf_segment_is_blob_yields_conflict() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let hex = hex_of(target);

    // Depth 1: prefix is hex[0..2], leaf name is hex[2..]. Plant a blob at the
    // leaf path instead of a tree.
    let prefix: gix::bstr::BString = hex[0..2].into();
    let leaf: gix::bstr::BString = hex[2..].into();
    let squatter = blob(&repo, b"leaf-squat");
    let root = write_tree(
        &repo,
        vec![(vec![prefix, leaf.clone()], EntryKind::Blob, squatter)],
    );
    set_ref(&repo, root);

    let err = repo
        .find_metadata(Some(FANOUT_REF), target)
        .expect_err("must error");
    assert!(
        matches!(&err, Error::FanoutPathConflict(p) if p == &leaf),
        "got {err:?}"
    );
}

#[test]
fn invalid_fanout_depth_propagates() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let root = write_tree(
        &repo,
        vec![(vec![".fanout".into()], EntryKind::Blob, blob(&repo, b"99"))],
    );
    set_ref(&repo, root);

    let err = repo
        .find_metadata(Some(FANOUT_REF), target)
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutDepth { .. }),
        "got {err:?}"
    );
}
