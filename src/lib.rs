#![no_std]

//! laquna — a deterministic, reversible byte transform for
//! at-rest content opacity.
//!
//! Given a plaintext, an opaque `seed`, and a 32-byte `slug`, [`encode`] produces an
//! artifact whose payload is opaque to casual inspection; given that artifact and the
//! same seed, [`decode`] recovers the plaintext.
//!
//! **laquna is not encryption.** The decoder ships in this repository, the slug travels
//! inline in every artifact, and the seed is typically derived from public metadata.
//! laquna's value is friction against opportunistic, at-scale content extraction — not
//! confidentiality, authentication, or resistance to a motivated adversary.
//!
//! The implementation is introduced module-by-module across the implementation arcs;
//! this root is the placeholder left after the prior implementation was removed.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;
