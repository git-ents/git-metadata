//! End-to-end `.fanout` tests against `metadatas` (validation matrix lives in
//! `ref_fanout_table` / `ref_fanout_property`).

use super::helpers::*;
use git_metadata::MetadataRepository;

#[test]
fn fanout_blob_with_trailing_newline_is_accepted() {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"payload");
    let data = empty_tree(&repo);

    let fanout = blob(&repo, b"2\n");
    let hex = hex_of(blob_id);
    let root_id = write_tree(
        &repo,
        vec![
            (vec![".fanout".into()], EntryKind::Blob, fanout),
            (fanout_segments(&hex, 2), EntryKind::Tree, data),
        ],
    );
    set_ref(&repo, root_id);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
}
