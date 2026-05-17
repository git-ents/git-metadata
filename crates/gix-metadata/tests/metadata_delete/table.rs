//! Table-driven tests for `MetadataRepository::metadata_delete`.

use crate::common::*;
use gix_metadata::MetadataRepository;
use rstest::rstest;

#[rstest]
#[case::default_depth(None)]
#[case::depth_one(Some(1))]
#[case::depth_two(Some(2))]
#[case::depth_three(Some(3))]
#[case::depth_nineteen(Some(19))]
fn deletes_leaf_at_configured_depth(#[case] depth: Option<u8>) {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data = empty_tree(&repo);

    let root = write_fanout(&repo, depth, &[(target, data)]);
    set_ref(&repo, root);

    repo.metadata_delete(target, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert!(got.is_empty());
}

#[rstest]
#[case(None)]
#[case(Some(1))]
#[case(Some(2))]
#[case(Some(4))]
fn deletes_one_among_siblings(#[case] depth: Option<u8>) {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);

    let leaves: Vec<_> = (0..5)
        .map(|i| (blob(&repo, format!("blob-{i}").as_bytes()), data))
        .collect();
    let root = write_fanout(&repo, depth, &leaves);
    set_ref(&repo, root);

    let (drop_id, _) = leaves[2];
    repo.metadata_delete(drop_id, Some(FANOUT_REF), sig(), sig(), None)
        .expect("delete");

    let want: Vec<_> = leaves
        .iter()
        .copied()
        .filter(|(id, _)| *id != drop_id)
        .collect();
    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &want));
}
