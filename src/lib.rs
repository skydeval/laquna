#![no_std]

//! laquna: a deterministic, reversible byte transform.
//!
//! Given a `plaintext`, an opaque `seed`, and a 32-byte `slug`, [`encode`] produces
//! an artifact whose payload is statistically indistinguishable from uniformly random
//! bytes; given that artifact and the same `seed`, [`decode`] recovers the plaintext.
//!
//! **laquna is not encryption.** The algorithm is documented, the slug is embedded
//! inline in every artifact, and any party with the spec can decode any artifact given
//! the seed. laquna's value is friction against opportunistic, at-scale content
//! extraction — not confidentiality, authentication, or resistance to a motivated
//! adversary. See `SPEC.md` and `DESIGN.md` for the full framing.
//!
//! The pipeline (SPEC §4 / §6):
//!
//! ```text
//! encode: plaintext -> zstd compress (level 3, content checksum)
//!                   -> XOR ChaCha20 keystream (key = HKDF-SHA256(seed, slug))
//!                   -> [0x01][payload][slug]
//! decode: reverse
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Version byte of a laquna v0.1 artifact (SPEC §3). Always the first byte.
const VERSION_V0_1: u8 = 0x01;

/// Length of the inline slug, in bytes (SPEC §3, §9).
const SLUG_LEN: usize = 32;

/// Minimum artifact length: version byte + empty payload + slug (SPEC §3).
const MIN_ARTIFACT_LEN: usize = 1 + SLUG_LEN;

/// Fixed 12-byte zero ChaCha20 nonce, counter starts at 0 (SPEC §4 step 3).
const NONCE: [u8; 12] = [0u8; 12];

/// HKDF `info` string, bound exactly to this spec version (SPEC §5).
/// Exactly 20 ASCII bytes, no null terminator.
const SLUG_KEY_INFO: &[u8] = b"laquna/v0.1/slug-key";

// Compile-time guard on the spec-mandated info-string length (SPEC §5).
const _: () = assert!(SLUG_KEY_INFO.len() == 20);

/// Error returned by [`decode`] when an artifact cannot be recovered.
///
/// See SPEC §7 (Error semantics). A conforming decoder must keep these variants
/// distinct rather than collapsing them into one opaque error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Artifact is shorter than the 33-byte minimum (version byte + slug).
    InputTooShort,
    /// Version byte does not match a supported version. v0.1 supports only `0x01`.
    InvalidVersion(u8),
    /// zstd decompression of the recovered payload failed, or content-checksum
    /// verification failed. Typically a wrong seed, wrong slug, or wire corruption.
    DecompressionFailed,
}

/// Derive the 32-byte ChaCha20 slug-key via HKDF-SHA256 (SPEC §5).
///
/// `ikm = seed`, `salt = slug`, `info = SLUG_KEY_INFO`, `L = 32`. The seed carries
/// per-record entropy and the slug the batch context; IKM and salt MUST NOT be swapped.
#[cfg(feature = "alloc")]
fn derive_slug_key(seed: &[u8], slug: &[u8; SLUG_LEN]) -> [u8; 32] {
    use hkdf::Hkdf;
    use sha2::Sha256;

    let hk = Hkdf::<Sha256>::new(Some(&slug[..]), seed);
    let mut okm = [0u8; 32];
    hk.expand(SLUG_KEY_INFO, &mut okm)
        .expect("HKDF-SHA256 expand to 32 bytes is always within output bounds");
    okm
}

/// XOR the ChaCha20 keystream into `buf` in place (SPEC §4 steps 3-4 / §6 steps 5-6).
///
/// ChaCha20 (RFC 8439 IETF variant) with the 32-byte slug-key, the 12-byte zero nonce,
/// and counter 0. The operation is symmetric, so the same routine drives both encode
/// (compressed -> payload) and decode (payload -> compressed).
#[cfg(feature = "alloc")]
fn apply_keystream(slug_key: &[u8; 32], buf: &mut [u8]) {
    use chacha20::cipher::{KeyIvInit, StreamCipher};
    use chacha20::ChaCha20;

    let mut cipher = ChaCha20::new_from_slices(slug_key, &NONCE)
        .expect("32-byte key and 12-byte nonce are valid ChaCha20 inputs");
    cipher.apply_keystream(buf);
}

/// Decompress `compressed` with ruzstd, verifying the zstd content checksum if present.
///
/// ruzstd does not compare the stored and computed content checksums itself; this
/// routine performs that comparison (SPEC §6 step 7). Any parse, decompression, or
/// checksum-mismatch failure maps to [`DecodeError::DecompressionFailed`].
#[cfg(feature = "alloc")]
fn decompress_verify(compressed: &[u8]) -> Result<Vec<u8>, DecodeError> {
    use ruzstd::io::Read;
    use ruzstd::StreamingDecoder;

    let mut decoder =
        StreamingDecoder::new(compressed).map_err(|_| DecodeError::DecompressionFailed)?;

    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = decoder
            .read(&mut buf)
            .map_err(|_| DecodeError::DecompressionFailed)?;
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
    }

    // Verify the content checksum when the frame carries one (SPEC §6 step 7).
    let frame_decoder = decoder.into_frame_decoder();
    if let Some(expected) = frame_decoder.get_checksum_from_data() {
        match frame_decoder.get_calculated_checksum() {
            Some(calculated) if calculated == expected => {}
            _ => return Err(DecodeError::DecompressionFailed),
        }
    }

    Ok(out)
}

/// Encode `plaintext` into a laquna v0.1 artifact (SPEC §4).
///
/// `slug` is the 32-byte rotation-batch identifier embedded inline in the output.
/// `seed` supplies the per-record entropy for slug-key derivation and is treated as
/// opaque bytes.
///
/// The transform is deterministic: identical `(plaintext, seed, slug)` always produce
/// byte-identical output. The caller MUST ensure seeds are unique per record within a
/// slug (SPEC §8) — reuse is nonce reuse in the underlying stream cipher.
///
/// # Examples
///
/// ```
/// use laquna::{decode, encode};
///
/// let slug = [0x5a_u8; 32];
/// let seed = b"did:plc:example||3kabcd2lqr7s2y||laquna/v0.1";
///
/// let artifact = encode(b"a private post", seed, &slug);
/// assert_eq!(decode(&artifact, seed).unwrap(), b"a private post");
/// ```
///
/// # Panics
///
/// Panics if `seed` is empty. An empty seed cannot satisfy the per-record uniqueness
/// requirement (SPEC §8, §10) and is treated as a caller-precondition violation.
#[cfg(feature = "std")]
pub fn encode(plaintext: &[u8], seed: &[u8], slug: &[u8; SLUG_LEN]) -> Vec<u8> {
    assert!(
        !seed.is_empty(),
        "laquna: seed must be non-empty (SPEC §8)"
    );

    // 1. zstd compress at level 3 with the content-checksum frame flag enabled.
    let compressed = {
        use zstd::bulk::Compressor;
        use zstd::stream::raw::CParameter;

        let mut compressor = Compressor::new(3).expect("zstd compression level 3 is valid");
        compressor
            .set_parameter(CParameter::ChecksumFlag(true))
            .expect("enabling the zstd content-checksum flag is supported");
        compressor
            .compress(plaintext)
            .expect("zstd compression of an in-memory buffer does not fail")
    };

    // 2. Derive the slug-key, then 3-4. XOR the ChaCha20 keystream over the payload.
    let slug_key = derive_slug_key(seed, slug);
    let mut payload = compressed;
    apply_keystream(&slug_key, &mut payload);

    // 5. Assemble [version][payload][slug].
    let mut artifact = Vec::with_capacity(1 + payload.len() + SLUG_LEN);
    artifact.push(VERSION_V0_1);
    artifact.extend_from_slice(&payload);
    artifact.extend_from_slice(slug);
    artifact
}

/// Decode a laquna v0.1 artifact back to its plaintext (SPEC §6).
///
/// `seed` must reproduce the value used at encode time. The 32-byte slug is read from
/// the tail of `encoded`; the caller does not supply it.
///
/// # Errors
///
/// - [`DecodeError::InputTooShort`] if `encoded` is shorter than 33 bytes.
/// - [`DecodeError::InvalidVersion`] if the version byte is not `0x01`.
/// - [`DecodeError::DecompressionFailed`] if zstd decompression or content-checksum
///   verification fails — typically a wrong seed, wrong slug, or wire corruption.
///
/// # Panics
///
/// Panics if `seed` is empty, mirroring [`encode`] (SPEC §8, §10).
#[cfg(feature = "alloc")]
pub fn decode(encoded: &[u8], seed: &[u8]) -> Result<Vec<u8>, DecodeError> {
    assert!(
        !seed.is_empty(),
        "laquna: seed must be non-empty (SPEC §8)"
    );

    // 1-2. Structural pre-checks before any keystream derivation.
    if encoded.len() < MIN_ARTIFACT_LEN {
        return Err(DecodeError::InputTooShort);
    }
    if encoded[0] != VERSION_V0_1 {
        return Err(DecodeError::InvalidVersion(encoded[0]));
    }

    // 3. Split: slug is the last 32 bytes; payload is everything between.
    let split = encoded.len() - SLUG_LEN;
    let payload = &encoded[1..split];
    let slug: &[u8; SLUG_LEN] = encoded[split..]
        .try_into()
        .expect("slice is exactly SLUG_LEN bytes by construction");

    // 4-6. Derive the slug-key and XOR the keystream to recover the compressed bytes.
    let slug_key = derive_slug_key(seed, slug);
    let mut compressed = Vec::from(payload);
    apply_keystream(&slug_key, &mut compressed);

    // 7-8. Decompress with checksum verification.
    decompress_verify(&compressed)
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn info_string_is_exactly_31_ascii_bytes() {
        assert_eq!(SLUG_KEY_INFO.len(), 20);
        assert_eq!(SLUG_KEY_INFO, b"laquna/v0.1/slug-key");
        assert!(SLUG_KEY_INFO.is_ascii());
    }

    #[test]
    fn round_trip_smoke() {
        let plaintext = b"the bytes at rest should not be greppable English text";
        let seed = b"did:plc:example||rkey||spec-constant";
        let slug = [0x5a_u8; SLUG_LEN];

        let artifact = encode(plaintext, seed, &slug);
        assert_eq!(artifact[0], VERSION_V0_1);
        assert_eq!(&artifact[artifact.len() - SLUG_LEN..], &slug);

        let recovered = decode(&artifact, seed).expect("round-trip decode");
        assert_eq!(recovered, plaintext);
    }
}
