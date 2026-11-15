# Conformance

## Requirements

A conforming implementation must:

- decode every Schema A artifact, under its listed seed, to the listed plaintext
- return the listed error variant for every Schema B vector
- round-trip its own output: anything it encodes must decode back to the
  original plaintext under the same seed
- meet the seed and slug requirements in [design.md](design.md)

A conforming implementation does **not** need to produce artifacts byte-identical
to the reference implementation. Compressor output varies between
implementations, and only decoding is fixed. The masking step is deterministic
for a given `(seed, slug)` and compressed input.

## Reference vectors

The canonical vectors are in
[`laquna/tests/vectors.json`](../laquna/tests/vectors.json). The conformance
suite ([`laquna/tests/conformance.rs`](../laquna/tests/conformance.rs)) runs
against them. All byte fields are hex-encoded.

### Schema A: round-trip vectors

Fields: `name`, `description`, `plaintext_hex`, `seed_hex`, `slug_hex`,
`artifact_hex`.

Coverage:
- empty plaintext and a single byte
- short ASCII
- 64 bytes of `0x00` and 64 bytes of `0xFF`
- a ~600-byte post
- a 64 KiB stress input
- a plaintext containing the slug bytes inline
- distinct-slug and distinct-seed variants of the same plaintext, whose artifacts
  must differ

Schema A artifacts are **decode targets**, not encode targets.

### Schema B: decode-error vectors

Fields: `name`, `description`, `artifact_hex`, `seed_hex`, and `expected`, which
is `{ "error": <variant> }` plus `version` for `InvalidVersion`.

| Cases | Expected error |
|-------|----------------|
| Lengths 0, 1, 16, 32 | `InputTooShort` |
| Version bytes `0x00`, `0x01`, `0x03`, `0xFF` | `InvalidVersion(n)` |
| A valid artifact with an empty seed | `EmptySeed` |
| A valid artifact with a mismatched seed | `DecompressionFailed` |
| A valid artifact with a corrupted slug | `DecompressionFailed` |
| A valid artifact with one payload bit flipped | `DecompressionFailed` |

### Schema C: debugging vectors

These cover component-level internal state for two `(seed, slug)` pairs whose
internal parameters differ. They help an implementer find which stage diverges.
They are **not** required for conformance.

## Regenerating the vectors

From the `laquna/` crate directory:

```sh
cargo run --example gen_vectors --features testing-internals
```

The generator rewrites `tests/vectors.json` and prints a human-readable Markdown
listing to stdout. Before emitting any vector, it checks that:

- encoding is deterministic
- every Schema A artifact round-trips
- every Schema B case produces its listed error

Regeneration is deterministic.
