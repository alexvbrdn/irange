use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

/// The integer types a [`RangeSet`](crate::RangeSet) can hold.
///
/// This trait is blanket-implemented for every type meeting its bounds, so it is
/// implemented for `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`,
/// `i64`, `i128` and `isize`, and there is nothing to implement by hand.
pub trait NumericInteger:
    Display + Copy + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + Bounded
{
}

impl<T> NumericInteger for T where
    T: Display + Copy + Ord + Add<Output = T> + Sub<Output = T> + AddAssign + Bounded
{
}

/// The constants a [`NumericInteger`] needs: its range and its unit step.
///
/// It is implemented for every supported integer type. `RangeSet` uses it to walk
/// between adjacent values and to describe the total set, so an implementation must
/// report the true bounds of the type.
pub trait Bounded {
    /// The smallest value the type can hold.
    fn min_value() -> Self;
    /// The largest value the type can hold.
    fn max_value() -> Self;
    /// The value `1`, i.e. the distance between two adjacent values.
    fn one() -> Self;
}

impl Bounded for u8 {
    fn min_value() -> Self {
        u8::MIN
    }

    fn max_value() -> Self {
        u8::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for u16 {
    fn min_value() -> Self {
        u16::MIN
    }

    fn max_value() -> Self {
        u16::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for u32 {
    fn min_value() -> Self {
        u32::MIN
    }

    fn max_value() -> Self {
        u32::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for u64 {
    fn min_value() -> Self {
        u64::MIN
    }

    fn max_value() -> Self {
        u64::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for u128 {
    fn min_value() -> Self {
        u128::MIN
    }

    fn max_value() -> Self {
        u128::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for usize {
    fn min_value() -> Self {
        usize::MIN
    }

    fn max_value() -> Self {
        usize::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for i8 {
    fn min_value() -> Self {
        i8::MIN
    }

    fn max_value() -> Self {
        i8::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for i16 {
    fn min_value() -> Self {
        i16::MIN
    }

    fn max_value() -> Self {
        i16::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for i32 {
    fn min_value() -> Self {
        i32::MIN
    }

    fn max_value() -> Self {
        i32::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for i64 {
    fn min_value() -> Self {
        i64::MIN
    }

    fn max_value() -> Self {
        i64::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for i128 {
    fn min_value() -> Self {
        i128::MIN
    }

    fn max_value() -> Self {
        i128::MAX
    }

    fn one() -> Self {
        1
    }
}

impl Bounded for isize {
    fn min_value() -> Self {
        isize::MIN
    }

    fn max_value() -> Self {
        isize::MAX
    }

    fn one() -> Self {
        1
    }
}
