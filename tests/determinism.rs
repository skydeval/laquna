//! Determinism and input-separation properties.

mod common;

use common::{SEED, SLUG};
use laquna::encode;

const PLAINTEXT: &[u8] = b"a private post worth exactly one rotation batch";

/// Identical `(plaintext, seed, slug)` must produce byte-identical artifacts.
#[test]
fn same_inputs_are_byte_identical() {
    let a = encode(PLAINTEXT, SEED, &SLUG);
    let b = encode(PLAINTEXT, SEED, &SLUG);
    assert_eq!(a, b);
}

/// Two distinct slugs over the same plaintext + seed must differ.
#[test]
fn distinct_slugs_differ() {
    let other_slug = [0xa5u8; 32];
    let a = encode(PLAINTEXT, SEED, &SLUG);
    let b = encode(PLAINTEXT, SEED, &other_slug);
    assert_ne!(a, b);
}

/// Two distinct seeds over the same plaintext + slug must differ.
#[test]
fn distinct_seeds_differ() {
    let other_seed: &[u8] =
        b"did:plc:7iza6de2dwap2sbkpav7c6c6||3different||laquna/v0.2";
    let a = encode(PLAINTEXT, SEED, &SLUG);
    let b = encode(PLAINTEXT, other_seed, &SLUG);
    assert_ne!(a, b);
    // Same slug, so the tails match; only the payload diverges.
    assert_eq!(&a[a.len() - 32..], &b[b.len() - 32..]);
}
