# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-11-15

### Added

- `DecodeError::EmptySeed`. `decode` returns it for an empty seed, before any
  other validation. `encode` still panics on an empty seed.
- Reference vectors in `tests/vectors.json` and a conformance suite that runs
  against them:
  - Schema A: round-trip vectors.
  - Schema B: decode-error vectors.
  - Schema C: component-level debugging vectors.
- A `testing-internals` feature, off by default. It exposes the internal state
  needed to regenerate and cross-check Schema C vectors. It is not part of the
  public API, and production builds must not enable it.

### Changed

- **Breaking:** a new masking construction, with the format version byte bumped
  from `0x01` to `0x02`. 0.2 decoders reject 0.1 artifacts, and records encoded
  with 0.1 must be re-encoded.
- The signatures of `encode` and `decode` are unchanged.

## [0.1.0] - 2026-06-10

### Added

- Initial release with `encode`, `decode` and `DecodeError`.
- Artifact format `[0x01][payload][32-byte slug]`.
- A `no_std` decode path behind the `alloc` feature. Encode requires `std`.
- Reference vectors and a conformance suite.

[Unreleased]: https://github.com/skydeval/laquna/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/skydeval/laquna/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/skydeval/laquna/releases/tag/v0.1.0
