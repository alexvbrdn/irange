# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-09-03

This release fixes three bugs that made `RangeSet` return wrong values rather than fail loudly. No API is removed and no code that compiled against 1.1.2 stops compiling, but results change where they used to be incorrect. See *Compatibility* below before upgrading.

### Fixed

- `new_from_ranges` now merges ranges that only touch each other: `[1..=2, 3..=4]` gives `[ 1..=4 ]` instead of `[ 1..=2 3..=4 ]`. `union` already merged adjacent ranges, so the two constructors disagreed and a set had more than one possible representation. Because of this:
  - `complement` could emit an inverted range (`{1,2} ∪ {3,4}` gave `[ 0..=0 3..=2 5..=255 ]`), and so could `difference`, which is built on it.
  - `contains_all` could return `false` for a set it does contain, e.g. `[ 3..=6 7..=11 ].contains_all(3..=8)`.
  - Two `RangeSet` holding the same values could compare unequal and hash differently.
- An empty range no longer produces the total range. `RangeSet::<u8>::new_from_range(0..0)` returned `[ 0..=255 ]` in release builds and panicked with `attempt to subtract with overflow` in debug builds; it now returns the empty set. The same applied to `..T::MIN` and to an excluded lower bound equal to `T::MAX`.
- No operation panics on a `RangeSet` built directly from its public field or deserialized from untrusted input. An odd number of bounds used to panic with an index out of bounds in `is_total`, `Display`, `union`, `intersection`, `has_intersection` and `contains_all`; an inverted range made `iter` overflow instead of terminating; and unsorted bounds could make `complement` underflow.

### Added

- `RangeSet::new_from_bounds(Vec<T>) -> Option<RangeSet<T>>`, to build a `RangeSet` from the raw inclusive bounds with validation. It returns `None` when the bounds are not canonical.
- `AnyRange::is_empty`, to tell whether a range holds no value.
- A `CHANGELOG.md`, this file.
- Crate-level documentation, so `docs.rs` has a landing page: it covers the canonical
representation, the complexity of every operation and the `serde` feature. `missing_docs` is
now warned on and every public item is documented.

### Changed

- With the `serde` feature, deserializing a `RangeSet` now validates the bounds and fails with a descriptive error instead of accepting a malformed representation. Serialization is unchanged.
- The canonical representation is now documented on the `RangeSet` tuple field: an even number of bounds, no inverted range, and ranges that are sorted and separated by at least one value. Building a `RangeSet` directly from the field bypasses the invariant and gives unspecified (but never panicking) results.
- Documented that `PartialOrd` and `Ord` compare the internal representation lexicographically. They give a total order, not the inclusion order.
- `contains` is documented as `O(log n)` in the README; it was listed as `O(n)` but has been a binary search since 1.1.0.

### Compatibility

- The minimum supported Rust version is unchanged.
- Deserializing data written by 1.1.2 fails if it holds adjacent ranges that 1.1.2 should have
merged, e.g. `[3,4,5,6]`. Such data was produced by the `new_from_ranges` bug above. Rebuild it with `new_from_ranges`, which now returns the canonical `[3,6]`.
- Code that relied on `new_from_ranges` keeping adjacent ranges apart, or that indexed the public field expecting a fixed number of ranges, sees different (correct) values.

## [1.1.2] - 2024-09-05

### Fixed

- Overflow in `union`.

## [1.1.1] - 2024-09-02

### Added

- `serde` support behind the `serde` feature flag.

## [1.1.0] - 2024-08-30

### Changed

- Improved performance and memory usage.

## [1.0.0] - 2024-08-25

Initial release.

[1.2.0]: https://github.com/alexvbrdn/irange/compare/v1.1.2...v1.2.0
[1.1.2]: https://github.com/alexvbrdn/irange/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/alexvbrdn/irange/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/alexvbrdn/irange/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/alexvbrdn/irange/releases/tag/v1.0.0
