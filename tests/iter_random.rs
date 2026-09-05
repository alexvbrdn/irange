//! Randomised cross-checks of `RangeSetIter` against a plain list of the values.

use irange::range::AnyRange;
use irange::RangeSet;
use rand::RngExt;

/// Build a random set together with the values it holds, in order.
fn random_set(rng: &mut impl RngExt) -> (RangeSet<i32>, Vec<i32>) {
    let mut ranges = Vec::new();
    let mut cursor = rng.random_range(-50..50);

    for _ in 0..rng.random_range(0..8) {
        let min = cursor + rng.random_range(1..6);
        let max = min + rng.random_range(0..6);
        ranges.push(AnyRange::new(min, max));
        cursor = max;
    }

    let set = RangeSet::new_from_ranges(&ranges);
    let values = set.iter().collect();
    (set, values)
}

#[test]
fn every_interleaving_of_both_ends_yields_the_values_exactly_once() {
    let mut rng = rand::rng();

    for _ in 0..2_000 {
        let (set, expected) = random_set(&mut rng);
        let mut iter = set.iter();
        let mut front = Vec::new();
        let mut back = Vec::new();

        // Consume from a randomly chosen end until both are exhausted.
        loop {
            let taken = if rng.random_bool(0.5) {
                iter.next().inspect(|&value| front.push(value))
            } else {
                iter.next_back().inspect(|&value| back.push(value))
            };
            if taken.is_none() {
                // One end is empty, so the other one is too.
                if iter.next().is_none() && iter.next_back().is_none() {
                    break;
                }
            }
        }

        back.reverse();
        front.extend(back);
        assert_eq!(expected, front, "set was {set}");
    }
}

#[test]
fn reverse_iteration_is_forward_iteration_reversed() {
    let mut rng = rand::rng();

    for _ in 0..2_000 {
        let (set, expected) = random_set(&mut rng);
        let mut backward: Vec<i32> = set.iter().rev().collect();
        backward.reverse();

        assert_eq!(expected, backward, "set was {set}");
    }
}
