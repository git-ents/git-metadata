//! Tests for `DEFAULT_FANOUT` and the absent-`.fanout`-blob fallthrough.

use crate::common::*;
use gix_metadata::{DEFAULT_FANOUT, MetadataRepository};

#[test]
/// If the default fanout ever changes, this test failure is the documentation needed
/// to note the breaking change.
fn default_fanout_is_one() {
    assert_eq!(DEFAULT_FANOUT, 1);
}

#[test]
fn explicit_default_fanout_matches_implicit() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    // Writing `.fanout = DEFAULT_FANOUT` explicitly must behave identically
    // to omitting the blob.
    let root = write_fanout(&repo, Some(DEFAULT_FANOUT), &[(blob_id, data)]);
    set_ref(&repo, root);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}
