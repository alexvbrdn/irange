//! A data structure to store and manipulate ranges of integers with set operations.
//!
//! [`RangeSet`] holds an arbitrary set of integers as a sorted collection of
//! non-overlapping inclusive ranges. It is compact for values that come in runs — a
//! set such as "every codepoint that is a word character" costs a handful of bounds
//! rather than one entry per value — and it supports the usual set algebra:
//! [`union`], [`intersection`], [`difference`], [`complement`], and the containment
//! queries [`contains`] and [`contains_all`].
//!
//! Supported types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`,
//! `i64`, `i128` and `isize`.
//!
//! # Example
//!
//! ```
//! use irange::range::AnyRange;
//! use irange::RangeSet;
//!
//! let range1 = RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);
//! let range2 = RangeSet::<i64>::new_from_range(-2..=4);
//!
//! assert_eq!("[ -2..=4 7..=8 ]", range1.union(&range2).to_string());
//! assert_eq!("[ 3..=4 ]", range1.intersection(&range2).to_string());
//! assert_eq!("[ 7..=8 ]", range1.difference(&range2).to_string());
//!
//! let values: Vec<i64> = range1.union(&range2).iter().collect();
//! assert_eq!(vec![-2, -1, 0, 1, 2, 3, 4, 7, 8], values);
//! ```
//!
//! # Representation
//!
//! A `RangeSet` is a flat `Vec` of inclusive bounds: the elements at an even index are
//! lower bounds and the elements at an odd index are the matching upper bounds, so
//! `[ 3..=4 7..=8 ]` is stored as `vec![3, 4, 7, 8]`.
//!
//! Every set has exactly one such representation. The bounds are even in number, no
//! range is inverted, and consecutive ranges are sorted and separated by at least one
//! value — two ranges that touch, like `1..=2` and `3..=4`, are merged into `1..=4`.
//! The constructors maintain this invariant, and [`RangeSet::new_from_bounds`] checks
//! it when you build a set from raw bounds. Writing to the public field directly
//! bypasses the check and gives unspecified (but never panicking) results.
//!
//! # Complexity
//!
//! `n` is the total number of ranges involved. Every operation makes a single ordered
//! pass over the bounds, so nothing here is worse than linear.
//!
//! | Operation | Time | Space |
//! |---|---|---|
//! | [`union`], [`intersection`], [`difference`], [`complement`] | `O(n)` | `O(n)` |
//! | [`has_intersection`], [`contains_all`] | `O(n)` | `O(1)` |
//! | [`contains`] | `O(log n)` | `O(1)` |
//! | [`is_total`], [`is_empty`] | `O(1)` | `O(1)` |
//!
//! # Feature flags
//!
//! - `serde` — implement `Serialize` and `Deserialize` for [`RangeSet`], using the flat
//!   list of bounds as the serialized form. Deserializing validates the invariant above
//!   and fails with a descriptive error rather than accepting a malformed set.
//!
//! [`union`]: RangeSet::union
//! [`intersection`]: RangeSet::intersection
//! [`difference`]: RangeSet::difference
//! [`complement`]: RangeSet::complement
//! [`contains`]: RangeSet::contains
//! [`contains_all`]: RangeSet::contains_all
//! [`has_intersection`]: RangeSet::has_intersection
//! [`is_total`]: RangeSet::is_total
//! [`is_empty`]: RangeSet::is_empty

#![warn(missing_docs)]

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

use std::ops::{Bound, RangeBounds};

use integer::NumericInteger;
use range::AnyRange;

/// The integer types a [`RangeSet`] can hold.
pub mod integer;
/// A single range of integers, used to build a [`RangeSet`].
pub mod range;

fn range_to_bounds<T: NumericInteger, R: RangeBounds<T>>(range: &R) -> Option<(T, T)> {
    let min = match range.start_bound() {
        Bound::Included(t) => *t,
        Bound::Excluded(t) => {
            if *t == T::max_value() {
                return None;
            }
            *t + T::one()
        }
        Bound::Unbounded => T::min_value(),
    };
    let max = match range.end_bound() {
        Bound::Included(t) => *t,
        Bound::Excluded(t) => {
            if *t == T::min_value() {
                return None;
            }
            *t - T::one()
        }
        Bound::Unbounded => T::max_value(),
    };

    if min > max {
        None
    } else {
        Some((min, max))
    }
}

/// A structure holding a collection of `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` or `isize`.
#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RangeSet<T: NumericInteger>(
    /// In this collection all the elements with even index represent the lower bounds (inclusive) and all the odd index represent the upper bounds (inclusive).
    pub Vec<T>,
);

#[cfg(feature = "serde")]
impl<'de, T: NumericInteger + Deserialize<'de>> Deserialize<'de> for RangeSet<T> {
    fn deserialize<D>(deserializer: D) -> Result<RangeSet<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bounds = Vec::<T>::deserialize(deserializer)?;
        RangeSet::new_from_bounds(bounds).ok_or_else(|| {
            serde::de::Error::custom(
                "invalid RangeSet: expected an even number of inclusive bounds describing sorted, non-overlapping and non-adjacent ranges",
            )
        })
    }
}

impl<T: NumericInteger> std::fmt::Display for RangeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[ ")?;
        for bounds in self.0.chunks_exact(2) {
            write!(f, "{}..={} ", bounds[0], bounds[1])?;
        }
        write!(f, "]")
    }
}

/// A structure to hold the iterator of a `RangeSet` instance.
pub struct RangeSetIter<'a, T: NumericInteger> {
    range_set: &'a RangeSet<T>,
    index: usize,
    value: Option<T>,
}

impl<'a, T: NumericInteger> Iterator for RangeSetIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(min), Some(max)) = (
            self.range_set.0.get(self.index),
            self.range_set.0.get(self.index + 1),
        ) {
            if let Some(value) = self.value {
                if value >= *max {
                    self.index += 2;
                    if let (Some(min), Some(_)) = (
                        self.range_set.0.get(self.index),
                        self.range_set.0.get(self.index + 1),
                    ) {
                        self.value = Some(*min);
                        Some(*min)
                    } else {
                        None
                    }
                } else {
                    let next = value + T::one();
                    self.value = Some(next);

                    self.value
                }
            } else {
                self.value = Some(*min);
                self.value
            }
        } else {
            None
        }
    }
}

impl<T: NumericInteger> RangeSet<T> {
    /// Return an iterator to iterate in order over all the values contained.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range = RangeSet::new_from_range(2..=5);
    ///
    /// for value in range.iter() {
    ///     print!("{value} "); // 2 3 4 5
    /// }
    /// ```
    pub fn iter(&self) -> RangeSetIter<'_, T> {
        RangeSetIter {
            range_set: self,
            index: 0,
            value: None,
        }
    }

    /// Create a new instance from the given range.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// // Contains 2, 3
    /// RangeSet::new_from_range(2..4);
    ///
    /// // Contains 0, 1
    /// RangeSet::<u8>::new_from_range(..2);
    ///
    /// // Contains 0, 1, 2
    /// RangeSet::<u32>::new_from_range(..=2);
    /// ```
    pub fn new_from_range<R: RangeBounds<T>>(range: R) -> RangeSet<T> {
        match range_to_bounds(&range) {
            Some((min, max)) => RangeSet(vec![min, max]),
            None => RangeSet::empty(),
        }
    }

    /// Create a new instance from the given ranges.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    /// use irange::range::AnyRange;
    ///
    /// // Contains 3, 4, 7, 8
    /// RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);
    /// ```
    pub fn new_from_ranges(ranges: &[AnyRange<T>]) -> RangeSet<T> {
        let mut ranges: Vec<(T, T)> = ranges
            .iter()
            .map(|range| range.get_bounds())
            .filter(|(min, max)| max >= min)
            .copied()
            .collect();
        ranges.sort_by_key(|range| range.0);

        let mut bounds = Vec::with_capacity(ranges.len() * 2);
        let mut current_max = T::min_value();
        for (min, max) in ranges {
            if bounds.is_empty() || (current_max < T::max_value() && min > current_max + T::one()) {
                bounds.push(min);
                bounds.push(max);
                current_max = max;
            } else if max > current_max {
                *bounds.last_mut().unwrap() = max;
                current_max = max;
            }
        }

        bounds.shrink_to_fit();
        RangeSet(bounds)
    }

    /// Create a new instance from the raw collection of bounds used internally: the elements with
    /// an even index are the lower bounds (inclusive) and the elements with an odd index are the
    /// upper bounds (inclusive).
    ///
    /// Return `None` if the collection is not a valid representation, i.e. if it does not hold an
    /// even number of bounds, if a range is inverted, or if two consecutive ranges are not sorted,
    /// overlap, or merely touch each other (they should be merged into a single range).
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// // Contains 2, 3, 4, 7, 8
    /// assert!(RangeSet::<u8>::new_from_bounds(vec![2, 4, 7, 8]).is_some());
    ///
    /// // 4..=6 and 7..=8 are adjacent, they must be given as a single 4..=8 range
    /// assert!(RangeSet::<u8>::new_from_bounds(vec![4, 6, 7, 8]).is_none());
    /// ```
    pub fn new_from_bounds(bounds: Vec<T>) -> Option<RangeSet<T>> {
        if bounds.len() % 2 == 1 {
            return None;
        }

        let mut previous_max: Option<T> = None;
        for range in bounds.chunks_exact(2) {
            let (min, max) = (range[0], range[1]);
            if min > max {
                return None;
            }
            if let Some(previous_max) = previous_max {
                // The ranges must be sorted and separated by at least one value.
                if previous_max >= T::max_value() || min <= previous_max + T::one() {
                    return None;
                }
            }
            previous_max = Some(max);
        }

        Some(RangeSet(bounds))
    }

    /// Create a new instance that does not contain any value.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// // Contains nothing
    /// RangeSet::<i32>::empty();
    /// ```
    #[inline]
    pub fn empty() -> RangeSet<T> {
        RangeSet(vec![])
    }

    /// Create a new instance that contains all possible values.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// // Contains all values that can be stored into a u8
    /// // -> 0..=255
    /// RangeSet::<u8>::total();
    ///
    /// // Contains all values that can be stored into a i16
    /// // -> -32768..=32767
    /// RangeSet::<i16>::total();
    /// ```
    #[inline]
    pub fn total() -> RangeSet<T> {
        RangeSet(vec![T::min_value(), T::max_value()])
    }

    /// Return `true` if it contains all the possible values.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let total = RangeSet::<u128>::total();
    /// assert!(total.is_total());
    ///
    /// let range = RangeSet::<u32>::new_from_range(..=2);
    /// assert!(!range.is_total());
    /// ```
    #[inline]
    pub fn is_total(&self) -> bool {
        self.0.len() >= 2 && self.0[0] == T::min_value() && self.0[1] >= T::max_value()
    }

    /// Return `true` if it does not contain any value.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let empty = RangeSet::<u128>::empty();
    /// assert!(empty.is_empty());
    ///
    /// let range = RangeSet::<i64>::new_from_range(2..4);
    /// assert!(!range.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return `true` if it contains the given value.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range = RangeSet::<i64>::new_from_range(2..4);
    /// assert!(range.contains(2));
    /// assert!(range.contains(3));
    /// assert!(!range.contains(4));
    /// ```
    pub fn contains(&self, value: T) -> bool {
        if self.is_empty() {
            return false;
        }
        let first = *self.0.first().unwrap();
        let last = *self.0.last().unwrap();
        if value < first || value > last {
            return false;
        } else if value == first || value == last {
            return true;
        }

        let position = self.0.partition_point(|&x| x < value);

        self.0[position] == value || position % 2 != 0
    }

    /// Return `true` if it contains the given `RangeSet`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range1 = RangeSet::<i64>::new_from_range(11..90);
    /// let range2 = RangeSet::<i64>::new_from_range(19..23);
    ///
    /// assert!(range1.contains_all(&range2));
    /// assert!(!range2.contains_all(&range1));
    /// ```
    pub fn contains_all(&self, that: &RangeSet<T>) -> bool {
        if self.is_total() || that.is_empty() {
            return true;
        }
        if self.is_empty() || that.is_total() {
            return false;
        }

        let mut self_i = 0;
        let mut that_i = 0;

        while that_i + 1 < that.0.len() {
            if self_i + 1 >= self.0.len() {
                return false;
            } else {
                let self_min = self.0[self_i];
                let self_max = self.0[self_i + 1];

                let that_min = that.0[that_i];
                let that_max = that.0[that_i + 1];

                if self_min <= that_min && self_max >= that_max {
                    that_i += 2;
                } else if self_max > that_min {
                    return false;
                } else {
                    self_i += 2;
                }
            }
        }
        true
    }

    /// Return the union with the given `RangeSet`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range1 = RangeSet::<i64>::new_from_range(2..4);
    /// let range2 = RangeSet::<i64>::new_from_range(3..=5);
    ///
    /// // Contains 2..=5
    /// let union = range1.union(&range2);
    /// ```
    pub fn union(&self, that: &RangeSet<T>) -> RangeSet<T> {
        if self.is_empty() || that.is_total() {
            return that.clone();
        } else if that.is_empty() || self.is_total() {
            return self.clone();
        }

        let mut new_range = Vec::with_capacity(self.0.len() + that.0.len());

        let mut self_i = 0;
        let mut that_i = 0;

        let mut current_min = T::min_value();
        let mut current_max = T::min_value();
        let mut current_i = None;

        while self_i + 1 < self.0.len() || that_i + 1 < that.0.len() {
            if that_i + 1 < that.0.len()
                && (self_i + 1 >= self.0.len() || self.0[self_i] > that.0[that_i])
            {
                let (that_min, that_max) = (that.0[that_i], that.0[that_i + 1]);

                if let Some(ci) = current_i {
                    if that_min <= current_max + T::one() && that_max >= current_max {
                        new_range[ci + 1] = that_max;
                    } else if that_min < current_min || that_max > current_max {
                        new_range.extend_from_slice(&[that_min, that_max]);
                    }
                } else {
                    new_range.extend_from_slice(&[that_min, that_max]);
                }

                that_i += 2;
            } else {
                let (self_min, self_max) = (self.0[self_i], self.0[self_i + 1]);

                if let Some(ci) = current_i {
                    if self_min <= current_max + T::one() && self_max >= current_max {
                        new_range[ci + 1] = self_max;
                    } else if self_min < current_min || self_max > current_max {
                        new_range.extend_from_slice(&[self_min, self_max]);
                    }
                } else {
                    new_range.extend_from_slice(&[self_min, self_max]);
                }

                self_i += 2;
            }
            current_min = new_range[new_range.len() - 2];
            current_max = new_range[new_range.len() - 1];
            if current_max == T::max_value() {
                break;
            }
            current_i = Some(new_range.len() - 2);
        }

        new_range.shrink_to_fit();
        RangeSet(new_range)
    }

    /// Return `true` if there is common value with the given `RangeSet`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range1 = RangeSet::<i64>::new_from_range(2..4);
    /// let range2 = RangeSet::<i64>::new_from_range(3..=5);
    /// let range3 = RangeSet::<i64>::new_from_range(5..13);
    ///
    /// assert!(range1.has_intersection(&range2));
    /// assert!(!range1.has_intersection(&range3));
    /// assert!(range2.has_intersection(&range3));
    /// ```
    pub fn has_intersection(&self, that: &RangeSet<T>) -> bool {
        let mut i = 0;
        let mut j = 0;

        while i + 1 < self.0.len() && j + 1 < that.0.len() {
            let self_min = self.0[i];
            let self_max = self.0[i + 1];
            let that_min = that.0[j];
            let that_max = that.0[j + 1];

            if self_max < that_min {
                i += 2;
            } else if that_max < self_min {
                j += 2;
            } else {
                return true;
            }
        }

        false
    }

    /// Return the intersection with the given `RangeSet`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range1 = RangeSet::<i64>::new_from_range(2..4);
    /// let range2 = RangeSet::<i64>::new_from_range(3..=5);
    ///
    /// // Contains 2..=3
    /// let intersection = range1.intersection(&range2);
    /// ```
    pub fn intersection(&self, that: &RangeSet<T>) -> RangeSet<T> {
        if self.is_empty() || that.is_empty() {
            return RangeSet::empty();
        } else if self.is_total() {
            return that.clone();
        } else if that.is_total() {
            return self.clone();
        }

        let mut new_range = Vec::with_capacity(self.0.len() + that.0.len());

        let mut i = 0;
        let mut j = 0;

        while i + 1 < self.0.len() && j + 1 < that.0.len() {
            let self_min = self.0[i];
            let self_max = self.0[i + 1];
            let that_min = that.0[j];
            let that_max = that.0[j + 1];

            if self_max < that_min {
                i += 2;
            } else if that_max < self_min {
                j += 2;
            } else {
                new_range.push(std::cmp::max(self_min, that_min));
                new_range.push(std::cmp::min(self_max, that_max));

                if self_max < that_max {
                    i += 2;
                } else {
                    j += 2;
                }
            }
        }

        new_range.shrink_to_fit();
        RangeSet(new_range)
    }

    /// Return the complement.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range = RangeSet::<u8>::new_from_range(2..4);
    ///
    /// // Contains 0..=1 + 4..=255
    /// range.complement();
    /// ```
    pub fn complement(&self) -> RangeSet<T> {
        if self.is_empty() {
            return Self::total();
        } else if self.is_total() {
            return Self::empty();
        }

        let mut new_range = Vec::with_capacity(self.0.len() + 2);

        for bounds in self.0.chunks_exact(2) {
            let (min, max) = (bounds[0], bounds[1]);

            if new_range.is_empty() && min != T::min_value() {
                new_range.push(T::min_value());
                new_range.push(min - T::one());
            }

            if new_range.len() % 2 == 1 {
                if min == T::min_value() {
                    new_range.pop();
                } else {
                    new_range.push(min - T::one());
                }
            }
            if max < T::max_value() {
                new_range.push(max + T::one());
            }
        }
        if new_range.len() % 2 == 1 {
            new_range.push(T::max_value());
        }

        new_range.shrink_to_fit();
        RangeSet(new_range)
    }

    /// Return the difference with the given `RangeSet`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::RangeSet;
    ///
    /// let range1 = RangeSet::<i64>::new_from_range(2..4);
    /// let range2 = RangeSet::<i64>::new_from_range(3..=5);
    ///
    /// // Contains 2
    /// let difference = range1.difference(&range2);
    /// ```
    #[inline]
    pub fn difference(&self, that: &RangeSet<T>) -> RangeSet<T> {
        self.intersection(&that.complement())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use super::*;

    macro_rules! test_empty_and_total_for_types {
        ($($t:ty),*) => {
            $(
                let empty = RangeSet::<$t>::empty();
                assert!(empty.is_empty());
                assert!(!empty.is_total());

                let total = RangeSet::<$t>::total();
                assert!(!total.is_empty());
                assert!(total.is_total());
            )*
        };
    }

    #[test]
    fn test_empty_and_total() -> Result<(), String> {
        test_empty_and_total_for_types!(
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        );

        Ok(())
    }

    #[test]
    fn test_new_from_range() -> Result<(), String> {
        assert_eq!(
            RangeSet::<u8>(vec![3, 4]),
            RangeSet::<u8>::new_from_range(3..5)
        );

        assert_eq!(
            RangeSet::<u8>(vec![3, 5]),
            RangeSet::<u8>::new_from_range(3..=5)
        );

        assert_eq!(
            RangeSet::<u8>(vec![0, 5]),
            RangeSet::<u8>::new_from_range(..=5)
        );

        assert_eq!(
            RangeSet::<u8>(vec![2, 255]),
            RangeSet::<u8>::new_from_range(2..)
        );

        assert_eq!(
            RangeSet::<u8>(vec![0, 255]),
            RangeSet::<u8>::new_from_range(..)
        );

        Ok(())
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // reversed ranges are used on purpose here
    fn test_new_from_ranges() -> Result<(), String> {
        assert_eq!(
            RangeSet(vec![3, 5, 9, 14]),
            RangeSet::new_from_ranges(&[AnyRange::from(3..=5), AnyRange::from(9..15)])
        );

        assert_eq!(
            RangeSet(vec![3, 5, 9, 14]),
            RangeSet::new_from_ranges(&[AnyRange::from(9..15), AnyRange::from(3..=5)])
        );

        assert_eq!(
            RangeSet(vec![3, 5, 9, 14]),
            RangeSet::new_from_ranges(&[
                AnyRange::from(9..15),
                AnyRange::from(23..18),
                AnyRange::from(3..=5),
                AnyRange::from(4..=4)
            ])
        );

        assert_eq!(
            RangeSet(vec![3, 5, 9, 14]),
            RangeSet::new_from_ranges(&[
                AnyRange::from(9..15),
                AnyRange::from(23..18),
                AnyRange::from(3..=5)
            ])
        );

        assert_eq!(
            RangeSet(vec![3, 255]),
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(4..), AnyRange::from(3..=5)])
        );

        assert_eq!(
            RangeSet(vec![0, 5]),
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(..=4), AnyRange::from(3..=5)])
        );

        assert_eq!(
            RangeSet::total(),
            RangeSet::new_from_ranges(&[AnyRange::from(..), AnyRange::from(3..=5)])
        );

        assert_eq!(
            RangeSet::empty(),
            RangeSet::new_from_ranges(&[AnyRange::from(3..1), AnyRange::from(10..=5)])
        );

        Ok(())
    }

    #[test]
    fn test_contains_all() -> Result<(), String> {
        assert!(RangeSet::<u8>::empty().contains_all(&RangeSet::empty()));
        assert!(RangeSet::<u8>::total().contains_all(&RangeSet::empty()));
        assert!(!RangeSet::<u8>::empty().contains_all(&RangeSet::total()));

        assert!(!RangeSet(vec![2, 44]).contains_all(&RangeSet(vec![56, 60])));
        assert!(!RangeSet(vec![56, 60]).contains_all(&RangeSet(vec![2, 44])));
        assert!(RangeSet(vec![56, 60]).contains_all(&RangeSet(vec![56, 60])));
        assert!(RangeSet(vec![56, 61]).contains_all(&RangeSet(vec![56, 61])));
        assert!(RangeSet(vec![55, 61]).contains_all(&RangeSet(vec![56, 60])));

        assert!(RangeSet(vec![56, 60]).contains_all(&RangeSet(vec![57, 59])));
        assert!(RangeSet(vec![56, 60]).contains_all(&RangeSet(vec![57, 60])));
        assert!(RangeSet(vec![56, 60]).contains_all(&RangeSet(vec![56, 59])));

        assert!(RangeSet(vec![19, 33]).contains_all(&RangeSet(vec![20, 21, 30, 32])));
        assert!(
            RangeSet(vec![19, 33, 53, 70]).contains_all(&RangeSet(vec![20, 21, 30, 32, 66, 69]))
        );
        assert!(
            !RangeSet(vec![19, 33, 53, 70]).contains_all(&RangeSet(vec![20, 21, 30, 32, 66, 71]))
        );

        Ok(())
    }

    #[test]
    fn test_iter_and_contains() -> Result<(), String> {
        let empty = RangeSet::<u8>::empty();
        assert_eq!(0, empty.iter().count());
        assert!(!empty.contains(0));

        let total = RangeSet::<u8>::total();
        assert_eq!(256, total.iter().collect::<HashSet<_>>().len());
        assert!(total.iter().all(|v| total.contains(v)));

        let range = RangeSet(vec![19, 33, 53, 70]);
        assert_eq!(33, range.iter().collect::<HashSet<_>>().len()); // (33 - 19 + 1) + (77 - 53 + 1) = 33
        assert!(range.iter().all(|v| range.contains(v)));
        assert!(!range.contains(34));

        Ok(())
    }

    #[test]
    fn test_union() -> Result<(), String> {
        assert!(RangeSet::<u8>::empty().union(&RangeSet::empty()).is_empty());
        assert!(RangeSet::<u8>::total().union(&RangeSet::empty()).is_total());
        assert!(RangeSet::<u8>::empty().union(&RangeSet::total()).is_total());

        assert_eq!(
            vec![2, 44, 56, 60],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![56, 60])).0
        );

        assert_eq!(
            vec![0, 44],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![0, 3])).0
        );

        assert_eq!(
            vec![2, 48],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![5, 48])).0
        );

        assert_eq!(
            vec![2, 44],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![2, 44])).0
        );

        assert_eq!(
            vec![2, 44],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![5, 20])).0
        );

        assert_eq!(
            vec![2, 50],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![45, 50])).0
        );

        assert_eq!(
            vec![2, 44],
            RangeSet(vec![2, 44])
                .union(&RangeSet(vec![5, 20, 23, 40, 42, 43]))
                .0
        );

        assert_eq!(
            vec![1, 44],
            RangeSet(vec![2, 44]).union(&RangeSet(vec![1, 2, 4, 5])).0
        );

        assert_eq!(
            vec![0, 9, 11, 97],
            RangeSet(vec![0, 9, 11, 96])
                .union(&RangeSet(vec![97, 97]))
                .0
        );

        let range1 = RangeSet::new_from_ranges(&[
            AnyRange::from(..=12),
            AnyRange::from(15..=15),
            AnyRange::from(18..),
        ]);
        let range2 = RangeSet::new_from_ranges(&[AnyRange::from(..=12), AnyRange::from(15..)]);

        let range3 = RangeSet::new_from_ranges(&[AnyRange::from(..=12), AnyRange::from(15..)]);
        assert_eq!(range3, range1.union(&range2));

        Ok(())
    }

    #[test]
    fn test_intersection() -> Result<(), String> {
        assert!(RangeSet::<u32>::empty()
            .intersection(&RangeSet::empty())
            .is_empty());
        assert!(RangeSet::<u32>::total()
            .intersection(&RangeSet::empty())
            .is_empty());
        assert!(RangeSet::<u32>::empty()
            .intersection(&RangeSet::total())
            .is_empty());
        assert!(RangeSet::<u32>::total()
            .intersection(&RangeSet::total())
            .is_total());

        assert_eq!(
            RangeSet::empty(),
            RangeSet(vec![2, 44]).intersection(&RangeSet(vec![56, 60]))
        );

        assert_eq!(
            vec![2, 3],
            RangeSet(vec![2, 44]).intersection(&RangeSet(vec![0, 3])).0
        );

        assert_eq!(
            vec![5, 44],
            RangeSet(vec![2, 44]).intersection(&RangeSet(vec![5, 48])).0
        );

        assert_eq!(
            vec![2, 44],
            RangeSet(vec![2, 44]).intersection(&RangeSet(vec![2, 44])).0
        );

        assert_eq!(
            vec![5, 20],
            RangeSet(vec![2, 44]).intersection(&RangeSet(vec![5, 20])).0
        );
        assert_eq!(
            vec![5, 20, 23, 40],
            RangeSet(vec![2, 44])
                .intersection(&RangeSet(vec![5, 20, 23, 40]))
                .0
        );

        assert_eq!(
            vec![5, 20, 23, 40],
            RangeSet(vec![5, 20, 23, 40])
                .intersection(&RangeSet(vec![2, 44]))
                .0
        );

        assert!(RangeSet(vec![99, 99])
            .intersection(&RangeSet(vec![i32::MIN, 98, 100, i32::MAX]))
            .is_empty());
        Ok(())
    }

    #[test]
    fn test_complement() -> Result<(), String> {
        assert!(RangeSet::<u32>::total().complement().is_empty());
        assert!(RangeSet::<u32>::empty().complement().is_total());

        assert_eq!(
            vec![0, 1, 45, u32::MAX],
            RangeSet(vec![2, 44]).complement().0
        );

        assert_eq!(
            vec![45, i64::MAX],
            RangeSet(vec![i64::MIN, 44]).complement().0
        );

        assert_eq!(
            vec![45, 238],
            RangeSet(vec![0, 44, 239, u32::MAX]).complement().0
        );

        Ok(())
    }

    #[test]
    fn readme() -> Result<(), String> {
        let range1 =
            RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);

        let range2 = RangeSet::<i64>::new_from_range(-2..=4);

        let union = range1.union(&range2);
        println!("{union}"); // [ -2..=4 7..=8 ]
        for value in union.iter() {
            print!("{value} "); // -2 -1 0 1 2 3 4 7 8
        }
        println!();

        let intersection = range1.intersection(&range2);
        println!("{intersection}"); // [ 3..=4 ]
        for value in intersection.iter() {
            print!("{value} "); // 3 4
        }
        println!();

        let difference = range1.difference(&range2);
        println!("{difference}"); // [ 7..=8 ]
        for value in difference.iter() {
            print!("{value} "); // 7 8
        }
        println!();

        Ok(())
    }

    #[cfg(feature = "serde")]
    macro_rules! serde_test {
        ($($t:ty),*) => {
            $(
                let range = RangeSet::<$t>::empty();
                let serialized = serde_json::to_string(&range).unwrap();
                let unserialized: RangeSet<$t> = serde_json::from_str(&serialized).unwrap();
                assert_eq!(range, unserialized);

                let range = RangeSet::<$t>::total();
                let serialized = serde_json::to_string(&range).unwrap();
                let unserialized: RangeSet<$t> = serde_json::from_str(&serialized).unwrap();
                assert_eq!(range, unserialized);

                let range =
                    RangeSet::<$t>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);
                let serialized = serde_json::to_string(&range).unwrap();
                let unserialized: RangeSet<$t> = serde_json::from_str(&serialized).unwrap();
                assert_eq!(range, unserialized);
            )*
        };
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_test() {
        serde_test!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
    }

    /// A lower bound excluded from the range and equal to the maximum value: the range is empty.
    struct ExcludedMaxStart;
    impl RangeBounds<u8> for ExcludedMaxStart {
        fn start_bound(&self) -> Bound<&u8> {
            Bound::Excluded(&u8::MAX)
        }
        fn end_bound(&self) -> Bound<&u8> {
            Bound::Unbounded
        }
    }

    /// A range with both bounds excluded, e.g. `(3, 7)` -> `4..=6`.
    struct ExcludedBothEnds(u8, u8);
    impl RangeBounds<u8> for ExcludedBothEnds {
        fn start_bound(&self) -> Bound<&u8> {
            Bound::Excluded(&self.0)
        }
        fn end_bound(&self) -> Bound<&u8> {
            Bound::Excluded(&self.1)
        }
    }

    fn range_set_from_values(values: &[u8]) -> RangeSet<u8> {
        RangeSet::new_from_ranges(
            &values
                .iter()
                .map(|&value| AnyRange::from(value..=value))
                .collect::<Vec<_>>(),
        )
    }

    /// Assert the internal representation is canonical: an even number of bounds, no inverted
    /// range and no two ranges that overlap or merely touch each other.
    fn assert_canonical(range: &RangeSet<u8>) {
        assert_eq!(
            0,
            range.0.len() % 2,
            "odd number of bounds in {:?}",
            range.0
        );
        for bounds in range.0.chunks(2) {
            assert!(
                bounds[0] <= bounds[1],
                "inverted range {}..={} in {:?}",
                bounds[0],
                bounds[1],
                range.0
            );
        }
        for pair in range.0.chunks(2).collect::<Vec<_>>().windows(2) {
            assert!(
                u16::from(pair[1][0]) > u16::from(pair[0][1]) + 1,
                "ranges {}..={} and {}..={} should have been merged in {:?}",
                pair[0][0],
                pair[0][1],
                pair[1][0],
                pair[1][1],
                range.0
            );
        }
    }

    #[test]
    fn test_new_from_ranges_merges_adjacent_ranges() -> Result<(), String> {
        assert_eq!(
            RangeSet::<u8>::new_from_range(1..=4),
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(1..=2), AnyRange::from(3..=4)])
        );

        assert_eq!(
            vec![1, 4],
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(1..=2)]).0
        );

        // A chain of ranges that only touch each other collapses into a single one.
        assert_eq!(
            vec![0, 9],
            RangeSet::<u8>::new_from_ranges(&[
                AnyRange::from(0..=2),
                AnyRange::from(3..=3),
                AnyRange::from(4..8),
                AnyRange::from(8..=9)
            ])
            .0
        );

        // A range contained in the current one must not shrink it.
        assert_eq!(
            vec![0, 9],
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(0..=9), AnyRange::from(3..=4)]).0
        );

        // A one value gap is preserved.
        assert_eq!(
            vec![0, 2, 4, 9],
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(0..=2), AnyRange::from(4..=9)]).0
        );

        // Merging up to the maximum value must not overflow.
        assert_eq!(
            vec![250, 255],
            RangeSet::<u8>::new_from_ranges(&[
                AnyRange::from(250..=254),
                AnyRange::from(255..=255)
            ])
            .0
        );
        assert!(
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(..), AnyRange::from(200..=255)])
                .is_total()
        );

        Ok(())
    }

    #[test]
    fn test_complement_and_difference_with_adjacent_ranges() -> Result<(), String> {
        let range =
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(1..=2), AnyRange::from(3..=4)]);
        assert_canonical(&range);

        let complement = range.complement();
        assert_canonical(&complement);
        assert_eq!(vec![0, 0, 5, 255], complement.0);

        let range1 = RangeSet::<u8>::new_from_range(9..=12);
        let range2 =
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(11..=11), AnyRange::from(12..=17)]);
        let difference = range1.difference(&range2);
        assert_canonical(&difference);
        assert_eq!(vec![9, 10], difference.0);

        let range =
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(3..=6), AnyRange::from(7..=11)]);
        assert!(range.contains_all(&RangeSet::new_from_range(3..=8)));
        assert!(range.contains_all(&RangeSet::new_from_range(3..=11)));
        assert!(!range.contains_all(&RangeSet::new_from_range(3..=12)));

        Ok(())
    }

    #[test]
    fn test_new_from_empty_range() -> Result<(), String> {
        assert!(RangeSet::<u8>::new_from_range(5..5).is_empty());
        assert!(RangeSet::<u8>::new_from_range(0..0).is_empty());
        assert!(RangeSet::<u8>::new_from_range(..0).is_empty());
        assert!(RangeSet::<u8>::new_from_range(ExcludedMaxStart).is_empty());
        assert!(RangeSet::<i32>::new_from_range(i32::MIN..i32::MIN).is_empty());
        assert!(RangeSet::<i8>::new_from_range(..i8::MIN).is_empty());
        assert!(RangeSet::<usize>::new_from_range(0..0).is_empty());

        // An empty range must not be turned into the total range.
        assert!(!RangeSet::<u8>::new_from_range(0..0).is_total());
        assert!(!RangeSet::<i32>::new_from_range(i32::MIN..i32::MIN).is_total());

        // An empty range is simply ignored when building from several ranges.
        assert!(RangeSet::<u8>::new_from_ranges(&[AnyRange::from(0..0)]).is_empty());
        assert_eq!(
            RangeSet::<u8>::new_from_range(7..=9),
            RangeSet::<u8>::new_from_ranges(&[AnyRange::from(0..0), AnyRange::from(7..=9)])
        );

        // Excluded bounds that are not degenerate keep working.
        assert_eq!(
            vec![4, 6],
            RangeSet::<u8>::new_from_range(ExcludedBothEnds(3, 7)).0
        );
        assert_eq!(vec![0, 254], RangeSet::<u8>::new_from_range(..u8::MAX).0);

        Ok(())
    }

    #[test]
    fn test_malformed_range_set_does_not_panic() -> Result<(), String> {
        // A `RangeSet` can be built directly from its public field, so no operation may panic
        // on a malformed representation.
        for bounds in [vec![0u8], vec![1, 5, 9], vec![5, 3], vec![10, 20, 0, 5]] {
            let malformed = RangeSet::<u8>(bounds);
            let _ = malformed.is_total();
            let _ = malformed.is_empty();
            let _ = format!("{malformed}");
            let _ = malformed.contains(4);
            let _ = malformed.iter().take(512).count();
            let _ = malformed.complement();
            let _ = malformed.union(&RangeSet(vec![1, 2]));
            let _ = malformed.intersection(&RangeSet(vec![1, 2]));
            let _ = malformed.difference(&RangeSet(vec![1, 2]));
            let _ = malformed.has_intersection(&RangeSet(vec![1, 2]));
            let _ = malformed.contains_all(&RangeSet(vec![1, 2]));
            let _ = RangeSet(vec![1u8, 2]).contains_all(&malformed);
            let _ = RangeSet(vec![1u8, 2]).union(&malformed);
            let _ = RangeSet(vec![1u8, 2]).intersection(&malformed);
            let _ = RangeSet(vec![1u8, 2]).has_intersection(&malformed);
        }

        Ok(())
    }

    #[test]
    fn test_exhaustive_against_btree_set() -> Result<(), String> {
        const UNIVERSE: u8 = 7;

        let subsets: Vec<(BTreeSet<u8>, RangeSet<u8>)> = (0..(1u32 << UNIVERSE))
            .map(|mask| {
                let values: Vec<u8> = (0..UNIVERSE)
                    .filter(|value| mask & (1 << value) != 0)
                    .collect();
                (
                    values.iter().copied().collect(),
                    range_set_from_values(&values),
                )
            })
            .collect();

        for (values, range) in &subsets {
            assert_canonical(range);
            assert_eq!(*values, range.iter().collect::<BTreeSet<_>>());
            assert_eq!(values.is_empty(), range.is_empty());

            let complement = range.complement();
            assert_canonical(&complement);
            assert_eq!(
                (0..=u8::MAX)
                    .filter(|v| !values.contains(v))
                    .collect::<BTreeSet<_>>(),
                complement.iter().collect::<BTreeSet<_>>()
            );

            for value in 0..=UNIVERSE {
                assert_eq!(values.contains(&value), range.contains(value));
            }
        }

        for (values1, range1) in &subsets {
            for (values2, range2) in &subsets {
                let union = range1.union(range2);
                assert_canonical(&union);
                assert_eq!(
                    values1.union(values2).copied().collect::<BTreeSet<_>>(),
                    union.iter().collect::<BTreeSet<_>>(),
                    "{range1} union {range2}"
                );

                let intersection = range1.intersection(range2);
                assert_canonical(&intersection);
                assert_eq!(
                    values1
                        .intersection(values2)
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    intersection.iter().collect::<BTreeSet<_>>(),
                    "{range1} intersection {range2}"
                );

                let difference = range1.difference(range2);
                assert_canonical(&difference);
                assert_eq!(
                    values1
                        .difference(values2)
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    difference.iter().collect::<BTreeSet<_>>(),
                    "{range1} difference {range2}"
                );

                assert_eq!(
                    !values1.is_disjoint(values2),
                    range1.has_intersection(range2),
                    "{range1} has_intersection {range2}"
                );
                assert_eq!(
                    values2.is_subset(values1),
                    range1.contains_all(range2),
                    "{range1} contains_all {range2}"
                );

                // Equal sets must have equal representations, otherwise `Eq` and `Hash` lie.
                if values1 == values2 {
                    assert_eq!(range1, range2);
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_new_from_bounds_validation() -> Result<(), String> {
        assert_eq!(
            Some(RangeSet::<u8>::empty()),
            RangeSet::<u8>::new_from_bounds(vec![])
        );
        assert_eq!(
            Some(RangeSet::<u8>::new_from_range(2..=5)),
            RangeSet::<u8>::new_from_bounds(vec![2, 5])
        );
        assert_eq!(
            Some(RangeSet::<u8>::total()),
            RangeSet::<u8>::new_from_bounds(vec![0, 255])
        );
        assert_eq!(
            Some(RangeSet::<u8>(vec![2, 5, 7, 9])),
            RangeSet::<u8>::new_from_bounds(vec![2, 5, 7, 9])
        );

        // Odd number of bounds.
        assert_eq!(None, RangeSet::<u8>::new_from_bounds(vec![0]));
        assert_eq!(None, RangeSet::<u8>::new_from_bounds(vec![0, 5, 7]));
        // Inverted range.
        assert_eq!(None, RangeSet::<u8>::new_from_bounds(vec![5, 3]));
        // Unsorted ranges.
        assert_eq!(None, RangeSet::<u8>::new_from_bounds(vec![10, 20, 0, 5]));
        // Overlapping ranges.
        assert_eq!(None, RangeSet::<u8>::new_from_bounds(vec![0, 5, 4, 9]));
        // Adjacent ranges, they should have been merged.
        assert_eq!(None, RangeSet::<u8>::new_from_bounds(vec![0, 5, 6, 9]));
        // A range starting after the maximum value cannot exist.
        assert_eq!(
            None,
            RangeSet::<u8>::new_from_bounds(vec![0, 255, 255, 255])
        );

        Ok(())
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_rejects_malformed_range_set() {
        assert!(serde_json::from_str::<RangeSet<u8>>("[]").is_ok());
        assert!(serde_json::from_str::<RangeSet<u8>>("[0,5]").is_ok());
        assert!(serde_json::from_str::<RangeSet<u8>>("[0,5,7,9]").is_ok());

        // Odd number of bounds.
        assert!(serde_json::from_str::<RangeSet<u8>>("[0]").is_err());
        assert!(serde_json::from_str::<RangeSet<u8>>("[0,5,7]").is_err());
        // Inverted range.
        assert!(serde_json::from_str::<RangeSet<u8>>("[5,3]").is_err());
        // Unsorted ranges.
        assert!(serde_json::from_str::<RangeSet<u8>>("[10,20,0,5]").is_err());
        // Overlapping ranges.
        assert!(serde_json::from_str::<RangeSet<u8>>("[0,5,4,9]").is_err());
        // Adjacent ranges that should have been merged.
        assert!(serde_json::from_str::<RangeSet<u8>>("[0,5,6,9]").is_err());
    }
}
