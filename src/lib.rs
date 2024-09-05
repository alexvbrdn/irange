#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

use std::ops::{Bound, RangeBounds};

use integer::NumericInteger;
use range::AnyRange;

pub mod integer;
pub mod range;

fn range_to_bounds<T: NumericInteger, R: RangeBounds<T>>(range: &R) -> (T, T) {
    let min = match range.start_bound() {
        Bound::Included(t) => *t,
        Bound::Excluded(t) => *t + T::one(),
        Bound::Unbounded => T::min_value(),
    };
    let max = match range.end_bound() {
        Bound::Included(t) => *t,
        Bound::Excluded(t) => *t - T::one(),
        Bound::Unbounded => T::max_value(),
    };

    (min, max)
}

/// A structure holding a collection of `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` or `isize`.
#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangeSet<T: NumericInteger>(
    /// In this collection all the elements with even index represent the lower bounds (inclusive) and all the odd index represent the upper bounds (inclusive).
    pub Vec<T>,
);

impl<T: NumericInteger> std::fmt::Display for RangeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[ ")?;
        for i in (0..self.0.len()).step_by(2) {
            let (min, max) = (self.0[i], self.0[i + 1]);
            write!(f, "{}..={} ", min, max)?;
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
                if value == *max {
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
    pub fn iter(&self) -> RangeSetIter<T> {
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
        let (min, max) = range_to_bounds(&range);
        if max >= min {
            RangeSet(vec![min, max])
        } else {
            RangeSet::empty()
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
        ranges.sort_by(|r1, r2| r1.0.cmp(&r2.0));

        let mut bounds = Vec::with_capacity(ranges.len() * 2);
        let mut current_max = T::min_value();
        for (min, max) in ranges {
            if bounds.is_empty() || min > current_max {
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
        !self.0.is_empty() && self.0[0] == T::min_value() && self.0[1] >= T::max_value()
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

        while that_i < that.0.len() {
            if self_i == self.0.len() {
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

        while self_i < self.0.len() || that_i < that.0.len() {
            if that_i < that.0.len() && (self_i >= self.0.len() || self.0[self_i] > that.0[that_i])
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

        while i < self.0.len() && j < that.0.len() {
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

        while i < self.0.len() && j < that.0.len() {
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

        for i in (0..self.0.len()).step_by(2) {
            let (min, max) = (self.0[i], self.0[i + 1]);

            if new_range.is_empty() && min != T::min_value() {
                new_range.push(T::min_value());
                new_range.push(min - T::one());
            }

            if new_range.len() % 2 == 1 {
                new_range.push(min - T::one());
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
    use std::collections::HashSet;

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
}
