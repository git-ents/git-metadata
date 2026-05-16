//! Table-driven tests for `MetadataRepository::metadata`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};
use rstest::rstest;

#[rstest]
#[case::default_depth(None, 1)]
#[case::depth_one(Some(1), 1)]
#[case::depth_two(Some(2), 2)]
#[case::depth_three(Some(3), 3)]
#[case::depth_nineteen(Some(19), 19)]
fn writes_leaf_at_configured_depth(#[case] seed: Option<u8>, #[case] want_depth: u8) {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let target = blob(&repo, b"target");

    // Seed the ref with a fanout root at the desired depth (or no `.fanout`
    // entry, exercising the `None` default-depth branch).
    let seeded = write_fanout(&repo, seed, &[]);
    set_ref(&repo, seeded);

    repo.metadata(
        sig(),
        sig(),
        None,
        Some(FANOUT_REF),
        target,
        &data,
        false,
        None,
    )
    .expect("write");

    let depth = repo.metadata_ref_fanout(Some(FANOUT_REF)).expect("depth");
    assert_eq!(depth, want_depth);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(target, data)]));

    // Leaf path matches the depth.
    let tree = repo
        .find_reference(FANOUT_REF)
        .unwrap()
        .peel_to_tree()
        .unwrap();
    let path = leaf_path(target, want_depth);
    let entry = tree
        .lookup_entry(path.iter().map(|p| p.as_slice()))
        .expect("lookup")
        .expect("leaf present");
    assert!(entry.mode().is_tree());
    assert_eq!(entry.detach().oid, data);
}

#[derive(Debug)]
enum Pre {
    /// No pre-existing ref at all.
    Missing,
    /// Ref points at an empty tree (no `.fanout`).
    EmptyTree,
    /// Ref already contains the target leaf with the given data.
    LeafPresent,
}

#[derive(Debug)]
enum Outcome {
    Ok,
    AlreadyExists,
}

#[rstest]
#[case::fresh_no_force(Pre::Missing, false, Outcome::Ok)]
#[case::fresh_force(Pre::Missing, true, Outcome::Ok)]
#[case::empty_tree_no_force(Pre::EmptyTree, false, Outcome::Ok)]
#[case::empty_tree_force(Pre::EmptyTree, true, Outcome::Ok)]
#[case::dup_no_force(Pre::LeafPresent, false, Outcome::AlreadyExists)]
#[case::dup_force(Pre::LeafPresent, true, Outcome::Ok)]
fn force_matrix(#[case] pre: Pre, #[case] force: bool, #[case] outcome: Outcome) {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);
    let target = blob(&repo, b"target");

    match pre {
        Pre::Missing => {}
        Pre::EmptyTree => {
            let root = empty_tree(&repo);
            set_ref(&repo, root);
        }
        Pre::LeafPresent => {
            repo.metadata(
                sig(),
                sig(),
                None,
                Some(FANOUT_REF),
                target,
                &data,
                false,
                None,
            )
            .expect("seed leaf");
        }
    }

    let res = repo.metadata(
        sig(),
        sig(),
        None,
        Some(FANOUT_REF),
        target,
        &data,
        force,
        None,
    );
    match outcome {
        Outcome::Ok => {
            res.expect("must succeed");
            let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
            assert_eq!(sorted(got), expected(&repo, &[(target, data)]));
        }
        Outcome::AlreadyExists => {
            let err = res.expect_err("must error");
            assert!(
                matches!(err, Error::AlreadyExists(o) if o == target),
                "got {err:?}"
            );
        }
    }
}
