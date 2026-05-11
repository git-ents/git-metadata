//! Property tests.

use super::helpers::*;
use git_metadata::MetadataRepository;
use proptest::prelude::*;

/// Strategy: a (depth, leaf-count) shape. `depth` covers the full valid range.
fn shape() -> impl Strategy<Value = (Option<u8>, usize)> {
    let depth = prop_oneof![Just(None), (1u8..=19).prop_map(Some),];
    (depth, 0usize..8)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    #[test]
    fn round_trip((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);

        let mut leaves = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            let id = blob(&repo, format!("blob-{i}").as_bytes());
            if seen.insert(id) {
                leaves.push((id, data));
            }
        }
        let root_id = write_fanout(&repo, depth, &leaves);
        set_ref(&repo, root_id);

        let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
        prop_assert_eq!(sorted(got), expected(&repo, &leaves));
    }

    #[test]
    fn noise_entries_never_change_result((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);
        let noise_blob = blob(&repo, b"noise");

        let mut leaves = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            let id = blob(&repo, format!("blob-{i}").as_bytes());
            if seen.insert(id) {
                leaves.push((id, data));
            }
        }

        // Build the legitimate fanout, then graft noise siblings at root.
        let mut root = Node::dir();
        if let Some(d) = depth {
            let fanout_blob = blob(&repo, d.to_string().as_bytes());
            root.insert(&[b".fanout" as &[u8]], Node::BlobRef(fanout_blob));
        }
        let effective_depth = depth.unwrap_or(1) as usize;
        for (id, d_oid) in &leaves {
            let hex = hex_of(*id);
            let segs = fanout_segments(&hex, effective_depth);
            let seg_refs: Vec<&[u8]> = segs.iter().map(|s| s.as_slice()).collect();
            root.insert(&seg_refs, Node::TreeRef(*d_oid));
        }
        // Noise: non-hex name, too-short hex, blob-mode at a plausible path.
        root.insert(&[b"README" as &[u8]], Node::BlobRef(noise_blob));
        root.insert(&[b"docs" as &[u8]], Node::TreeRef(data));
        root.insert(&[b"abc" as &[u8]], Node::TreeRef(data));
        let root_id = write_tree(&repo, &root);
        set_ref(&repo, root_id);

        let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
        prop_assert_eq!(sorted(got), expected(&repo, &leaves));
    }
}
