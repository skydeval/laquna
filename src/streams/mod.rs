//! The three component stream generators whose outputs are folded into the mask.
//!
//! Each submodule exposes a generator with the same shape — `new(&Material)` then
//! `produce(length)` — so the consumer in `lib.rs` treats them uniformly and never sees
//! what each one does internally. The streams are labelled `a`, `b`, `c` after the
//! component-stream letters in the format; the labels name a role, not a primitive.

pub(crate) mod anchor;
pub(crate) mod extendable;
pub(crate) mod network;
