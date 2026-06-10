//! Decode-time failure variants and the conditions that produce them.
//!
//! Returned by [`crate::decode`]; see that function for the order in which these
//! conditions are checked.

/// Error returned by [`crate::decode`] when an artifact cannot be recovered.
///
/// A conforming decoder keeps these variants distinct rather than collapsing them
/// into a single opaque error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Input is shorter than the minimum artifact length (version byte + slug).
    InputTooShort,
    /// Leading version byte is not the one this version of the format accepts.
    InvalidVersion(u8),
    /// The recovered bytes did not validate — typically a wrong seed, a wrong
    /// slug, or corruption. Carries no detail by design.
    DecompressionFailed,
    /// The supplied seed was empty. An empty seed cannot satisfy the per-record
    /// uniqueness requirement and is rejected before any other processing.
    EmptySeed,
}
