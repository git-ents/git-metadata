//! Example-based tests for `MetadataRepository::find_metadata`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};

#[test]
fn returns_data_oid_after_write() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), Some(FANOUT_REF), target, &data, false)
        .expect("write");

    let got = repo.find_metadata(Some(FANOUT_REF), target).expect("find");
    assert_eq!(got, data);
}

#[test]
fn none_metadatas_ref_uses_default() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), None, target, &data, false)
        .expect("write");

    let got = repo.find_metadata(None, target).expect("find");
    assert_eq!(got, data);
}

#[test]
fn missing_leaf_returns_not_found() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let other = blob(&repo, b"other");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), Some(FANOUT_REF), target, &data, false)
        .expect("write");

    let err = repo
        .find_metadata(Some(FANOUT_REF), other)
        .expect_err("must error");
    assert!(
        matches!(err, Error::NotFound(o) if o == other),
        "got {err:?}"
    );
}

#[test]
fn missing_ref_returns_reference_error() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");

    let err = repo
        .find_metadata(Some("refs/metadatas/does-not-exist"), target)
        .expect_err("must error");
    assert!(matches!(err, Error::Gix(_)), "got {err:?}");
}

#[test]
fn empty_fanout_tree_returns_not_found() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let root = write_fanout(&repo, None, &[]);
    set_ref(&repo, root);

    let err = repo
        .find_metadata(Some(FANOUT_REF), target)
        .expect_err("must error");
    assert!(
        matches!(err, Error::NotFound(o) if o == target),
        "got {err:?}"
    );
}
