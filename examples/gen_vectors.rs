//! Reference test-vector generator for laquna v0.1 (SPEC §13).
//!
//! Run with `cargo run --example gen_vectors`. It writes the machine-readable
//! vectors to `tests/vectors.json` (consumed by `tests/conformance.rs`) and prints
//! a human-readable Markdown hex listing to stdout for pasting into SPEC §13.
//!
//! IMPORTANT — what these vectors are: the Schema A artifacts are produced by *this*
//! reference implementation only. Other-language implementations need not reproduce
//! them byte-for-byte (zstd encode output is implementation- and version-dependent;
//! RFC 8478 standardizes decoding, not encoding). They are **reference-decode
//! targets**, not cross-encode targets: a conforming implementation MUST decode each
//! listed artifact under its seed to the listed plaintext.
//!
//! Schema B error cases (mismatched seed, corrupted slug, bit-flipped payload) are
//! verified deterministic at generation time: the generator confirms each one
//! actually produces the named error before emitting it.

use laquna::{decode, encode, DecodeError};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Reference seed (kryphocron derives `did || rkey || spec-constant`).
const SEED: &[u8] = b"did:plc:7iza6de2dwap2sbkpav7c6c6||3laqlxv2k7s2y||laquna/v0.1";
/// A second seed, for the distinct-seed Schema A vector.
const SEED2: &[u8] = b"did:plc:7iza6de2dwap2sbkpav7c6c6||3different-rkey||laquna/v0.1";

/// Reference slug (any 32 bytes is valid; SPEC §9).
const SLUG: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];
/// A second slug, for the distinct-slug Schema A vector.
const SLUG2: [u8; 32] = [0xa5u8; 32];

/// Candidate mismatched seeds for the Schema B wrong-seed vector; the generator picks
/// the first one whose decode deterministically fails (SPEC §13 requirement).
const WRONG_SEED_CANDIDATES: &[&[u8]] = &[
    b"mismatched-seed-candidate-0",
    b"mismatched-seed-candidate-1",
    b"mismatched-seed-candidate-2",
    b"mismatched-seed-candidate-3",
];

fn typical_post_600() -> Vec<u8> {
    let mut s = String::new();
    while s.len() < 600 {
        s.push_str("the bytes someone can pull from a PDS should not be greppable English text. ");
    }
    s.truncate(600);
    s.into_bytes()
}

fn stress_64k() -> Vec<u8> {
    (0..65_536usize).map(|i| (i % 251) as u8).collect()
}

fn slug_inline_plaintext() -> Vec<u8> {
    let mut pt = Vec::new();
    pt.extend_from_slice(b"prefix-");
    pt.extend_from_slice(&SLUG);
    pt.extend_from_slice(b"-suffix");
    pt
}

/// One Schema A vector: an encode input tuple plus the reference artifact.
struct SchemaA {
    name: &'static str,
    description: String,
    plaintext: Vec<u8>,
    seed: Vec<u8>,
    slug: [u8; 32],
    artifact: Vec<u8>,
}

fn make_a(name: &'static str, description: &str, plaintext: Vec<u8>, seed: &[u8], slug: [u8; 32]) -> SchemaA {
    let artifact = encode(&plaintext, seed, &slug);
    // Determinism self-check: identical inputs must reproduce identical bytes (SPEC §4).
    let again = encode(&plaintext, seed, &slug);
    assert_eq!(artifact, again, "encode is not deterministic for vector '{name}'");
    // Sanity: the reference artifact must decode back under its own seed.
    let recovered = decode(&artifact, seed).expect("reference artifact must round-trip");
    assert_eq!(recovered, plaintext, "round-trip mismatch for vector '{name}'");
    SchemaA {
        name,
        description: description.to_string(),
        plaintext,
        seed: seed.to_vec(),
        slug,
        artifact,
    }
}

impl SchemaA {
    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "plaintext_hex": hex::encode(&self.plaintext),
            "seed_hex": hex::encode(&self.seed),
            "slug_hex": hex::encode(self.slug),
            "artifact_hex": hex::encode(&self.artifact),
        })
    }
}

/// One Schema B vector: a decode input plus the expected error.
struct SchemaB {
    name: &'static str,
    description: String,
    artifact: Vec<u8>,
    seed: Vec<u8>,
    expected: DecodeError,
}

fn expected_to_json(e: &DecodeError) -> Value {
    match e {
        DecodeError::InputTooShort => json!({ "error": "InputTooShort" }),
        DecodeError::InvalidVersion(v) => json!({ "error": "InvalidVersion", "version": v }),
        DecodeError::DecompressionFailed => json!({ "error": "DecompressionFailed" }),
    }
}

impl SchemaB {
    fn to_json(&self) -> Value {
        let mut obj = json!({
            "name": self.name,
            "description": self.description,
            "artifact_hex": hex::encode(&self.artifact),
            "seed_hex": hex::encode(&self.seed),
        });
        obj.as_object_mut()
            .unwrap()
            .insert("expected".to_string(), expected_to_json(&self.expected));
        obj
    }
}

fn make_b(name: &'static str, description: &str, artifact: Vec<u8>, seed: &[u8], expected: DecodeError) -> SchemaB {
    // Verify the case is deterministic before emitting it (SPEC §13).
    let got = decode(&artifact, seed);
    assert_eq!(
        got,
        Err(expected.clone()),
        "Schema B vector '{name}' did not produce the expected error"
    );
    SchemaB {
        name,
        description: description.to_string(),
        artifact,
        seed: seed.to_vec(),
        expected,
    }
}

fn main() {
    // ---- Schema A: encode / round-trip vectors ----
    let schema_a = vec![
        make_a("empty", "Empty plaintext (0 bytes).", Vec::new(), SEED, SLUG),
        make_a("single_byte", "Single-byte plaintext (0x2a).", vec![0x2a], SEED, SLUG),
        make_a("hello_world", "Short ASCII plaintext \"hello world\".", b"hello world".to_vec(), SEED, SLUG),
        make_a("all_zero_64", "64 bytes, all 0x00.", vec![0x00u8; 64], SEED, SLUG),
        make_a("all_ff_64", "64 bytes, all 0xFF.", vec![0xffu8; 64], SEED, SLUG),
        make_a("typical_post_600", "~600-byte typical Bluesky post (UTF-8).", typical_post_600(), SEED, SLUG),
        make_a("stress_64k", "64 KiB stress plaintext: byte i = i mod 251.", stress_64k(), SEED, SLUG),
        make_a("slug_inline", "Plaintext embeds the 32-byte slug inline (must not be stripped/escaped).", slug_inline_plaintext(), SEED, SLUG),
        make_a("hello_world_slug2", "Same plaintext+seed as `hello_world` but a distinct slug; artifact MUST differ.", b"hello world".to_vec(), SEED, SLUG2),
        make_a("hello_world_seed2", "Same plaintext+slug as `hello_world` but a distinct seed; artifact MUST differ.", b"hello world".to_vec(), SEED2, SLUG),
    ];

    // Cross-vector property assertions (SPEC §13 Schema A): distinctness.
    let by_name = |n: &str| schema_a.iter().find(|v| v.name == n).unwrap();
    assert_ne!(by_name("hello_world").artifact, by_name("hello_world_slug2").artifact);
    assert_ne!(by_name("hello_world").artifact, by_name("hello_world_seed2").artifact);
    // Distinct seed keeps the same tail slug; only the payload diverges.
    assert_eq!(
        &by_name("hello_world").artifact[by_name("hello_world").artifact.len() - 32..],
        &by_name("hello_world_seed2").artifact[by_name("hello_world_seed2").artifact.len() - 32..],
    );

    // ---- Schema B: decode-error vectors ----
    // A valid artifact reused by the seed-mismatch / corruption cases.
    let b_plaintext: &[u8] = b"some private post content of a reasonable, compressible length";
    let valid = encode(b_plaintext, SEED, &SLUG);

    // Mismatched seed: pick the first candidate that deterministically fails.
    let wrong_seed = WRONG_SEED_CANDIDATES
        .iter()
        .copied()
        .find(|s| decode(&valid, s) == Err(DecodeError::DecompressionFailed))
        .expect("at least one candidate seed must deterministically fail decode");

    // Corrupted slug: flip the final byte.
    let mut corrupted_slug = valid.clone();
    let last = corrupted_slug.len() - 1;
    corrupted_slug[last] ^= 0xff;

    // Single-bit-flip payload: flip one bit mid-payload (region [1 .. len-32]).
    let mut bitflip = valid.clone();
    let mid = 1 + (bitflip.len() - 33) / 2;
    bitflip[mid] ^= 0x01;

    let mut bad_version_00 = vec![0u8; 33];
    bad_version_00[0] = 0x00;
    let mut bad_version_02 = vec![0u8; 33];
    bad_version_02[0] = 0x02;
    let mut bad_version_ff = vec![0u8; 33];
    bad_version_ff[0] = 0xff;

    let schema_b = vec![
        make_b("too_short_0", "Zero-length artifact.", Vec::new(), SEED, DecodeError::InputTooShort),
        make_b("too_short_1", "1-byte artifact.", vec![0x01], SEED, DecodeError::InputTooShort),
        make_b("too_short_16", "16-byte artifact.", vec![0u8; 16], SEED, DecodeError::InputTooShort),
        make_b("too_short_32", "32-byte artifact (one below the floor).", vec![0u8; 32], SEED, DecodeError::InputTooShort),
        make_b("bad_version_00", "33-byte artifact with version byte 0x00.", bad_version_00, SEED, DecodeError::InvalidVersion(0x00)),
        make_b("bad_version_02", "33-byte artifact with version byte 0x02.", bad_version_02, SEED, DecodeError::InvalidVersion(0x02)),
        make_b("bad_version_ff", "33-byte artifact with version byte 0xFF.", bad_version_ff, SEED, DecodeError::InvalidVersion(0xff)),
        make_b("wrong_seed", "Valid artifact decoded under a deliberately mismatched seed.", valid.clone(), wrong_seed, DecodeError::DecompressionFailed),
        make_b("corrupted_slug", "Valid artifact with the tail slug corrupted.", corrupted_slug, SEED, DecodeError::DecompressionFailed),
        make_b("bitflip_payload", "Valid artifact with a single payload bit flipped.", bitflip, SEED, DecodeError::DecompressionFailed),
    ];

    // ---- Write tests/vectors.json ----
    let root = json!({
        "spec": "laquna",
        "spec_version": "0.1",
        "note": "Schema A artifacts are from the reference implementation only. Other implementations need not produce byte-identical encode output; they MUST decode each artifact under its seed to the listed plaintext. These are reference-decode targets, not cross-encode targets. Schema B error cases are verified deterministic at generation time.",
        "schema_a": Value::Array(schema_a.iter().map(SchemaA::to_json).collect()),
        "schema_b": Value::Array(schema_b.iter().map(SchemaB::to_json).collect()),
    });
    let serialized = serde_json::to_string_pretty(&root).expect("serialize vectors");
    std::fs::write("tests/vectors.json", format!("{serialized}\n")).expect("write tests/vectors.json");
    eprintln!("wrote tests/vectors.json ({} bytes)", serialized.len() + 1);

    // ---- Emit the human-readable Markdown listing for SPEC §13 ----
    print_markdown(&schema_a, &schema_b, wrong_seed);
}

/// Threshold above which a vector's hex is summarized (by length + SHA-256) rather
/// than inlined in the SPEC. Keeps the 64 KiB stress vector from bloating the doc;
/// full bytes always live in tests/vectors.json.
const INLINE_HEX_LIMIT: usize = 2048;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn hex_field(bytes: &[u8]) -> String {
    if bytes.len() <= INLINE_HEX_LIMIT {
        format!("`{}`", hex::encode(bytes))
    } else {
        format!(
            "{} bytes — full hex in `tests/vectors.json`; SHA-256 `{}`",
            bytes.len(),
            sha256_hex(bytes)
        )
    }
}

fn print_markdown(schema_a: &[SchemaA], schema_b: &[SchemaB], wrong_seed: &[u8]) {
    println!("<!-- BEGIN GENERATED VECTORS (examples/gen_vectors.rs) -->");
    println!();
    println!("The byte-exact, machine-readable vectors live in `tests/vectors.json`; the");
    println!("conformance suite (`tests/conformance.rs`) is driven from that file. The listings");
    println!("below are the human-readable form. Schema A artifacts are **reference-decode");
    println!("targets** — produced by the reference implementation only; other implementations");
    println!("need not reproduce the bytes but MUST decode each to the listed plaintext.");
    println!();
    println!("**Schema A — encode / round-trip vectors:**");
    println!();
    for v in schema_a {
        println!("- **{}** — {}", v.name, v.description);
        println!("  - plaintext: {}", hex_field(&v.plaintext));
        println!("  - seed: `{}`", hex::encode(&v.seed));
        println!("  - slug: `{}`", hex::encode(v.slug));
        println!("  - artifact: {}", hex_field(&v.artifact));
    }
    println!();
    println!("**Schema B — decode-error vectors:**");
    println!();
    println!(
        "(The `wrong_seed` vector decodes the `corrupted_slug`/`bitflip_payload` base artifact \
         under mismatched seed `{}`.)",
        hex::encode(wrong_seed)
    );
    println!();
    for v in schema_b {
        let expected = match &v.expected {
            DecodeError::InputTooShort => "InputTooShort".to_string(),
            DecodeError::InvalidVersion(n) => format!("InvalidVersion({n})"),
            DecodeError::DecompressionFailed => "DecompressionFailed".to_string(),
        };
        println!("- **{}** — {}", v.name, v.description);
        println!("  - artifact: {}", hex_field(&v.artifact));
        println!("  - seed: `{}`", hex::encode(&v.seed));
        println!("  - expected: `{expected}`");
    }
    println!();
    println!("<!-- END GENERATED VECTORS -->");
}
