//! Parameterized leaf-shape classification (default depth = 1, i.e. 2/38).

use super::helpers::*;
use git_metadata::MetadataRepository;
use rstest::rstest;

#[derive(Debug)]
enum Name {
    /// Use `[hex[0..2], hex[2..40]]` for the row's freshly-written blob.
    ValidShape,
    /// Insert at the given literal path (split on `/`).
    Literal(&'static [&'static [u8]]),
}

#[derive(Debug)]
enum Expect {
    Included,
    Skipped,
}

#[rstest]
#[case::valid_shape_tree(Name::ValidShape, true, Expect::Included)]
#[case::valid_shape_blob_mode(Name::ValidShape, false, Expect::Skipped)]
#[case::flat_40_char_hex(Name::Literal(&[b"abcdef0123456789abcdef0123456789abcdef01"]), true, Expect::Skipped)]
#[case::non_hex_prefix(Name::Literal(&[b"zz", b"cdef0123456789abcdef0123456789abcdef0123"]), true, Expect::Skipped)]
#[case::non_hex_leaf(Name::Literal(&[b"ab", b"NOPE0123456789abcdef0123456789abcdef0123"]), true, Expect::Skipped)]
#[case::prefix_wrong_length(Name::Literal(&[b"a", b"bcdef0123456789abcdef0123456789abcdef012"]), true, Expect::Skipped)]
#[case::leaf_wrong_length(Name::Literal(&[b"ab", b"cdef"]), true, Expect::Skipped)]
#[case::too_deep(Name::Literal(&[b"ab", b"cd", b"ef0123456789abcdef0123456789abcdef0123"]), true, Expect::Skipped)]
fn leaf_classification(#[case] name: Name, #[case] as_tree: bool, #[case] expect: Expect) {
    let (_dir, repo) = init_repo();
    let blob_id = blob(&repo, b"v");
    let data = empty_tree(&repo);

    let path_owned: Vec<Vec<u8>> = match name {
        Name::ValidShape => {
            let hex = hex_of(blob_id);
            vec![hex[0..2].to_vec(), hex[2..].to_vec()]
        }
        Name::Literal(parts) => parts.iter().map(|p| p.to_vec()).collect(),
    };
    let path_refs: Vec<&[u8]> = path_owned.iter().map(|s| s.as_slice()).collect();
    let child = if as_tree {
        Node::TreeRef(data)
    } else {
        Node::BlobRef(blob(&repo, b"payload"))
    };

    let mut root = Node::dir();
    root.insert(&path_refs, child);
    let root_id = write_tree(&repo, &root);
    set_ref(&repo, root_id);

    let got = repo.metadatas(Some(FANOUT_REF)).expect("metadatas");
    match expect {
        Expect::Included => {
            assert_eq!(sorted(got), expected(&repo, &[(blob_id, data)]));
        }
        Expect::Skipped => {
            assert!(got.is_empty());
        }
    }
}
