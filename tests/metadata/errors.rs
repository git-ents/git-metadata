//! Table-driven error-path tests for `MetadataRepository::metadata`.

use crate::common::*;
use git_metadata::{Error, MetadataRepository};
use rstest::rstest;

#[derive(Debug)]
enum Setup {
    /// Repo is fresh; no ref pre-seeded.
    Fresh,
    /// Ref points at the given object kind.
    RefPointsAtBlob,
}

#[derive(Debug)]
enum Target {
    /// Use a freshly-written blob oid (valid).
    FreshBlob,
    /// Use a SHA-256 oid (hash kind mismatch in a SHA-1 repo).
    Sha256,
}

#[derive(Debug)]
enum Meta {
    /// Use a freshly-written empty tree (valid).
    EmptyTree,
    /// Use a freshly-written blob (wrong kind).
    Blob,
    /// Use an oid that doesn't resolve in the odb.
    Missing,
}

#[derive(Debug)]
enum Expect {
    UnsupportedHashKind,
    InvalidType,
    NotFound,
    InvalidRootType,
}

#[rstest]
#[case::sha256_target(
    Setup::Fresh,
    Target::Sha256,
    Meta::EmptyTree,
    Expect::UnsupportedHashKind
)]
#[case::metadata_is_blob(Setup::Fresh, Target::FreshBlob, Meta::Blob, Expect::InvalidType)]
#[case::metadata_missing(Setup::Fresh, Target::FreshBlob, Meta::Missing, Expect::NotFound)]
#[case::ref_points_at_blob(
    Setup::RefPointsAtBlob,
    Target::FreshBlob,
    Meta::EmptyTree,
    Expect::InvalidRootType
)]
fn rejects(
    #[case] setup: Setup,
    #[case] target: Target,
    #[case] meta: Meta,
    #[case] expect: Expect,
) {
    let (_dir, repo) = init_repo();

    match setup {
        Setup::Fresh => {}
        Setup::RefPointsAtBlob => {
            let b = blob(&repo, b"not-a-tree-or-commit");
            set_ref(&repo, b);
        }
    }

    let target_oid = match target {
        Target::FreshBlob => blob(&repo, b"target"),
        Target::Sha256 => gix::ObjectId::empty_tree(gix::hash::Kind::Sha256),
    };

    let meta_oid = match meta {
        Meta::EmptyTree => empty_tree(&repo),
        Meta::Blob => blob(&repo, b"payload"),
        Meta::Missing => {
            gix::ObjectId::from_hex(b"0123456789abcdef0123456789abcdef01234567").unwrap()
        }
    };

    let err = repo
        .metadata(
            sig(),
            sig(),
            None,
            Some(FANOUT_REF),
            target_oid,
            &meta_oid,
            false,
        )
        .expect_err("must error");

    match expect {
        Expect::UnsupportedHashKind => {
            assert!(matches!(err, Error::UnsupportedHashKind(..)), "got {err:?}");
        }
        Expect::InvalidType => {
            assert!(matches!(err, Error::InvalidType(_)), "got {err:?}");
        }
        Expect::NotFound => {
            assert!(matches!(err, Error::NotFound(_)), "got {err:?}");
        }
        Expect::InvalidRootType => {
            assert!(matches!(err, Error::InvalidRootType(_)), "got {err:?}");
        }
    }
}

#[test]
fn validate_propagates_invalid_fanout_depth() {
    let (_dir, repo) = init_repo();
    let root = write_tree(
        &repo,
        vec![(vec![".fanout".into()], EntryKind::Blob, blob(&repo, b"99"))],
    );
    set_ref(&repo, root);

    let err = repo
        .validate_metadata_tree(Some(FANOUT_REF))
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutDepth { .. }),
        "got {err:?}"
    );
}

#[test]
fn validate_propagates_invalid_fanout_type() {
    let (_dir, repo) = init_repo();
    let subtree = empty_tree(&repo);
    let root = write_tree(
        &repo,
        vec![(vec![".fanout".into()], EntryKind::Tree, subtree)],
    );
    set_ref(&repo, root);

    let err = repo
        .validate_metadata_tree(Some(FANOUT_REF))
        .expect_err("must error");
    assert!(
        matches!(err, Error::InvalidFanoutType { .. }),
        "got {err:?}"
    );
}

/// Seed a root whose first fanout segment (the target's `hex[0..2]`) is a blob.
/// `validate_metadata_tree` should detect the corruption; `metadata` itself
/// does not check structural integrity on the write path.
#[test]
fn validate_detects_blob_at_intermediate_segment() {
    let (_dir, repo) = init_repo();
    let target = blob(&repo, b"target");
    let hex = hex_of(target);
    let head: gix::bstr::BString = hex[0..2].into();

    // Depth 2 root: `.fanout = 2`, plus a blob squatting at `hex[0..2]`.
    let squatter = blob(&repo, b"squat");
    let root = write_tree(
        &repo,
        vec![
            (vec![".fanout".into()], EntryKind::Blob, blob(&repo, b"2")),
            (vec![head.clone()], EntryKind::Blob, squatter),
        ],
    );
    set_ref(&repo, root);

    let err = repo
        .validate_metadata_tree(Some(FANOUT_REF))
        .expect_err("must error");
    assert!(
        matches!(&err, Error::FanoutPathConflict(p) if p == &head),
        "got {err:?}"
    );
}
