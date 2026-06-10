//! Reference test-vector generator for laquna v0.2 (SPEC §16).
//!
//! Run with `cargo run --example gen_vectors`. It writes the machine-readable vectors
//! to `tests/vectors.json` (consumed by `tests/conformance.rs`) and prints a
//! human-readable Markdown hex listing to stdout for pasting into SPEC §16.
//!
//! IMPORTANT — what these vectors are: the Schema A artifacts are produced by *this*
//! reference implementation only. Other-language implementations need not reproduce
//! them byte-for-byte (the compression step's output is implementation-dependent; the
//! standard fixes decoding, not encoding). They are **reference-decode targets**: a
//! conforming implementation MUST decode each listed artifact under its seed to the
//! listed plaintext.
//!
//! Schema B error cases are verified deterministic at generation time: the generator
//! confirms each one actually produces the named error before emitting it.
//!
//! Schema C (component / round-by-round vectors per SPEC §16) is intentionally not
//! emitted here: it would require exposing the construction's internal state through
//! the public API, which the presentation discipline forbids. Its regression intent —
//! catching round-collapse and hardcoded-offset bugs, across two distinct-offset
//! materials — is covered by the in-source unit tests instead.

use laquna::{decode, encode, DecodeError, _internals};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Reference seed (the consuming system derives it from per-record identifiers).
const SEED: &[u8] = b"did:plc:7iza6de2dwap2sbkpav7c6c6||3laqlxv2k7s2y||laquna/v0.2";
/// A second seed, for the distinct-seed Schema A vector.
const SEED2: &[u8] = b"did:plc:7iza6de2dwap2sbkpav7c6c6||3different-rkey||laquna/v0.2";

/// Reference slug (any 32 bytes is valid).
const SLUG: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];
/// A second slug, for the distinct-slug Schema A vector.
const SLUG2: [u8; 32] = [0xa5u8; 32];

/// Candidate mismatched seeds for the Schema B wrong-seed vector; the generator picks
/// the first one whose decode deterministically fails.
const WRONG_SEED_CANDIDATES: &[&[u8]] = &[
    b"mismatched-seed-candidate-0",
    b"mismatched-seed-candidate-1",
    b"mismatched-seed-candidate-2",
    b"mismatched-seed-candidate-3",
];

fn typical_post_600() -> Vec<u8> {
    let mut s = String::new();
    while s.len() < 600 {
        s.push_str("the bytes someone can pull from a record store should not be greppable text. ");
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

fn make_a(
    name: &'static str,
    description: &str,
    plaintext: Vec<u8>,
    seed: &[u8],
    slug: [u8; 32],
) -> SchemaA {
    let artifact = encode(&plaintext, seed, &slug);
    // Determinism self-check: identical inputs must reproduce identical bytes.
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
        DecodeError::EmptySeed => json!({ "error": "EmptySeed" }),
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

fn make_b(
    name: &'static str,
    description: &str,
    artifact: Vec<u8>,
    seed: &[u8],
    expected: DecodeError,
) -> SchemaB {
    // Verify the case is deterministic before emitting it.
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

/// One Schema C vector: the network stream's internal state for a `(seed, slug)` pair.
/// These expose internal state and are not required for conformance; they let a
/// third-party implementer verify the network stage by stage.
struct SchemaC {
    name: &'static str,
    description: String,
    seed: Vec<u8>,
    slug: [u8; 32],
    offset: u8,
    stream_a: Vec<u8>,
    stream_b: Vec<u8>,
    stream_c: Vec<u8>,
    substitutions: [[u8; 256]; 8],
    permutations: [[u8; 64]; 8],
    initial_state: [u8; 64],
    state_after_1: [u8; 64],
    state_after_2: [u8; 64],
    state_after_8: [u8; 64],
    combined: Vec<u8>,
}

fn make_c(name: &'static str, description: &str, seed: &[u8], slug: [u8; 32]) -> SchemaC {
    let stream_c = _internals::network_stream(seed, &slug, 64);
    let state_after_8 = _internals::state_after_rounds(seed, &slug, 8);
    // Self-check: a full block of passes equals the first block of stream C output.
    assert_eq!(
        &state_after_8[..],
        &stream_c[..],
        "state after 8 passes must equal the first stream-C block for '{name}'"
    );
    SchemaC {
        name,
        description: description.to_string(),
        seed: seed.to_vec(),
        slug,
        offset: _internals::diffuse_offset(seed, &slug),
        stream_a: _internals::anchor_stream(seed, &slug, 64),
        stream_b: _internals::extendable_stream(seed, &slug, 64),
        stream_c,
        substitutions: _internals::substitution_tables(seed, &slug),
        permutations: _internals::position_permutations(seed, &slug),
        initial_state: _internals::initial_state(seed, &slug),
        state_after_1: _internals::state_after_rounds(seed, &slug, 1),
        state_after_2: _internals::state_after_rounds(seed, &slug, 2),
        state_after_8,
        combined: _internals::combined_mask(seed, &slug, 64),
    }
}

impl SchemaC {
    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "seed_hex": hex::encode(&self.seed),
            "slug_hex": hex::encode(self.slug),
            "diffuse_offset": self.offset,
            "stream_a_first64_hex": hex::encode(&self.stream_a),
            "stream_b_first64_hex": hex::encode(&self.stream_b),
            "stream_c_first64_hex": hex::encode(&self.stream_c),
            "substitution_tables_hex": self.substitutions.iter().map(hex::encode).collect::<Vec<_>>(),
            "position_permutations_hex": self.permutations.iter().map(hex::encode).collect::<Vec<_>>(),
            "initial_state_hex": hex::encode(self.initial_state),
            "state_after_1_round_hex": hex::encode(self.state_after_1),
            "state_after_2_rounds_hex": hex::encode(self.state_after_2),
            "state_after_8_rounds_hex": hex::encode(self.state_after_8),
            "combined_first64_hex": hex::encode(&self.combined),
        })
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
        make_a("typical_post_600", "~600-byte typical post (UTF-8).", typical_post_600(), SEED, SLUG),
        make_a("stress_64k", "64 KiB stress plaintext: byte i = i mod 251.", stress_64k(), SEED, SLUG),
        make_a("slug_inline", "Plaintext embeds the 32-byte slug inline (must not be stripped/escaped).", slug_inline_plaintext(), SEED, SLUG),
        make_a("hello_world_slug2", "Same plaintext+seed as `hello_world` but a distinct slug; artifact MUST differ.", b"hello world".to_vec(), SEED, SLUG2),
        make_a("hello_world_seed2", "Same plaintext+slug as `hello_world` but a distinct seed; artifact MUST differ.", b"hello world".to_vec(), SEED2, SLUG),
    ];

    // Cross-vector property assertions (SPEC §16 Schema A): distinctness.
    let by_name = |n: &str| schema_a.iter().find(|v| v.name == n).unwrap();
    assert_ne!(by_name("hello_world").artifact, by_name("hello_world_slug2").artifact);
    assert_ne!(by_name("hello_world").artifact, by_name("hello_world_seed2").artifact);
    // Distinct seed keeps the same tail slug; only the payload diverges.
    assert_eq!(
        &by_name("hello_world").artifact[by_name("hello_world").artifact.len() - 32..],
        &by_name("hello_world_seed2").artifact[by_name("hello_world_seed2").artifact.len() - 32..],
    );

    // ---- Schema B: decode-error vectors ----
    // A valid artifact reused by the empty-seed / seed-mismatch / corruption cases.
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

    let bad_version = |v: u8| {
        let mut a = vec![0u8; 33];
        a[0] = v;
        a
    };

    let schema_b = vec![
        make_b("too_short_0", "Zero-length artifact.", Vec::new(), SEED, DecodeError::InputTooShort),
        make_b("too_short_1", "1-byte artifact.", vec![0x02], SEED, DecodeError::InputTooShort),
        make_b("too_short_16", "16-byte artifact.", vec![0u8; 16], SEED, DecodeError::InputTooShort),
        make_b("too_short_32", "32-byte artifact (one below the floor).", vec![0u8; 32], SEED, DecodeError::InputTooShort),
        make_b("bad_version_00", "33-byte artifact with version byte 0x00.", bad_version(0x00), SEED, DecodeError::InvalidVersion(0x00)),
        make_b("bad_version_01", "33-byte artifact with version byte 0x01 (the prior format version).", bad_version(0x01), SEED, DecodeError::InvalidVersion(0x01)),
        make_b("bad_version_03", "33-byte artifact with version byte 0x03.", bad_version(0x03), SEED, DecodeError::InvalidVersion(0x03)),
        make_b("bad_version_ff", "33-byte artifact with version byte 0xFF.", bad_version(0xff), SEED, DecodeError::InvalidVersion(0xff)),
        make_b("empty_seed", "Valid artifact decoded under an empty seed.", valid.clone(), b"", DecodeError::EmptySeed),
        make_b("wrong_seed", "Valid artifact decoded under a deliberately mismatched seed.", valid.clone(), wrong_seed, DecodeError::DecompressionFailed),
        make_b("corrupted_slug", "Valid artifact with the tail slug corrupted.", corrupted_slug, SEED, DecodeError::DecompressionFailed),
        make_b("bitflip_payload", "Valid artifact with a single payload bit flipped.", bitflip, SEED, DecodeError::DecompressionFailed),
    ];

    // ---- Schema C: component / round-by-round vectors (debugging aids) ----
    // Two pairs chosen so the diffuse offsets differ, which catches a hardcoded offset.
    let schema_c = vec![
        make_c("pair1", "First (seed, slug) pair; diffuse offset 59.", SEED, SLUG),
        make_c("pair2", "Second (seed, slug) pair; distinct diffuse offset 55.", SEED, SLUG2),
    ];
    assert_ne!(
        schema_c[0].offset, schema_c[1].offset,
        "the two Schema C pairs must have distinct diffuse offsets"
    );

    // ---- Write tests/vectors.json ----
    let root = json!({
        "spec": "laquna",
        "spec_version": "0.2.1",
        "note": "Schema A artifacts are from the reference implementation only. Other implementations need not produce byte-identical encode output; they MUST decode each artifact under its seed to the listed plaintext. These are reference-decode targets, not cross-encode targets. Schema B error cases are verified deterministic at generation time. Schema C exposes the network stream's internal state for two distinct-offset (seed, slug) pairs; it is a debugging aid, not required for conformance.",
        "schema_a": Value::Array(schema_a.iter().map(SchemaA::to_json).collect()),
        "schema_b": Value::Array(schema_b.iter().map(SchemaB::to_json).collect()),
        "schema_c": Value::Array(schema_c.iter().map(SchemaC::to_json).collect()),
    });
    let serialized = serde_json::to_string_pretty(&root).expect("serialize vectors");
    std::fs::write("tests/vectors.json", format!("{serialized}\n")).expect("write tests/vectors.json");
    eprintln!("wrote tests/vectors.json ({} bytes)", serialized.len() + 1);

    // ---- Emit the human-readable Markdown listing for SPEC §16 ----
    print_markdown(&schema_a, &schema_b, &schema_c, wrong_seed);
}

/// Threshold above which a vector's hex is summarized (by length + SHA-256) rather than
/// inlined in the SPEC. Keeps the 64 KiB stress vector from bloating the doc; full bytes
/// always live in tests/vectors.json.
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

fn print_markdown(
    schema_a: &[SchemaA],
    schema_b: &[SchemaB],
    schema_c: &[SchemaC],
    wrong_seed: &[u8],
) {
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
            DecodeError::EmptySeed => "EmptySeed".to_string(),
        };
        println!("- **{}** — {}", v.name, v.description);
        println!("  - artifact: {}", hex_field(&v.artifact));
        println!("  - seed: `{}`", hex::encode(&v.seed));
        println!("  - expected: `{expected}`");
    }
    println!();
    println!("**Schema C — component / round-by-round vectors (implementer debugging aids):**");
    println!();
    println!("Not required for conformance. Two `(seed, slug)` pairs with deliberately distinct");
    println!("diffuse offsets; an implementation that hardcoded one offset would match at most one.");
    for v in schema_c {
        println!();
        println!("- **{}** — {}", v.name, v.description);
        println!("  - seed: `{}`", hex::encode(&v.seed));
        println!("  - slug: `{}`", hex::encode(v.slug));
        println!("  - diffuse offset: `{}`", v.offset);
        println!("  - stream A (first 64): `{}`", hex::encode(&v.stream_a));
        println!("  - stream B (first 64): `{}`", hex::encode(&v.stream_b));
        println!("  - stream C (first 64): `{}`", hex::encode(&v.stream_c));
        for (i, s) in v.substitutions.iter().enumerate() {
            println!("  - S_{} : `{}`", i + 1, hex::encode(s));
        }
        for (i, p) in v.permutations.iter().enumerate() {
            println!("  - P_{} : `{}`", i + 1, hex::encode(p));
        }
        println!("  - W after 0 rounds: `{}`", hex::encode(v.initial_state));
        println!("  - W after 1 round: `{}`", hex::encode(v.state_after_1));
        println!("  - W after 2 rounds: `{}`", hex::encode(v.state_after_2));
        println!("  - W after 8 rounds: `{}`", hex::encode(v.state_after_8));
        println!("  - combined keystream (first 64): `{}`", hex::encode(&v.combined));
    }
    println!();
    println!("<!-- END GENERATED VECTORS -->");
}
