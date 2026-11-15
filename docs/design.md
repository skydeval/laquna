# Design

## What laquna is

laquna is a deterministic, reversible byte transform. Given a plaintext, an
opaque `seed` and a 32-byte `slug`, `encode` produces an artifact whose payload
is opaque to casual inspection: it contains no plaintext bytes, no recognizable
inner header and no standard encoding patterns. Given that artifact and the same
seed, `decode` recovers the plaintext.

laquna was built as the default at-rest content module for kryphocron, which
applies it to private records. The crate has no dependency on kryphocron and can be used on its own.

## What laquna is not

**laquna is not encryption.** The decoder is public, the slug travels inline in
every artifact, and the seed is typically derived from public record metadata.
Anyone with this code and the seed can recover the plaintext.

laquna's value is **friction**. Extracting content at scale takes more than
pattern-matching against standard encodings: an adversary has to run this
decoder with the right seed for every record.

## Threat model

laquna raises the cost of opportunistic, at-scale extraction of at-rest content.
An example is a scraper harvesting records from a data store it can read.

laquna does **not** defend against:

- **A motivated adversary.** Anyone willing to run or reimplement the decoder can
  decode any artifact whose seed they can construct.
- **LLM-assisted bulk extraction.** Anyone able to call the public `decode` API
  directly gains nothing from the transform's internals being unfamiliar.
  Consumers that need resistance here must add access controls in the consuming
  system.
- **The operator.** The encoding system holds plaintext during encode and
  during rotation re-encoding.

## Non-goals

laquna deliberately does **not** provide:

- **Confidentiality.**
- **Authentication or tamper detection.** Decoding usually fails on a wrong seed
  or corrupted input, but a crafted artifact may decode to different plaintext.
  laquna does not guarantee that all corruption is detected.
- **Indistinguishability.** The version byte and the tail slug identify an
  artifact as laquna output.
- **Statistical randomness of the payload.** The payload is opaque to casual
  inspection. laquna makes no claim that it passes tests against uniform random
  bytes.
- **Forward secrecy.** Rotating the slug does not protect artifacts produced
  before the rotation.
- **Long-term secrecy.** Once a decoder exists anywhere, every artifact whose
  seed can be constructed is recoverable.

Consumers that need any of these properties must layer them on externally:
authenticated encryption, a MAC or signatures, or a different module entirely.

## Seed requirements

The seed is opaque bytes. laquna does not interpret it or validate its length,
except to reject an empty seed.

- **Unique per record within a slug (required).** All masking material comes
  from `(seed, slug)`. Two records that share both seed and slug are masked
  identically. An observer holding both artifacts can then combine them to
  cancel the mask and attack the two plaintexts together. The consuming system
  is responsible for uniqueness.
- **Reproducible at decode time.** A record whose seed can't be rebuilt can't be
  recovered. If a seed input can change over a record's lifetime, such as an
  identifier migrating, the consumer must re-encode the affected records.
- **Non-empty.** `encode` panics on an empty seed, which is a caller bug.
  `decode` returns `DecodeError::EmptySeed`, because decode handles untrusted
  input.

kryphocron derives the seed from record identifiers (the originator, the
collection and the record key), which makes it unique per record.

## Slug requirements

The slug is exactly 32 bytes, and any 32-byte value is valid. It is embedded
inline in every artifact and is **not secret**.

- The slug is the **rotation lever**. To rotate, the consuming system picks a new
  slug and re-encodes records under it.
- Slugs should come from a cryptographically secure random source. laquna never
  generates slugs; managing them is the consuming system's job.
- A slug must not be reused across distinct rotation batches in the same system.

## Determinism

Encoding is deterministic: identical `(plaintext, seed, slug)` inputs produce
byte-identical artifacts. As a result, equal plaintexts under equal seed and slug
are visibly equal. Consumers that need to hide plaintext equality must make seeds
unique at the granularity at which equality should be hidden.

## Build profiles

| Feature | Default | Enables |
|---------|---------|---------|
| `alloc` | yes | The decode path and `Vec<u8>` returns. `no_std`-compatible. |
| `std` | yes | The encode path. Implies `alloc`. |
| `testing-internals` | no | Internal state for regenerating debugging vectors. Not public API; do not enable in production. |

A `no_std` consumer can decode but not encode.

## See also

- [format.md](format.md): artifact byte layout, decode validation order and
  versioning.
- [conformance.md](conformance.md): reference vectors and conformance
  requirements.
