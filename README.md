# irange

[![Crates.io Version](https://img.shields.io/crates/v/irange)](https://crates.io/crates/irange)
[![docs.rs](https://img.shields.io/docsrs/irange)](https://docs.rs/irange)
[![Build status](https://img.shields.io/github/actions/workflow/status/alexvbrdn/irange/rust.yml?branch=main)](https://github.com/alexvbrdn/irange/actions/workflows/rust.yml)
[![License](https://img.shields.io/crates/l/irange)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.63%2B-blue)](#minimum-supported-rust-version)

A data structure to store and manipulate ranges of integers with set operations.

Supported types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` and `isize`.

## Installation

Add the following line in your `Cargo.toml`:

```toml
[dependencies]
irange = "1.2"
```

If you need `serde` support you can include the following feature flag:

```toml
[dependencies]
irange = { version = "1.2", features = ["serde"] }
```

## Minimum supported Rust version

`irange` builds with Rust 1.63 and later. Enabling the `serde` feature pulls in `serde_derive`,
whose own dependencies require a more recent compiler.

Raising this version is a breaking change and is only done in a minor release.

## Examples

```rust
use irange::range::AnyRange;
use irange::RangeSet;

let range1 = RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);
let range2 = RangeSet::<i64>::new_from_range(-2..=4);

let union = range1.union(&range2); // or `&range1 | &range2`
println!("{union}"); // [ -2..=4 7..=8 ]
for value in union.iter() {
    print!("{value} "); // -2 -1 0 1 2 3 4 7 8
}
println!();

let intersection = range1.intersection(&range2); // or `&range1 & &range2`
println!("{intersection}"); // [ 3..=4 ]
for value in intersection.iter() {
    print!("{value} "); // 3 4
}
println!();

let difference = range1.difference(&range2); // or `&range1 - &range2`
println!("{difference}"); // [ 7..=8 ]
for value in difference.iter() {
    print!("{value} "); // 7 8
}
println!();
```

## Supported Operations

| Operation | Operator | Description | Time complexity | Space complexity |
|---|---|---|---|---|
| `union` | `\|` | Compute the union with the given `RangeSet`. | `O(n)` | `O(n)` |
| `intersection` | `&` | Compute the intersection with the given `RangeSet`. | `O(n)` | `O(n)` |
| `difference` | `-` | Compute the difference with the given `RangeSet`. | `O(n)` | `O(n)` |
| `complement` | `!` | Compute the complement. | `O(n)` | `O(n)` |
| `has_intersection` | | Return `true` if there is a common value with the given `RangeSet`. | `O(n)` | `O(1)` |
| `contains` | | Return `true` if it contains the given value. | `O(log n)` | `O(1)` |
| `contains_all` | | Return `true` if it contains the given `RangeSet`. | `O(n)` | `O(1)` |
| `is_total` | | Return `true` if it contains all the possible values. | `O(1)` | `O(1)` |
| `is_empty` | | Return `true` if it does not contain any value. | `O(1)` | `O(1)` |

The operators take references, like those of `std::collections::BTreeSet`, so `&a | &b` is
the union of `a` and `b`.

## Iteration

`RangeSet::iter` walks the individual values in order, and `&RangeSet` implements
`IntoIterator`, so a `for` loop over a set works directly. The iterator is double-ended, so
`.rev()` walks the values from the largest down.

```rust
use irange::RangeSet;

let range = RangeSet::new_from_range(2..=5);

let ascending: Vec<i32> = (&range).into_iter().collect();
assert_eq!(vec![2, 3, 4, 5], ascending);

let descending: Vec<i32> = range.iter().rev().collect();
assert_eq!(vec![5, 4, 3, 2], descending);
```

## Documentation

The [API documentation](https://docs.rs/irange) covers every operation, the representation
`RangeSet` uses and the `serde` feature. `cargo run --example set_operations` runs a short
tour of the API.

Notable changes are recorded in [CHANGELOG.md](CHANGELOG.md).

## Contributing

Bug reports and pull requests are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the [MIT License](LICENSE).
