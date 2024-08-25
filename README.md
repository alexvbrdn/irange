# irange

[![Crates.io Version](https://img.shields.io/crates/v/irange)](https://crates.io/crates/irange)

A data structure to store and manipulate ranges of integers with set operations.

Supported types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` and `isize`.

## Examples

```rust
let range1 = RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);
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
```

## Supported Operations

- `union`
- `intersection`
- `difference`
- `complement`
- `has_intersection`
- `contains`
- `contains_all`
- `is_total`
- `is_empty`
