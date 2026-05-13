//! Property tests.

use crate::common::*;
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
        let mut entries: Vec<(Vec<gix::bstr::BString>, EntryKind, gix::ObjectId)> = Vec::new();
        if let Some(d) = depth {
            let fanout_blob = blob(&repo, d.to_string().as_bytes());
            entries.push((vec![".fanout".into()], EntryKind::Blob, fanout_blob));
        }
        let effective_depth = depth.unwrap_or(1) as usize;
        for (id, d_oid) in &leaves {
            let hex = hex_of(*id);
            entries.push((fanout_segments(&hex, effective_depth), EntryKind::Tree, *d_oid));
        }
        // Noise: non-hex name, too-short hex, blob-mode at a plausible path.
        entries.push((vec!["README".into()], EntryKind::Blob, noise_blob));
        entries.push((vec!["docs".into()], EntryKind::Tree, data));
        entries.push((vec!["abc".into()], EntryKind::Tree, data));
        let root_id = write_tree(&repo, entries);
        set_ref(&repo, root_id);

        let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
        prop_assert_eq!(sorted(got), expected(&repo, &leaves));
    }
}
