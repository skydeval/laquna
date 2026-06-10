//! Extendable stream generator (component stream B).
//!
//! A second independent contribution to the mask, decorrelated from the anchor by its
//! distinct derivation context. See `lib.rs` for how this stream is consumed.

use alloc::vec::Vec;

use crate::materials::Material;

/// Generator for the extendable stream.
pub(crate) struct ExtendableGenerator;

impl ExtendableGenerator {
    /// Build a generator seeded from the extendable material.
    pub(crate) fn new(material: &Material) -> Self {
        let _seed = material.extendable();
        unimplemented!("extendable generator lands in the stream arc")
    }

    /// Emit the next `length` bytes of the extendable stream.
    pub(crate) fn produce(&mut self, _length: usize) -> Vec<u8> {
        unimplemented!("extendable generator lands in the stream arc")
    }
}
