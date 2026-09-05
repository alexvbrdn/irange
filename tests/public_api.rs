//! Tests driving `irange` through its public API, the way a dependent crate sees it.

use irange::range::AnyRange;
use irange::RangeSet;

fn set(ranges: &[(i32, i32)]) -> RangeSet<i32> {
    RangeSet::new_from_ranges(
        &ranges
            .iter()
            .map(|&(min, max)| AnyRange::new(min, max))
            .collect::<Vec<_>>(),
    )
}

#[test]
fn default_is_empty() {
    assert_eq!(RangeSet::<i32>::empty(), RangeSet::<i32>::default());
    assert!(RangeSet::<i32>::default().is_empty());
}

#[test]
fn reference_into_iter_matches_iter() {
    let range = set(&[(3, 4), (7, 9)]);

    let by_method: Vec<i32> = range.iter().collect();
    let by_for_loop: Vec<i32> = (&range).into_iter().collect();

    assert_eq!(by_method, by_for_loop);
    assert_eq!(vec![3, 4, 7, 8, 9], by_method);
}

#[test]
fn collect_ranges_into_a_set() {
    let collected: RangeSet<i32> = [3..=4, 7..=8].into_iter().map(AnyRange::from).collect();
    assert_eq!(set(&[(3, 4), (7, 8)]), collected);

    let empty: RangeSet<i32> = std::iter::empty().collect();
    assert!(empty.is_empty());
}

#[test]
fn operators_match_the_named_methods() {
    let a = set(&[(3, 8), (20, 30)]);
    let b = set(&[(7, 9), (25, 40)]);

    assert_eq!(a.union(&b), &a | &b);
    assert_eq!(a.intersection(&b), &a & &b);
    assert_eq!(a.difference(&b), &a - &b);
    assert_eq!(a.complement(), !&a);
}

#[test]
fn any_range_derives() {
    let range = AnyRange::new(3, 4);
    let copied = range;

    assert_eq!(range, copied);
    assert_eq!(&(3, 4), copied.get_bounds());
    assert!(format!("{range:?}").contains('3'));

    let mut sorted = vec![AnyRange::new(7, 8), AnyRange::new(3, 4)];
    sorted.sort();
    assert_eq!(vec![AnyRange::new(3, 4), AnyRange::new(7, 8)], sorted);
}

#[test]
fn iterator_is_double_ended() {
    let range = set(&[(3, 5), (8, 8), (12, 14)]);

    let forward: Vec<i32> = range.iter().collect();
    assert_eq!(vec![3, 4, 5, 8, 12, 13, 14], forward);

    let mut backward: Vec<i32> = range.iter().rev().collect();
    backward.reverse();
    assert_eq!(forward, backward);

    assert_eq!(Some(14), range.iter().next_back());
    // `last` is the point of the assertion, so not `next_back` however much faster it is.
    #[allow(clippy::double_ended_iterator_last)]
    let last = range.iter().last();
    assert_eq!(Some(14), last);
}

#[test]
fn iterator_ends_meet_without_repeating_or_dropping_a_value() {
    let range = set(&[(3, 5), (8, 8), (12, 14)]);
    let expected: Vec<i32> = range.iter().collect();

    // Every interleaving of `next` and `next_back` must yield each value exactly once:
    // drive the two ends together with every possible split point.
    for take_from_front in 0..=expected.len() {
        let mut iter = range.iter();
        let mut got = Vec::new();
        let mut tail = Vec::new();

        for _ in 0..take_from_front {
            got.push(iter.next().unwrap());
        }
        while let Some(value) = iter.next_back() {
            tail.push(value);
        }
        tail.reverse();
        got.extend(tail);

        assert_eq!(
            expected, got,
            "split after {take_from_front} from the front"
        );
        assert_eq!(None, iter.next(), "iterator is drained and fused");
        assert_eq!(None, iter.next_back(), "iterator is drained and fused");
    }
}

#[test]
fn iterator_alternates_between_ends() {
    let range = set(&[(3, 5), (8, 8), (12, 14)]);
    let expected: Vec<i32> = range.iter().collect();

    let mut iter = range.iter();
    let mut front = Vec::new();
    let mut back = Vec::new();
    while let Some(value) = iter.next() {
        front.push(value);
        match iter.next_back() {
            Some(value) => back.push(value),
            None => break,
        }
    }
    back.reverse();
    front.extend(back);

    assert_eq!(expected, front);
}

#[test]
fn iterator_is_fused_at_both_ends() {
    let range = set(&[(3, 3)]);
    let mut iter = range.iter();

    assert_eq!(Some(3), iter.next());
    for _ in 0..3 {
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());
    }
}

#[test]
fn empty_set_iterates_to_nothing_from_both_ends() {
    let range = RangeSet::<i32>::empty();
    assert_eq!(None, range.iter().next());
    assert_eq!(None, range.iter().next_back());
}

#[test]
fn iterating_the_total_set_from_the_back_terminates() {
    let range = RangeSet::<u8>::total();

    let last_three: Vec<u8> = range.iter().rev().take(3).collect();
    assert_eq!(vec![255, 254, 253], last_three);

    assert_eq!(256, range.iter().count());
    assert_eq!(256, range.iter().rev().count());
}

#[test]
fn hand_written_malformed_set_never_panics_from_either_end() {
    // Reachable only by writing to the public field: an odd bound count, an inverted
    // range and unsorted bounds. Any result is acceptable as long as it terminates.
    for bounds in [
        vec![3],
        vec![3, 4, 7],
        vec![5, 3],
        vec![7, 8, 3, 4],
        vec![3, 4, 4, 5],
    ] {
        let range = RangeSet(bounds.clone());
        let forward: Vec<i32> = range.iter().take(64).collect();
        let backward: Vec<i32> = range.iter().rev().take(64).collect();

        assert!(
            forward.len() < 64 && backward.len() < 64,
            "iteration over {bounds:?} did not terminate"
        );
    }
}
