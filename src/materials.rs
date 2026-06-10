//! Keying-material derivation.
//!
//! Turns the caller's `(seed, slug)` into the opaque per-generator material that seeds
//! each of the three component streams in the mask pipeline. See `lib.rs` for how the
//! material flows into the generators.

/// Opaque keying material for the three component stream generators.
///
/// Constructed only by [`derive`]; the per-stream material is read back through the
/// crate-internal accessors.
pub struct Material;

/// Derive the keying material for all three component streams from `seed` and `slug`.
pub fn derive(_seed: &[u8], _slug: &[u8; 32]) -> Material {
    unimplemented!("materials::derive lands in the derivation arc")
}
