//! Mask assembly.
//!
//! Folds the three component streams into the single mask that is XOR'd against the
//! compressed payload, and provides the byte-local XOR that applies a mask to a buffer
//! in either direction. Output position `i` depends only on the three inputs at `i`.

use alloc::vec::Vec;

/// Fold the three equal-length component streams into the final mask.
pub(crate) fn mix(_stream_a: &[u8], _stream_b: &[u8], _stream_c: &[u8]) -> Vec<u8> {
    unimplemented!("combine::mix lands in the pipeline arc")
}

/// XOR two equal-length buffers into a fresh buffer. Self-inverse, so the same routine
/// serves both the encode and decode directions.
pub(crate) fn xor_buffers(_lhs: &[u8], _rhs: &[u8]) -> Vec<u8> {
    unimplemented!("combine::xor_buffers lands in the pipeline arc")
}
