# Contributing to irange

Bug reports, questions and pull requests are all welcome.

## Reporting a bug

Open an issue with the smallest `RangeSet` that reproduces the problem, the operation you called, what you expected and what you got. `RangeSet` implements `Display` and `Debug`, so printing the sets involved is usually enough.

## Before opening a pull request

Run what CI runs:

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-targets
cargo test --all-features --all-targets
cargo test --all-features --doc
```

Please also:

- **Add a test.** Unit tests live in `src/lib.rs`; tests that drive the crate the way a dependent would live in `tests/`.
- **Document new public items.** `missing_docs` is warned on, and doc examples are run as tests, so an example in a doc comment is a test too.
- **Keep the MSRV.** The crate builds with Rust 1.63 and later, verified in CI. Raising it is a breaking change.
- **Update `CHANGELOG.md`** under an `## [Unreleased]` heading.

## The representation invariant

A `RangeSet` is a flat `Vec` of inclusive bounds, even indices holding lower bounds and odd indices the matching upper bounds. Every set has exactly one valid representation: an even number of bounds, no inverted range, and ranges that are sorted and separated by at least one value, so `1..=2` and `3..=4` are stored merged as `1..=4`.

Every constructor must produce that form, and every operation may assume it, except that none of them may panic on a malformed set, since the `Vec` is a public field and can be written to directly. `RangeSet::new_from_bounds` is the check, and it is what the `serde` deserializer uses.

## Benchmarks

`cargo bench` measures the set operations over the Unicode codepoints matching the `\w` and `\d` regular expression classes. Numbers move a lot with machine load, so compare runs on an otherwise idle machine before claiming a change is faster.

## Releasing

Publishing is automated. Push a `v*` tag matching the version in `Cargo.toml` and the release workflow checks formatting, clippy, tests, docs and a dry-run publish before it uploads to crates.io.
