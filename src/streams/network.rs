//! Network stream generator (component stream C).
//!
//! The load-bearing novel component: a parameterized state-iteration network that
//! evolves a 64-byte working state through repeated substitute-diffuse-permute passes,
//! emitting the state as stream bytes. Its parameters (per-pass substitution tables,
//! per-pass position permutations, the diffuse offset, and the initial state) are all
//! derived from the network material. See `lib.rs` for how this stream is consumed.

use alloc::vec::Vec;

use crate::materials::Material;

/// Generator for the network stream.
pub(crate) struct NetworkGenerator;

impl NetworkGenerator {
    /// Build a generator by deriving the network's parameters and initial state.
    pub(crate) fn new(_material: &Material) -> Self {
        unimplemented!("network generator lands in the stream arc")
    }

    /// Emit the next `length` bytes of the network stream.
    pub(crate) fn produce(&mut self, _length: usize) -> Vec<u8> {
        unimplemented!("network generator lands in the stream arc")
    }
}
