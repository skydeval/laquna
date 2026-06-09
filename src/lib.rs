#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Error returned by [`decode`] when an artifact cannot be recovered.
///
/// See SPEC §7 (Error semantics). A conforming decoder must keep these
/// variants distinct rather than collapsing them into one opaque error.
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

/// Encode `plaintext` into a laquna v0.1 artifact. See SPEC §4.
#[cfg(feature = "std")]
pub fn encode(plaintext: &[u8], seed: &[u8], slug: &[u8; 32]) -> Vec<u8> {
    let _ = (plaintext, seed, slug);
    todo!("implemented in Arc 2")
}

/// Decode a laquna v0.1 artifact back to its plaintext. See SPEC §6.
#[cfg(feature = "alloc")]
pub fn decode(encoded: &[u8], seed: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let _ = (encoded, seed);
    todo!("implemented in Arc 2")
}
