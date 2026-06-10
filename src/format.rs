//! Artifact byte layout.
//!
//! An artifact is `[version byte][payload][32-byte slug]`. This module owns the
//! version byte, the tail placement of the slug, and the structural pre-checks that
//! guard disassembly. It does not touch the payload contents — that is the mask
//! pipeline's job (see `lib.rs`).

use alloc::vec::Vec;

use crate::DecodeError;

/// Length of the inline slug, in bytes.
pub(crate) const SLUG_LEN: usize = 32;

/// Concatenate `[version][payload][slug]` into the finished artifact.
#[cfg(feature = "std")]
pub(crate) fn assemble(_payload: Vec<u8>, _slug: &[u8; SLUG_LEN]) -> Vec<u8> {
    unimplemented!("format::assemble lands in the pipeline arc")
}

/// Validate the structural pre-checks and split an artifact into `(payload, slug)`.
pub(crate) fn disassemble(_encoded: &[u8]) -> Result<(Vec<u8>, [u8; SLUG_LEN]), DecodeError> {
    unimplemented!("format::disassemble lands in the pipeline arc")
}
