//! Decode error semantics and empty-seed handling.

mod common;

use common::{SEED, SLUG};
use laquna::{decode, encode, DecodeError};

const PLAINTEXT: &[u8] = b"some private post content of a reasonable, compressible length";

/// Artifacts below the 33-byte floor are rejected before the version is read.
#[test]
fn too_short_lengths() {
    for len in [0usize, 1, 16, 32] {
        let buf = vec![0x02u8; len];
        assert_eq!(
            decode(&buf, SEED),
            Err(DecodeError::InputTooShort),
            "length {len} must be InputTooShort"
        );
    }
}

/// Length check precedes the version check: a 32-byte buffer with a valid-looking
/// version byte is still InputTooShort.
#[test]
fn length_check_precedes_version_check() {
    let buf = vec![0x02u8; 32];
    assert_eq!(decode(&buf, SEED), Err(DecodeError::InputTooShort));
}

/// A long-enough artifact with an unsupported version byte reports InvalidVersion(N).
/// 0x01 is the previous format version and is now rejected.
#[test]
fn invalid_version_bytes() {
    for v in [0x00u8, 0x01, 0x03, 0xff] {
        let mut buf = vec![0u8; 40];
        buf[0] = v;
        assert_eq!(
            decode(&buf, SEED),
            Err(DecodeError::InvalidVersion(v)),
            "version 0x{v:02x} must be InvalidVersion"
        );
    }
}

/// Decoding a valid artifact under the wrong seed yields bytes that fail validation.
#[test]
fn wrong_seed_fails_decompression() {
    let artifact = encode(PLAINTEXT, SEED, &SLUG);
    let wrong_seed: &[u8] = b"a-deliberately-mismatched-seed-value";
    assert_eq!(
        decode(&artifact, wrong_seed),
        Err(DecodeError::DecompressionFailed)
    );
}

/// Corrupting the tail slug changes the derived materials, so recovery fails.
#[test]
fn corrupted_slug_fails_decompression() {
    let mut artifact = encode(PLAINTEXT, SEED, &SLUG);
    let last = artifact.len() - 1;
    artifact[last] ^= 0xff;
    assert_eq!(decode(&artifact, SEED), Err(DecodeError::DecompressionFailed));
}

/// Flipping a single bit in the payload is caught by frame validation or, failing that,
/// by content-checksum verification.
#[test]
fn single_bit_flip_payload_fails() {
    let mut artifact = encode(PLAINTEXT, SEED, &SLUG);
    // Pick an index inside the payload region [1 .. len - 32].
    let idx = 1 + (artifact.len() - 33) / 2;
    artifact[idx] ^= 0x01;
    assert_eq!(decode(&artifact, SEED), Err(DecodeError::DecompressionFailed));
}

/// An empty seed is a caller-precondition violation and panics on encode.
#[test]
#[should_panic(expected = "seed must be non-empty")]
fn encode_empty_seed_panics() {
    let _ = encode(b"x", b"", &SLUG);
}

/// On decode, which operates on untrusted input, an empty seed is returned as an error
/// rather than panicking — and it is checked before any structural inspection.
#[test]
fn decode_empty_seed_returns_error() {
    let artifact = encode(PLAINTEXT, SEED, &SLUG);
    assert_eq!(decode(&artifact, b""), Err(DecodeError::EmptySeed));
    // Checked before length/version: even a malformed artifact reports EmptySeed.
    assert_eq!(decode(&[0u8; 4], b""), Err(DecodeError::EmptySeed));
}
