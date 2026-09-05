# irange

[![Crates.io Version](https://img.shields.io/crates/v/irange)](https://crates.io/crates/irange)
[![docs.rs](https://img.shields.io/docsrs/irange)](https://docs.rs/irange)
[![Build status](https://img.shields.io/github/actions/workflow/status/alexvbrdn/irange/rust.yml?branch=main)](https://github.com/alexvbrdn/irange/actions/workflows/rust.yml)
[![License](https://img.shields.io/crates/l/irange)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.63%2B-blue)](#minimum-supported-rust-version)

A data structure to store and manipulate ranges of integers with set operations.

A `RangeSet` holds an arbitrary set of integers as a sorted collection of non-overlapping inclusive ranges, so a set whose values come in runs costs a handful of bounds rather than one entry per value.

Supported types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` and `isize`.

## Installation

```toml
[dependencies]
irange = "1.2"
```

The optional `serde` feature implements `Serialize` and `Deserialize` for `RangeSet`:

```toml
[dependencies]
irange = { version = "1.2", features = ["serde"] }
```

## Example

```rust
use irange::range::AnyRange;
use irange::RangeSet;

let a = RangeSet::<i64>::new_from_ranges(&[AnyRange::from(3..=4), AnyRange::from(7..9)]);
let b = RangeSet::<i64>::new_from_range(-2..=4);

let union = &a | &b;
assert_eq!("[ -2..=4 7..=8 ]", union.to_string());
assert_eq!("[ 3..=4 ]", (&a & &b).to_string());
assert_eq!("[ 7..=8 ]", (&a - &b).to_string());

// `&RangeSet` is `IntoIterator`, and the iterator is double-ended.
let values: Vec<i64> = union.iter().collect();
assert_eq!(vec![-2, -1, 0, 1, 2, 3, 4, 7, 8], values);

let descending: Vec<i64> = union.iter().rev().collect();
assert_eq!(vec![8, 7, 4, 3, 2, 1, 0, -1, -2], descending);
```

## Operations

`n` is the total number of ranges involved. The operators take references, like those of `std::collections::BTreeSet`, so `&a | &b` is the union of `a` and `b`.

| Operation | Operator | Description | Time | Space |
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

See the [API documentation](https://docs.rs/irange) for the full list, and [CHANGELOG.md](CHANGELOG.md) for notable changes.

## Minimum supported Rust version

`irange` builds with Rust 1.63 and later. Raising this version is a breaking change.

Enabling the `serde` feature pulls in `serde_derive`, whose own dependencies require a more recent compiler.

## Contributing

Bug reports and pull requests are welcome, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the [MIT License](LICENSE).
