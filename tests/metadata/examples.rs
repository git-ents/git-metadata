//! Example-based tests for `MetadataRepository::metadata`.

use crate::common::*;
use git_metadata::{DEFAULT_FANOUT, Error, MetadataRepository};

#[test]
fn missing_ref_creates_new_commit_with_default_fanout() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    let commit_id = repo
        .metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write metadata");

    // Ref now points to the new commit.
    let mut r = repo.find_reference(FANOUT_REF).expect("find ref");
    assert_eq!(r.peel_to_id().unwrap().detach(), commit_id);

    // Fanout depth defaults to DEFAULT_FANOUT and the leaf is readable.
    let depth = repo.metadata_ref_fanout(Some(FANOUT_REF)).expect("depth");
    assert_eq!(depth, DEFAULT_FANOUT);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(target, data)]));
}

#[test]
fn fanout_blob_is_written_even_when_initial_state_is_empty_tree_ref() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    // Pre-existing ref pointing at an empty tree (no `.fanout` blob).
    let empty = empty_tree(&repo);
    set_ref(&repo, empty);

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write metadata");

    // Commit's tree must now contain a `.fanout` blob.
    let tree = repo
        .find_reference(FANOUT_REF)
        .unwrap()
        .peel_to_tree()
        .unwrap();
    let entry = tree.find_entry(".fanout").expect(".fanout present");
    assert!(entry.mode().is_blob());
}

#[test]
fn second_write_uses_first_commit_as_parent() {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let t1 = blob(&repo, b"t1");
    let t2 = blob(&repo, b"t2");

    let c1 = repo
        .metadata(sig(), sig(), None, Some(FANOUT_REF), t1, &data, false)
        .expect("first");
    let c2 = repo
        .metadata(sig(), sig(), None, Some(FANOUT_REF), t2, &data, false)
        .expect("second");

    assert_ne!(c1, c2);
    let commit = repo.find_commit(c2).expect("find commit");
    let parents: Vec<_> = commit.parent_ids().map(|id| id.detach()).collect();
    assert_eq!(parents, vec![c1]);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(t1, data), (t2, data)]));
}

#[test]
fn existing_tree_ref_yields_parentless_commit() {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let target = blob(&repo, b"target");

    // Pre-existing ref pointing directly at a tree (no commit).
    let root = write_fanout(&repo, None, &[]);
    set_ref(&repo, root);

    let c = repo
        .metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("write metadata");
    let commit = repo.find_commit(c).expect("find commit");
    assert_eq!(commit.parent_ids().count(), 0);
}

#[test]
fn existing_leaf_without_force_errors_already_exists() {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let target = blob(&repo, b"target");

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect("first");
    let err = repo
        .metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data, false)
        .expect_err("duplicate must error");
    assert!(
        matches!(err, Error::AlreadyExists(o) if o == target),
        "got {err:?}"
    );
}

#[test]
fn existing_leaf_with_force_overwrites() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data1 = empty_tree(&repo);
    let inner = blob(&repo, b"inner");
    let data2 = write_tree(&repo, vec![(vec!["f".into()], EntryKind::Blob, inner)]);

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data1, false)
        .expect("first");
    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &data2, true)
        .expect("force overwrite");

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(target, data2)]));
}

#[test]
fn none_metadatas_ref_uses_default() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);
    let default_ref = repo.metadata_default_ref().expect("default");

    repo.metadata(sig(), sig(), None, None, target, &data, false)
        .expect("write");
    let got = repo.metadatas(Some(&default_ref)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(target, data)]));
}

#[test]
fn preserves_existing_leaves_at_other_paths() {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let t1 = blob(&repo, b"alpha");
    let t2 = blob(&repo, b"beta");

    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), t1, &data, false)
        .expect("first");
    repo.metadata(sig(), sig(), None, Some(FANOUT_REF), t2, &data, false)
        .expect("second");

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(t1, data), (t2, data)]));
}
