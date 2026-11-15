# Laquna

Laquna is a deterministic, reversible byte transform for at-rest content
opacity. Given a plaintext byte sequence, an opaque `seed`, and a 32-byte
`slug`, it produces an artifact whose payload is opaque to casual inspection;
given that artifact and the same seed, it recovers the original plaintext.

> ### Laquna is not encryption.
>
> The decoder is in this repository. The slug travels inline in every artifact,
> and the seed is typically derived from public metadata, so any party with this
> code and the seed can recover the plaintext. Laquna provides **no
> confidentiality, no authentication, and no resistance to a motivated
> adversary**. Its only value is *friction* against opportunistic, at-scale
> content extraction. Consumers needing confidentiality, integrity, or
> tamper-detection must layer those on externally (authenticated encryption, a
> MAC, or signatures).
>
> Laquna provides friction against opportunistic and determined human
> adversaries. It does **not** defend against LLM-assisted adversaries that can
> call the public `decode()` API directly; consumers requiring resistance to
> LLM-assisted bulk extraction must compose Laquna with additional access
> controls in the consuming system.

## Usage

```rust
use laquna::{encode, decode};

// A 32-byte rotation-batch slug (generate from a CSRNG in production).
let slug = [0x5a_u8; 32];

// A per-record-unique seed. Any scheme with per-record uniqueness works.
let seed = b"did:plc:example||3kabcd2lqr7s2y||laquna/v0.2";

let artifact = encode(b"a private post", seed, &slug);
let recovered = decode(&artifact, seed).expect("decodes under the same seed");
assert_eq!(recovered, b"a private post");
```

The seed **must be unique per record within a slug**. Decoding returns a
`DecodeError` for malformed input, a wrong seed, or corruption; an empty seed
panics on encode and returns `DecodeError::EmptySeed` on decode. See the crate
rustdoc (`cargo doc --open`) for the full API contract.

The encode path requires `std`; the decode path is `no_std`-compatible with the
`alloc` feature. A `no_std` consumer can decode but not encode.

## Documentation

- [Design](https://github.com/skydeval/laquna/blob/main/docs/design.md): purpose,
  threat model, non-goals, and seed and slug requirements.
- [Format](https://github.com/skydeval/laquna/blob/main/docs/format.md): artifact
  byte layout, decode validation order, and versioning.
- [Conformance](https://github.com/skydeval/laquna/blob/main/docs/conformance.md):
  reference vectors and what a conforming implementation must do.
- [Changelog](https://github.com/skydeval/laquna/blob/main/CHANGELOG.md)

## License

MPL-2.0. See [LICENSE](LICENSE).
