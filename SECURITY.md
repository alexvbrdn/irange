# Security Policy

## Supported versions

Fixes are published for the latest release of `irange`.

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub's
[security advisory form](https://github.com/alexvbrdn/irange/security/advisories/new)
rather than in a public issue.

## Scope

`irange` contains no `unsafe` code — the crate is `#![forbid(unsafe_code)]` — and performs
no I/O, so the realistic concerns are:

- A panic or a non-terminating loop reachable from a `RangeSet` built from untrusted input.
  Deserializing with the `serde` feature validates its input and rejects a malformed set,
  and no operation may panic even on a set written directly through the public field. A
  counterexample to either is a bug worth reporting.
- An operation returning a set that does not hold the right values.
