# Artifact format

This page describes format version `0x02`, which laquna 0.2 produces and
accepts.

## Layout

| Offset | Length | Field | Notes |
|--------|--------|-------|-------|
| 0 | 1 | version byte | `0x02` |
| 1 | n | payload | The plaintext, compressed and then masked |
| 1 + n | 32 | slug | The slug passed to `encode` |

- The slug sits at the **tail**, so apart from the version byte the artifact has
  no fixed prefix.
- The structural minimum is **33 bytes**: a version byte, an empty payload and the
  slug. Real encoder output is always longer, because even an empty plaintext
  compresses to a non-empty frame. The 33-byte floor is a decoder pre-check, not
  a size the encoder can produce.
- Compression uses the standard zstd format at level 3, with the content-checksum
  frame flag set. The checksum lets the decoder catch most wrong-seed and
  corruption cases that would otherwise decompress to garbage.

## Encode

1. Reject an empty seed. The reference implementation panics.
2. Compress the plaintext.
3. Derive the mask from `(seed, slug)` and apply it to the compressed bytes. This
   produces the payload.
4. Emit `[0x02][payload][slug]`.

## Decode

`decode` runs its checks in this order and returns the first error it hits:

| Step | Check | Error |
|------|-------|-------|
| 1 | Seed is non-empty | `DecodeError::EmptySeed` |
| 2 | Length is at least 33 | `DecodeError::InputTooShort` |
| 3 | Version byte is `0x02` | `DecodeError::InvalidVersion(byte)` |
| 4 | Split off the tail slug, then rebuild and remove the mask | none |
| 5 | Decompress and verify the content checksum | `DecodeError::DecompressionFailed` |

Notes on the order and errors:

- The length check comes before the version check. A 32-byte buffer is reported
  as too short even if its first byte is `0x02`.
- An empty seed is reported before any inspection of the artifact. Even a
  malformed artifact returns `EmptySeed` if the seed is empty.
- A wrong seed, a corrupted slug or a corrupted payload almost always produces
  `DecompressionFailed`, and this variant carries no detail by design. A crafted
  artifact can still decode to different plaintext, because laquna provides no
  authentication.
- Decoders should keep these variants distinct rather than collapsing them into
  one opaque error.

## Versioning

- The version byte identifies the format. `0x01` was laquna 0.1's format. 0.2
  decoders reject it with `InvalidVersion(0x01)`.
- A future format will use a new version byte (`0x03`, `0x04`, and so on). Its
  release will either keep decode support for older versions or state
  explicitly that older artifacts must be migrated.
- A decoder that handles several versions must handle each one exactly as that
  version is defined.
