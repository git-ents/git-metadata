//! Property tests for `MetadataRepository::metadata_delete`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};
use proptest::prelude::*;

fn shape() -> impl Strategy<Value = (Option<u8>, usize)> {
    let depth = prop_oneof![Just(None), (1u8..=19).prop_map(Some)];
    (depth, 1usize..12)
}

fn write_unique(repo: &gix::Repository, depth: Option<u8>, n: usize) -> Vec<gix::ObjectId> {
    let data = empty_tree(repo);
    if let Some(d) = depth {
        let seeded = write_fanout(repo, Some(d), &[]);
        set_ref(repo, seeded);
    }
    let mut written = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for i in 0..n {
        let id = blob(repo, format!("blob-{i}").as_bytes());
        if !seen.insert(id) {
            continue;
        }
        repo.metadata(sig(), sig(), Some(FANOUT_REF), id, &data, false)
            .expect("write");
        written.push(id);
    }
    written
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// Deleting every written leaf yields an empty listing.
    #[test]
    fn delete_all_yields_empty_listing((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let written = write_unique(&repo, depth, n);

        for id in &written {
            repo.metadata_delete(*id, Some(FANOUT_REF), sig(), sig())
                .expect("delete");
        }

        let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
        prop_assert!(got.is_empty());
    }

    /// Deleting a subset leaves the remainder findable and removes only the
    /// targeted leaves.
    #[test]
    fn delete_subset_preserves_remainder((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);
        let written = write_unique(&repo, depth, n);
        prop_assume!(written.len() >= 2);

        let (drop, keep) = written.split_at(written.len() / 2);
        for id in drop {
            repo.metadata_delete(*id, Some(FANOUT_REF), sig(), sig())
                .expect("delete");
        }

        for id in drop {
            let err = repo
                .find_metadata(Some(FANOUT_REF), *id)
                .expect_err("dropped must error");
            prop_assert!(matches!(err, Error::NotFound(o) if o == *id), "got {err:?}");
        }
        for id in keep {
            let got = repo.find_metadata(Some(FANOUT_REF), *id).expect("find");
            prop_assert_eq!(got, data);
        }
    }

    /// Delete then re-write succeeds and the leaf is findable again.
    #[test]
    fn delete_then_rewrite_round_trips((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);
        let written = write_unique(&repo, depth, n);

        for id in &written {
            repo.metadata_delete(*id, Some(FANOUT_REF), sig(), sig())
                .expect("delete");
            repo.metadata(sig(), sig(), Some(FANOUT_REF), *id, &data, false)
                .expect("re-write");
            let got = repo.find_metadata(Some(FANOUT_REF), *id).expect("find");
            prop_assert_eq!(got, data);
        }
    }
}
