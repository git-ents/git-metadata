//! Property tests for `MetadataRepository::metadata_ref_fanout`.

use super::helpers::*;
use git_metadata::{Error, MetadataRepository};
use proptest::prelude::*;

fn write_fanout_root(repo: &gix::Repository, content: &[u8]) -> gix::ObjectId {
    let fanout = blob(repo, content);
    write_tree(
        repo,
        vec![(vec![".fanout".into()], EntryKind::Blob, fanout)],
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    /// Any valid depth round-trips through the `.fanout` blob.
    #[test]
    fn round_trip_valid_depth(depth in 1u8..=19) {
        let (_dir, repo) = init_repo();
        let root = write_fanout_root(&repo, depth.to_string().as_bytes());
        set_ref(&repo, root);
        let got = repo
            .metadata_ref_fanout(Some(FANOUT_REF))
            .expect("fanout depth");
        prop_assert_eq!(got, depth);
    }

    /// Surrounding ASCII whitespace is trimmed.
    #[test]
    fn whitespace_is_trimmed(
        depth in 1u8..=19,
        lead in 0usize..4,
        trail in 0usize..4,
    ) {
        let mut content = Vec::new();
        content.extend(std::iter::repeat_n(b' ', lead));
        content.extend(depth.to_string().as_bytes());
        content.extend(std::iter::repeat_n(b'\n', trail));

        let (_dir, repo) = init_repo();
        let root = write_fanout_root(&repo, &content);
        set_ref(&repo, root);
        let got = repo
            .metadata_ref_fanout(Some(FANOUT_REF))
            .expect("fanout depth");
        prop_assert_eq!(got, depth);
    }

    /// Out-of-range integers are rejected.
    #[test]
    fn out_of_range_rejected(depth in prop_oneof![Just(0u16), 20u16..=255]) {
        let (_dir, repo) = init_repo();
        let root = write_fanout_root(&repo, depth.to_string().as_bytes());
        set_ref(&repo, root);
        let err = repo
            .metadata_ref_fanout(Some(FANOUT_REF))
            .expect_err("must error");
        prop_assert!(matches!(err, Error::InvalidFanoutDepth { .. }), "got {err:?}");
    }

    /// Arbitrary bytes that cannot parse to a valid depth are rejected.
    /// Filtered to exclude any string that *does* parse to a valid depth
    /// (after trimming).
    #[test]
    fn arbitrary_invalid_rejected(
        content in proptest::collection::vec(any::<u8>(), 0..16)
            .prop_filter("must not be a valid depth", |bytes| {
                match std::str::from_utf8(bytes.trim_ascii()) {
                    Ok(s) => match s.parse::<u8>() {
                        Ok(d) => !(1..=19).contains(&d),
                        Err(_) => true,
                    },
                    Err(_) => true,
                }
            }),
    ) {
        let (_dir, repo) = init_repo();
        let root = write_fanout_root(&repo, &content);
        set_ref(&repo, root);
        let err = repo
            .metadata_ref_fanout(Some(FANOUT_REF))
            .expect_err("must error");
        prop_assert!(matches!(err, Error::InvalidFanoutDepth { .. }), "got {err:?}");
    }
}
