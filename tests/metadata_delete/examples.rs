//! Example-based tests for `MetadataRepository::metadata_delete`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};

#[test]
fn deletes_only_leaf_and_returns_empty_listing() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write");
    repo.metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let err = repo
        .find_metadata(Some(FANOUT_REF), target)
        .expect_err("must error");
    assert!(
        matches!(err, Error::NotFound(o) if o == target),
        "got {err:?}"
    );

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert!(got.is_empty());
}

#[test]
fn delete_advances_ref_with_prior_commit_as_parent() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    let c1 = repo
        .metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write");
    repo.metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let tip = repo
        .find_reference(FANOUT_REF)
        .unwrap()
        .peel_to_id()
        .unwrap()
        .detach();
    assert_ne!(tip, c1);
    let commit = repo.find_commit(tip).expect("find commit");
    let parents: Vec<_> = commit.parent_ids().map(|id| id.detach()).collect();
    assert_eq!(parents, vec![c1]);
}

#[test]
fn delete_preserves_sibling_leaves() {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let keep = blob(&repo, b"keep");
    let drop = blob(&repo, b"drop");

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), keep, &data, false)
        .expect("write keep");
    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), drop, &data, false)
        .expect("write drop");
    repo.metadata_delete(drop, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(keep, data)]));
}

#[test]
fn delete_preserves_fanout_blob() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write");
    repo.metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let tree = repo
        .find_reference(FANOUT_REF)
        .unwrap()
        .peel_to_tree()
        .unwrap();
    let entry = tree.find_entry(".fanout").expect(".fanout present");
    assert!(entry.mode().is_blob());
}

#[test]
fn none_metadatas_ref_uses_default() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    repo.metadata(sig(), sig(), None, None, target, &data, false)
        .expect("write");
    repo.metadata_delete(target, None, sig(), sig(), None)
        .expect("delete");

    let default_ref = repo.metadata_default_ref().expect("default");
    let got = repo.metadatas(Some(&default_ref)).expect("metadatas");
    assert!(got.is_empty());
}

#[test]
fn delete_preserves_unrelated_root_entries() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);
    let extra_blob = blob(&repo, b"README contents");

    let hex = hex_of(target);
    let head: gix::bstr::BString = hex[0..2].into();
    let tail: gix::bstr::BString = hex[2..].into();
    let root = write_tree(
        &repo,
        vec![
            (vec![".fanout".into()], EntryKind::Blob, blob(&repo, b"1")),
            (vec!["README".into()], EntryKind::Blob, extra_blob),
            (vec![head, tail], EntryKind::Tree, data),
        ],
    );
    set_ref(&repo, root);

    repo.metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let tree = repo
        .find_reference(FANOUT_REF)
        .unwrap()
        .peel_to_tree()
        .unwrap();
    let readme = tree.find_entry("README").expect("README present");
    assert_eq!(readme.oid(), extra_blob);
    assert!(readme.mode().is_blob());
}

#[test]
fn delete_from_tree_rooted_ref_yields_parentless_commit() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    let root = write_fanout(&repo, None, &[(target, data)]);
    set_ref(&repo, root);

    repo.metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let tip = repo
        .find_reference(FANOUT_REF)
        .unwrap()
        .peel_to_id()
        .unwrap()
        .detach();
    let commit = repo.find_commit(tip).expect("find commit");
    assert_eq!(commit.parent_ids().count(), 0);
}
