//! Tests for the `serde` feature, driven through the public API.
#![cfg(feature = "serde")]

use irange::range::AnyRange;
use irange::RangeSet;

#[test]
fn round_trips_through_json() {
    let range = RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..=9)]);

    let json = serde_json::to_string(&range).unwrap();
    assert_eq!("[3,4,7,9]", json);

    let back: RangeSet<i64> = serde_json::from_str(&json).unwrap();
    assert_eq!(range, back);
}

#[test]
fn round_trips_the_edge_cases() {
    for range in [
        RangeSet::<u8>::empty(),
        RangeSet::<u8>::total(),
        RangeSet::<u8>::new_from_range(0..=0),
        RangeSet::<u8>::new_from_range(255..=255),
    ] {
        let json = serde_json::to_string(&range).unwrap();
        let back: RangeSet<u8> = serde_json::from_str(&json).unwrap();
        assert_eq!(range, back, "round trip of {range} through {json}");
    }
}

#[test]
fn malformed_input_is_rejected_with_a_message() {
    // An odd number of bounds, an inverted range, unsorted ranges, overlapping ranges
    // and adjacent ranges that should have been merged.
    for json in ["[3]", "[4,3]", "[7,8,3,4]", "[3,8,5,9]", "[1,2,3,4]"] {
        let error =
            serde_json::from_str::<RangeSet<i64>>(json).expect_err("{json} should not deserialize");
        assert!(
            error.to_string().contains("invalid RangeSet"),
            "unexpected error for {json}: {error}"
        );
    }
}

#[test]
fn well_formed_input_is_accepted() {
    let range: RangeSet<i64> = serde_json::from_str("[3,4,7,9]").unwrap();
    assert_eq!(vec![3, 4, 7, 8, 9], range.iter().collect::<Vec<_>>());
}
