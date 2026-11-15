//! Round-trip coverage: `decode(encode(pt)) == pt` across a range of plaintext shapes.

mod common;

use common::{stress_64k, typical_post, SEED, SLUG};
use laquna::{decode, encode};

/// Encode then decode `plaintext`, asserting recovery and the structural invariants
/// of the artifact (version byte at the head, slug at the tail).
fn assert_round_trip(plaintext: &[u8]) {
    let artifact = encode(plaintext, SEED, &SLUG);

    assert_eq!(artifact[0], 0x02, "version byte must be 0x02");
    assert!(artifact.len() >= 33, "artifact must meet the 33-byte floor");
    assert_eq!(
        &artifact[artifact.len() - 32..],
        &SLUG,
        "slug must be embedded at the tail unchanged"
    );

    let recovered = decode(&artifact, SEED).expect("valid artifact decodes under its seed");
    assert_eq!(recovered, plaintext, "decode must recover the original plaintext");
}

#[test]
fn empty_plaintext() {
    assert_round_trip(b"");
}

#[test]
fn single_byte() {
    assert_round_trip(b"\x00");
    assert_round_trip(b"Z");
    assert_round_trip(b"\xff");
}

#[test]
fn short_ascii() {
    assert_round_trip(b"hello world");
}

#[test]
fn all_zero_bytes() {
    assert_round_trip(&[0u8; 512]);
}

#[test]
fn all_ff_bytes() {
    assert_round_trip(&[0xffu8; 512]);
}

#[test]
fn typical_post_sized() {
    let pt = typical_post();
    assert_eq!(pt.len(), 600);
    assert_round_trip(&pt);
}

#[test]
fn stress_64kib() {
    assert_round_trip(&stress_64k());
}

/// The plaintext contains the 32-byte slug value inline; the implementation must not
/// strip, escape, or otherwise mangle those bytes.
#[test]
fn plaintext_contains_slug_inline() {
    let mut pt = Vec::new();
    pt.extend_from_slice(b"prefix-");
    pt.extend_from_slice(&SLUG);
    pt.extend_from_slice(b"-suffix");
    assert_round_trip(&pt);
}
