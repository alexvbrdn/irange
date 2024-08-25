use std::ops::{
    Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use crate::{integer::NumericInteger, range_to_bounds};

pub enum AnyRange<T: NumericInteger> {
    Range(Range<T>),
    RangeInclusive(RangeInclusive<T>),
    RangeFrom(RangeFrom<T>),
    RangeTo(RangeTo<T>),
    RangeToInclusive(RangeToInclusive<T>),
    RangeFull,
}

impl<T: NumericInteger> AnyRange<T> {
    pub fn from<R: RangeBounds<T> + Into<AnyRange<T>>>(range: R) -> AnyRange<T> {
        range.into()
    }

    pub fn get_bounds(&self) -> (T, T) {
        match self {
            AnyRange::Range(r) => range_to_bounds(r),
            AnyRange::RangeInclusive(r) => range_to_bounds(r),
            AnyRange::RangeFrom(r) => range_to_bounds(r),
            AnyRange::RangeTo(r) => range_to_bounds(r),
            AnyRange::RangeToInclusive(r) => range_to_bounds(r),
            AnyRange::RangeFull => (T::min_value(), T::max_value()),
        }
    }
}

impl<T: NumericInteger> From<Range<T>> for AnyRange<T> {
    fn from(range: Range<T>) -> Self {
        AnyRange::Range(range)
    }
}

impl<T: NumericInteger> From<RangeInclusive<T>> for AnyRange<T> {
    fn from(range: RangeInclusive<T>) -> Self {
        AnyRange::RangeInclusive(range)
    }
}

impl<T: NumericInteger> From<RangeFrom<T>> for AnyRange<T> {
    fn from(range: RangeFrom<T>) -> Self {
        AnyRange::RangeFrom(range)
    }
}

impl<T: NumericInteger> From<RangeTo<T>> for AnyRange<T> {
    fn from(range: RangeTo<T>) -> Self {
        AnyRange::RangeTo(range)
    }
}

impl<T: NumericInteger> From<RangeToInclusive<T>> for AnyRange<T> {
    fn from(range: RangeToInclusive<T>) -> Self {
        AnyRange::RangeToInclusive(range)
    }
}

impl<T: NumericInteger> From<RangeFull> for AnyRange<T> {
    fn from(_range: RangeFull) -> Self {
        AnyRange::RangeFull
    }
}
