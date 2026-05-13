//! Property tests for `MetadataRepository::metadata`.

use crate::common::*;
use git_metadata::MetadataRepository;
use proptest::prelude::*;

fn shape() -> impl Strategy<Value = (Option<u8>, usize)> {
    let depth = prop_oneof![Just(None), (1u8..=19).prop_map(Some),];
    (depth, 1usize..6)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        ..ProptestConfig::default()
    })]

    /// Writing N distinct leaves yields the same set when read back, and the
    /// commit chain is linear with N-1 parents.
    #[test]
    fn round_trip((depth, n) in shape()) {
        let (_dir, repo) = init_repo();
        let data = empty_tree(&repo);

        if let Some(d) = depth {
            let seeded = write_fanout(&repo, Some(d), &[]);
            set_ref(&repo, seeded);
        }

        let mut leaves = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut last_commit: Option<gix::ObjectId> = None;
        let mut commits_written = 0usize;
        for i in 0..n {
            let id = blob(&repo, format!("blob-{i}").as_bytes());
            if !seen.insert(id) {
                continue;
            }
            let c = repo
                .metadata(sig(), sig(), None, Some(FANOUT_REF), id, &data, false)
                .expect("write");
            if let Some(prev) = last_commit {
                let commit = repo.find_commit(c).expect("find");
                let parents: Vec<_> = commit.parent_ids().map(|p| p.detach()).collect();
                prop_assert_eq!(parents, vec![prev]);
            }
            last_commit = Some(c);
            commits_written += 1;
            leaves.push((id, data));
        }

        prop_assert_eq!(commits_written, leaves.len());
        let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
        prop_assert_eq!(sorted(got), expected(&repo, &leaves));
    }

    /// Repeated `force` writes for the same target keep the leaf reachable and
    /// the last data wins.
    #[test]
    fn force_overwrite_last_wins(n in 1usize..6) {
        let (_dir, repo) = init_repo();
        let target = blob(&repo, b"target");

        let mut last_data = empty_tree(&repo);
        for i in 0..n {
            let inner = blob(&repo, format!("v{i}").as_bytes());
            last_data = write_tree(
                &repo,
                vec![(vec!["f".into()], EntryKind::Blob, inner)],
            );
            repo.metadata(sig(), sig(), None, Some(FANOUT_REF), target, &last_data, true)
                .expect("force write");
        }

        let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
        prop_assert_eq!(sorted(got), expected(&repo, &[(target, last_data)]));
    }
}
