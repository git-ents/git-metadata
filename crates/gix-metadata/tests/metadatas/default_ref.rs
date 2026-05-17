//! Tests for `metadata_default_ref` and the `metadatas(None)` fallthrough.

use crate::common::*;
use gix_metadata::MetadataRepository;

const DEFAULT_REF: &str = "refs/metadata/objects";

#[test]
/// If the default ref ever changes, this test failure is the documentation needed
/// to note the breaking change.
fn default_ref_is_refs_metadata_objects() {
    let (_dir, repo) = init_repo();
    assert_eq!(
        repo.metadata_default_ref().expect("default ref"),
        DEFAULT_REF
    );
}

#[test]
fn metadatas_none_uses_default_ref() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    let root_id = write_fanout(&repo, None, &[(blob_id, data)]);
    set_ref_named(&repo, DEFAULT_REF, root_id);

    let got = repo.metadatas(None).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}
