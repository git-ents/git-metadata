//! Error-path tests for `MetadataRepository::metadata_delete`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};

#[test]
fn missing_leaf_returns_not_found() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let other = blob(&repo, b"other");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write");

    let err = repo
        .metadata_delete(other, Some(FANOUT_REF), sig(), sig(), None)
        .expect_err("must error");
    assert!(
        matches!(err, Error::NotFound(o) if o == other),
        "got {err:?}"
    );
}

#[test]
fn missing_ref_errors() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");

    let err = repo
        .metadata_delete(
            target,
            Some("refs/metadatas/does-not-exist"),
            sig(),
            sig(),
            None,
        )
        .expect_err("must error");
    assert!(matches!(err, Error::Gix(_)), "got {err:?}");
}

#[test]
fn intermediate_segment_is_blob_yields_not_found() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let hex = hex_of(target);
    let head: gix::bstr::BString = hex[0..2].into();

    let squatter = blob(&repo, b"squat");
    let root = write_tree(
        &repo,
        vec![
            (vec![".fanout".into()], EntryKind::Blob, blob(&repo, b"2")),
            (vec![head], EntryKind::Blob, squatter),
        ],
    );
    set_ref(&repo, root);

    let err = repo
        .metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect_err("must error");
    assert!(
        matches!(&err, Error::NotFound(t) if t == &target),
        "got {err:?}"
    );
}

#[test]
fn invalid_fanout_type_propagates() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let subtree = empty_tree(&repo);
    let root = write_tree(
        &repo,
        vec![(vec![".fanout".into()], EntryKind::Tree, subtree)],
    );
    set_ref(&repo, root);

    let err = repo
        .metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutType { .. }),
        "got {err:?}"
    );
}

#[test]
fn missing_leaf_at_crafted_id_returns_not_found() {
    let (_dir, repo) = init_repo();
    let kept = blob(&repo, b"kept");
    let data = empty_tree(&repo);
    let root = write_fanout(&repo, Some(1), &[(kept, data)]);
    set_ref(&repo, root);

    // Craft an id that shares no head byte with `kept` to avoid relying on
    // blob OID divergence.
    let kept_hex = hex_of(kept);
    let head = if &kept_hex[0..2] == b"00" {
        b"ff"
    } else {
        b"00"
    };
    let mut missing_hex = Vec::with_capacity(40);
    missing_hex.extend_from_slice(head);
    missing_hex.extend_from_slice(&kept_hex[2..]);
    let missing = gix::ObjectId::from_hex(&missing_hex).expect("hex");

    let err = repo
        .metadata_delete(missing, Some(FANOUT_REF), sig(), sig(), None)
        .expect_err("must error");
    assert!(
        matches!(err, Error::NotFound(o) if o == missing),
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
        .metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutDepth { .. }),
        "got {err:?}"
    );
}
