//! Internal testing surface — exposed only when the `testing-internals` feature is
//! enabled.
//!
//! This module exists to support reference-vector generation and cross-implementation
//! testing. It is **not part of the public API** and may change between patch releases
//! without semver implications. Production code MUST NOT enable this feature.
//!
//! Everything here takes the same `(seed, slug)` the encode/decode flow takes and
//! returns the corresponding component-stream bytes or the network's derived parameters
//! and round-by-round working state — the inputs a third-party implementer needs to
//! verify each stage of the network against the reference vectors.

use alloc::vec::Vec;

use crate::materials::derive;
use crate::streams::anchor::AnchorGenerator;
use crate::streams::extendable::ExtendableGenerator;
use crate::streams::network::{self, NetworkGenerator};

/// First `length` bytes of component stream A.
pub fn anchor_stream(seed: &[u8], slug: &[u8; 32], length: usize) -> Vec<u8> {
    AnchorGenerator::new(&derive(seed, slug)).produce(length)
}

/// First `length` bytes of component stream B.
pub fn extendable_stream(seed: &[u8], slug: &[u8; 32], length: usize) -> Vec<u8> {
    ExtendableGenerator::new(&derive(seed, slug)).produce(length)
}

/// First `length` bytes of component stream C.
pub fn network_stream(seed: &[u8], slug: &[u8; 32], length: usize) -> Vec<u8> {
    NetworkGenerator::new(&derive(seed, slug)).produce(length)
}

/// The network's eight per-pass substitution tables.
pub fn substitution_tables(seed: &[u8], slug: &[u8; 32]) -> [[u8; 256]; 8] {
    network::derive_schedule(&derive(seed, slug)).substitutions
}

/// The network's eight per-pass position permutations.
pub fn position_permutations(seed: &[u8], slug: &[u8; 32]) -> [[u8; 64]; 8] {
    network::derive_schedule(&derive(seed, slug)).permutations
}

/// The network's diffuse offset.
pub fn diffuse_offset(seed: &[u8], slug: &[u8; 32]) -> u8 {
    network::derive_schedule(&derive(seed, slug)).offset as u8
}

/// The network's initial working state (before any pass).
pub fn initial_state(seed: &[u8], slug: &[u8; 32]) -> [u8; 64] {
    network::derive_schedule(&derive(seed, slug)).initial_state
}

/// The network's working state after exactly `rounds` passes from the initial state.
/// `rounds` must not exceed the per-block pass count (8).
pub fn state_after_rounds(seed: &[u8], slug: &[u8; 32], rounds: usize) -> [u8; 64] {
    network::state_after(&derive(seed, slug), rounds)
}

/// First `length` bytes of the combined mask (the three streams folded together). The
/// fold is byte-local, so this prefix is independent of the total mask length.
pub fn combined_mask(seed: &[u8], slug: &[u8; 32], length: usize) -> Vec<u8> {
    crate::build_mask(&derive(seed, slug), length)
}
