use std::ops::RangeBounds;

use crate::{integer::NumericInteger, range_to_bounds};

pub struct AnyRange<T: NumericInteger>((T, T));

impl<T: NumericInteger> AnyRange<T> {
    pub fn new(min: T, max: T) -> AnyRange<T> {
        AnyRange((min, max))
    }
    
    pub fn from<R: RangeBounds<T>>(range: R) -> AnyRange<T> {
        AnyRange(range_to_bounds(&range))
    }

    pub fn get_bounds(&self) -> &(T, T) {
        &self.0
    }
}
