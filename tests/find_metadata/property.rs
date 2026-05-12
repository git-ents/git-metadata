//! Property tests for `MetadataRepository::find_metadata`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};
use proptest::prelude::*;

fn shape() -> impl Strategy<Value = (Option<u8>, usize)> {
    let depth = prop_oneof![Just(None), (1u8..=19).prop_map(Some)];
    (depth, 1usize..6)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        ..ProptestConfig::default()
    })]

    /// Every leaf written via `write_fanout` is findable and returns the
    /// associated data oid.
    #[test]
    fn round_trip_each_leaf((depth, n) in shape()) {
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
        let root = write_fanout(&repo, depth, &leaves);
        set_ref(&repo, root);

        for (id, expected_data) in &leaves {
            let got = repo.find_metadata(Some(FANOUT_REF), *id).expect("find");
            prop_assert_eq!(got, *expected_data);
        }
    }

    /// An id whose hex prefix doesn't collide with any present leaf reports
    /// `NotFound` (no spurious hits, no other error class).
    #[test]
    fn absent_ids_report_not_found((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);

        let mut leaves = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            let id = blob(&repo, format!("present-{i}").as_bytes());
            if seen.insert(id) {
                leaves.push((id, data));
            }
        }
        let root = write_fanout(&repo, depth, &leaves);
        set_ref(&repo, root);

        // Generate candidate absent ids and check any that aren't accidentally
        // present in the leaf set.
        for i in 0..n + 3 {
            let absent = blob(&repo, format!("absent-{i}").as_bytes());
            if seen.contains(&absent) {
                continue;
            }
            let err = repo
                .find_metadata(Some(FANOUT_REF), absent)
                .expect_err("absent must error");
            prop_assert!(matches!(err, Error::NotFound(o) if o == absent), "got {err:?}");
        }
    }

    /// `metadata` then `find_metadata` returns the data oid that was written,
    /// across the full valid depth range (seeded by a depth-only fanout root).
    #[test]
    fn write_then_find((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);

        if let Some(d) = depth {
            let seeded = write_fanout(&repo, Some(d), &[]);
            set_ref(&repo, seeded);
        }

        let mut written = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            let id = blob(&repo, format!("blob-{i}").as_bytes());
            if !seen.insert(id) {
                continue;
            }
            repo.metadata(sig(), sig(), Some(FANOUT_REF), id, &data, false)
                .expect("write");
            written.push(id);
        }

        for id in &written {
            let got = repo.find_metadata(Some(FANOUT_REF), *id).expect("find");
            prop_assert_eq!(got, data);
        }
    }
}
