# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - Unreleased

This release fixes three bugs that made `RangeSet` return wrong values rather than fail loudly, and makes most of the set operations faster. No API is removed and no code that compiled against 1.1.2 stops compiling, but results change where they used to be incorrect. See *Compatibility* below before upgrading.

### Fixed

- `new_from_ranges` now merges ranges that only touch each other: `[1..=2, 3..=4]` gives `[ 1..=4 ]` instead of `[ 1..=2 3..=4 ]`. `union` already merged adjacent ranges, so the two constructors disagreed and a set had more than one possible representation. Because of this:
  - `complement` could emit an inverted range (`{1,2} ∪ {3,4}` gave `[ 0..=0 3..=2 5..=255 ]`), and so could `difference`, which was built on it.
  - `contains_all` could return `false` for a set it does contain, e.g. `[ 3..=6 7..=11 ].contains_all(3..=8)`.
  - Two `RangeSet` holding the same values could compare unequal and hash differently.
- An empty range no longer produces the total range. `RangeSet::<u8>::new_from_range(0..0)` returned `[ 0..=255 ]` in release builds and panicked with `attempt to subtract with overflow` in debug builds; it now returns the empty set. The same applied to `..T::MIN` and to an excluded lower bound equal to `T::MAX`.
- No operation panics on a `RangeSet` built directly from its public field or deserialized from untrusted input. An odd number of bounds used to panic with an index out of bounds in `is_total`, `Display`, `union`, `intersection`, `has_intersection` and `contains_all`; an inverted range made `iter` overflow instead of terminating; and unsorted bounds could make `complement` underflow.

### Added

- `RangeSet::new_from_bounds(Vec<T>) -> Option<RangeSet<T>>`, to build a `RangeSet` from the raw inclusive bounds with validation. It returns `None` when the bounds are not canonical.
- `AnyRange::is_empty`, to tell whether a range holds no value.
- A `CHANGELOG.md`, this file.
- Crate-level documentation, so `docs.rs` has a landing page: it covers the canonical representation, the complexity of every operation and the `serde` feature. `missing_docs` is now warned on and every public item is documented.
- A declared minimum supported Rust version, 1.63, in the `rust-version` field and in the README. It was never declared before, so this documents the requirement rather than raising it.
- The set operations that build a new set now also have an operator, taking references like those of `BTreeSet`: `|` for `union`, `&` for `intersection`, `-` for `difference` and `!` for `complement`. The named methods are unchanged and are what the operators call.
- `RangeSetIter` is now a `DoubleEndedIterator` and a `FusedIterator`, so `rev` walks a set from its largest value down. It also derives `Clone` and `Debug`.
- `IntoIterator for &RangeSet<T>`, so a `for` loop can run over a set directly rather than over `set.iter()`.
- `FromIterator<AnyRange<T>> for RangeSet<T>`, so an iterator of ranges can be `collect`ed into a set.
- `Default for RangeSet<T>`, returning the empty set.
- `AnyRange` derives `PartialEq`, `Eq`, `Hash`, `Clone`, `Copy`, `Debug`, `PartialOrd` and `Ord`. It previously derived nothing, so a range could not even be printed with `{:?}`.
- Integration tests under `tests/`, which drive the crate through its public API the way a dependent crate does, including a randomised cross-check of the double-ended iterator.
- The README example is compiled and run as a doctest, so it cannot drift from the API.
- `CONTRIBUTING.md`, `SECURITY.md`, a pull request template, and Dependabot updates.

### Changed

- Faster set operations. The results are the same, only the implementations changed. Measured with `cargo bench`, using the two sets of Unicode codepoints matching the `\w` and `\d` regular expression classes:
  - `contains_all` is at least 35% faster. It rejects immediately when the argument reaches outside the span of the receiver, and its scan reads fewer bounds per step.
  - `complement` is about 40% faster. The complement of a set is the gaps between its ranges, so it is now read straight off the bounds rather than assembled one bound at a time.
  - `difference` is about 10% faster. It cuts the receiver against the argument in one pass, where it used to allocate the whole complement of the argument and intersect with it.
  - `union` is about 10% faster. The range being built is held in a local instead of being written to the output and then updated in place.
  - `new_from_ranges` no longer sorts ranges that already arrive in order, which is about 20% faster for such input and a few percent faster otherwise.
- `iter` looks up fewer bounds per value. This did not show up as a reliable difference in the benchmark.
- With the `serde` feature, deserializing a `RangeSet` now validates the bounds and fails with a descriptive error instead of accepting a malformed representation. Serialization is unchanged.
- The canonical representation is now documented on the `RangeSet` tuple field: an even number of bounds, no inverted range, and ranges that are sorted and separated by at least one value. Building a `RangeSet` directly from the field bypasses the invariant and gives unspecified (but never panicking) results.
- Documented that `PartialOrd` and `Ord` compare the internal representation lexicographically. They give a total order, not the inclusion order.
- `contains` is documented as `O(log n)` in the README; it was listed as `O(n)` but has been a binary search since 1.1.0.
- The crate is now `#![forbid(unsafe_code)]`. It never contained any `unsafe`; this makes that checkable.
- `Cargo.toml` declares `categories`, `homepage` and `documentation`, and switches from `exclude` to an `include` allowlist so a new file at the root cannot end up in the published package by accident. The `keywords` are now `range`, `interval`, `set`, `intersection` and `union`; `difference` and `complement` were dropped as nobody searches for them.
- CI now checks formatting, runs clippy with `-D warnings`, tests each feature combination, tests a 32-bit target, builds the documentation with `-D warnings`, dry-runs the publish, and verifies the declared 1.63 MSRV. Previously none of these were enforced outside the release workflow.
- `Cargo.lock` is no longer tracked. The crate is a library, so dependents pick their own versions, and the current lockfile format cannot be read by the 1.63 Cargo the MSRV names.
- The benchmark file is renamed from `benches/my_benchmark.rs` to `benches/set_ops.rs`.

### Compatibility

- The minimum supported Rust version is now declared as 1.63. No new compiler is required; the version is documented rather than raised.
- Deserializing data written by 1.1.2 fails if it holds adjacent ranges that 1.1.2 should have merged, e.g. `[3,4,5,6]`. Such data was produced by the `new_from_ranges` bug above. Rebuild it with `new_from_ranges`, which now returns the canonical `[3,6]`.
- Code that relied on `new_from_ranges` keeping adjacent ranges apart, or that indexed the public field expecting a fixed number of ranges, sees different (correct) values.
- With the `serde` feature, `irange::Serialize` and `irange::Deserialize` no longer exist. 1.1.2 re-exported serde's traits from the crate root by accident, which made them part of this crate's public API. Import them from `serde` instead.

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
