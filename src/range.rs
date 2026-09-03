use std::ops::RangeBounds;

use crate::{integer::NumericInteger, range_to_bounds};

/// A range of integers, held as an inclusive `(min, max)` pair.
///
/// A range where `min > max` is empty and is ignored by `RangeSet::new_from_ranges`.
pub struct AnyRange<T: NumericInteger>((T, T));

impl<T: NumericInteger> AnyRange<T> {
    /// Create a range from the given inclusive bounds.
    ///
    /// The range is empty if `min > max`.
    pub fn new(min: T, max: T) -> AnyRange<T> {
        AnyRange((min, max))
    }

    /// Create a range from any Rust range, inclusive or not, bounded or not.
    ///
    /// The range is empty if it holds no value, e.g. `2..2` or `..T::MIN`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::range::AnyRange;
    ///
    /// // Holds 3, 4
    /// assert_eq!(&(3, 4), AnyRange::from(3..5).get_bounds());
    ///
    /// // Holds 0, 1, 2
    /// assert_eq!(&(0, 2), AnyRange::<u8>::from(..=2).get_bounds());
    ///
    /// assert!(AnyRange::from(2..2).is_empty());
    /// ```
    pub fn from<R: RangeBounds<T>>(range: R) -> AnyRange<T> {
        match range_to_bounds(&range) {
            Some(bounds) => AnyRange(bounds),
            None => AnyRange((T::max_value(), T::min_value())),
        }
    }

    /// Return the inclusive `(min, max)` bounds of the range.
    ///
    /// `min` is greater than `max` when the range is empty.
    pub fn get_bounds(&self) -> &(T, T) {
        &self.0
    }

    /// Return `true` if the range holds no value, i.e. if `min > max`.
    ///
    /// # Example:
    ///
    /// ```
    /// use irange::range::AnyRange;
    ///
    /// assert!(AnyRange::new(4, 3).is_empty());
    /// assert!(!AnyRange::new(3, 4).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        let (min, max) = self.0;
        min > max
    }
}
