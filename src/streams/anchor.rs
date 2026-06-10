//! Anchor stream generator (component stream A).
//!
//! Provides the bulk of statistical uniformity in the combined mask. See `lib.rs` for
//! how this stream is consumed alongside the other two.

use alloc::vec::Vec;

use crate::materials::Material;

/// Generator for the anchor stream.
pub(crate) struct AnchorGenerator;

impl AnchorGenerator {
    /// Build a generator seeded from the anchor material.
    pub(crate) fn new(material: &Material) -> Self {
        let _seed = material.anchor();
        unimplemented!("anchor generator lands in the stream arc")
    }

    /// Emit the next `length` bytes of the anchor stream.
    pub(crate) fn produce(&mut self, _length: usize) -> Vec<u8> {
        unimplemented!("anchor generator lands in the stream arc")
    }
}
