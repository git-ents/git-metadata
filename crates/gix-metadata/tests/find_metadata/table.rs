//! Table-driven tests for `MetadataRepository::find_metadata`.

use crate::common::*;
use gix_metadata::MetadataRepository;
use rstest::rstest;

#[rstest]
#[case::default_depth(None)]
#[case::depth_one(Some(1))]
#[case::depth_two(Some(2))]
#[case::depth_three(Some(3))]
#[case::depth_nineteen(Some(19))]
fn finds_leaf_at_configured_depth(#[case] depth: Option<u8>) {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let data_inner = blob(&repo, b"payload");
    let data = write_tree(&repo, vec![(vec!["f".into()], EntryKind::Blob, data_inner)]);

    let root = write_fanout(&repo, depth, &[(target, data)]);
    set_ref(&repo, root);

    let got = repo.find_metadata(Some(FANOUT_REF), target).expect("find");
    assert_eq!(got, data);
}

/// `find_metadata` distinguishes hits from misses across many sibling leaves
/// laid out at the same depth.
#[rstest]
#[case(None)]
#[case(Some(1))]
#[case(Some(2))]
#[case(Some(4))]
fn finds_each_leaf_among_siblings(#[case] depth: Option<u8>) {
    let (_dir, repo) = init_repo();
    let data = empty_tree(&repo);

    let leaves: Vec<_> = (0..5)
        .map(|i| (blob(&repo, format!("blob-{i}").as_bytes()), data))
        .collect();
    let root = write_fanout(&repo, depth, &leaves);
    set_ref(&repo, root);

    for (id, expected_data) in &leaves {
        let got = repo.find_metadata(Some(FANOUT_REF), *id).expect("find");
        assert_eq!(got, *expected_data);
    }
}
