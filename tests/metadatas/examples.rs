//! Example-based tests.

use super::helpers::*;
use git_metadata::{Error, MetadataRepository};

#[test]
fn empty_fanout_tree_returns_empty_vec() {
    let (_dir, repo) = init_repo();
    let root = write_tree(&repo, &Node::dir());
    set_ref(&repo, root);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert!(got.is_empty());
}

#[test]
fn single_leaf_default_depth() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    let root = write_fanout(&repo, None, &[(blob_id, data)]);
    set_ref(&repo, root);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}

#[test]
fn single_leaf_with_explicit_fanout_depth_3() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    let root = write_fanout(&repo, Some(3), &[(blob_id, data)]);
    set_ref(&repo, root);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}

#[test]
fn max_depth_19() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    let root = write_fanout(&repo, Some(19), &[(blob_id, data)]);
    set_ref(&repo, root);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}

#[test]
fn missing_ref_returns_reference_error() {
    let (_dir, repo) = init_repo();
    let err = repo
        .metadatas(Some("refs/metadatas/does-not-exist"))
        .expect_err("must error on missing ref");
    assert!(matches!(err, Error::Reference(_)), "got {err:?}");
}

#[test]
fn data_tree_with_non_hex_children_does_not_pollute_results() {
    // The data tree is itself walked by the breadthfirst traversal. Children
    // inside it must not be mistaken for fanout leaves.
    let (_dir, repo) = init_repo();
    let inner_blob = blob(&repo, b"inside");

    let mut data_node = Node::dir();
    data_node.insert(&[b"file.txt" as &[u8]], Node::BlobRef(inner_blob));
    let data = write_tree(&repo, &data_node);

    let blob_id = blob(&repo, b"outer");
    let root = write_fanout(&repo, None, &[(blob_id, data)]);
    set_ref(&repo, root);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}
