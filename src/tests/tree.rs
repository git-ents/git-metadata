use crate::tree::*;
use proptest::prelude::*;
use rstest::rstest;

fn oid(hex: &[u8]) -> gix::ObjectId {
    gix::ObjectId::from_hex(hex).expect("valid hex")
}

#[rstest]
#[case::depth_1(1, &["ab", "cdef0123456789abcdef0123456789abcdef01"])]
#[case::depth_2(2, &["ab", "cd", "ef0123456789abcdef0123456789abcdef01"])]
#[case::depth_3(3, &["ab", "cd", "ef", "0123456789abcdef0123456789abcdef01"])]
#[case::depth_19(19, &[
    "ab", "cd", "ef", "01", "23", "45", "67", "89", "ab", "cd",
    "ef", "01", "23", "45", "67", "89", "ab", "cd", "ef", "01",
])]
fn fanout_path_splits_hex(#[case] depth: u8, #[case] want: &[&str]) {
    let id = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let got = fanout_path(id, depth);
    let want: Vec<gix::bstr::BString> = want.iter().map(|s| gix::bstr::BString::from(*s)).collect();
    assert_eq!(got, want);
}

#[rstest]
#[case(1)]
#[case(2)]
#[case(5)]
#[case(19)]
fn fanout_path_round_trips_to_full_hex(#[case] depth: u8) {
    let id = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    let path = fanout_path(id, depth);
    let mut joined = Vec::new();
    for seg in &path {
        joined.extend_from_slice(seg);
    }
    assert_eq!(joined, id.to_hex().to_string().as_bytes());
}

#[test]
fn fanout_path_segment_count_is_depth_plus_one() {
    let id = oid(b"abcdef0123456789abcdef0123456789abcdef01");
    for d in 1u8..=19 {
        assert_eq!(fanout_path(id, d).len(), d as usize + 1);
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Every prefix segment is exactly 2 hex chars; the trailing leaf
    /// segment is the remaining `40 - 2 * depth` chars.
    #[test]
    fn fanout_path_prefix_segments_are_two_chars(
        hex in "[0-9a-f]{40}",
        depth in 1u8..=19,
    ) {
        let id = oid(hex.as_bytes());
        let path = fanout_path(id, depth);
        let (leaf, prefix) = path.split_last().expect("depth + 1 segments");
        for seg in prefix {
            prop_assert_eq!(seg.len(), 2);
        }
        prop_assert_eq!(leaf.len(), 40 - 2 * depth as usize);
    }
}
