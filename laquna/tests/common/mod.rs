//! Shared fixtures for the integration tests.
//!
//! Each test binary includes this module via `mod common;`, so not every constant
//! or helper is referenced from every file — hence the crate-level `dead_code` allow.
#![allow(dead_code)]

/// A representative per-record seed (the consuming system derives it from per-record
/// identifiers plus a format-version label).
pub const SEED: &[u8] = b"did:plc:7iza6de2dwap2sbkpav7c6c6||3laqlxv2k7s2y||laquna/v0.2";

/// A fixed 32-byte slug. Any 32-byte value is a valid slug.
pub const SLUG: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

/// Build a ~600-byte plaintext, the rough size of a typical post in UTF-8.
pub fn typical_post() -> Vec<u8> {
    let mut s = String::new();
    while s.len() < 600 {
        s.push_str("the bytes someone can pull from a record store should not be greppable text. ");
    }
    s.truncate(600);
    s.into_bytes()
}

/// Build a 64 KiB plaintext with varied (poorly-compressible) bytes for a stress test.
pub fn stress_64k() -> Vec<u8> {
    (0..65_536usize).map(|i| (i % 251) as u8).collect()
}
