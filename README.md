# irange

[![Crates.io Version](https://img.shields.io/crates/v/irange)](https://crates.io/crates/irange)

A data structure to store and manipulate ranges of integers with set operations.

Supported types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` and `isize`.

## Installation

Add the following line in your `Cargo.toml`:

```toml
[dependencies]
irange = "1.1"
```

If you need `serde` support you can include the following feature flag:

```toml
[dependencies]
irange = { version = "1.1", features = ["serde"] }
```

## Examples

```rust
use irange::RangeSet;

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

| Operation | Time complexity | Space complexity |
|---|---|---|
| `union` | `O(n)` | `O(n)` |
| `intersection` | `O(n)` | `O(n)` |
| `difference` | `O(n)` | `O(n)` |
| `complement` | `O(n)` | `O(n)` |
| `has_intersection` | `O(n)` | `O(1)` |
| `contains` | `O(n)` | `O(1)` |
| `contains_all` | `O(n)` | `O(1)` |
| `is_total` | `O(1)` | `O(1)` |
| `is_empty` | `O(1)` | `O(1)` |
