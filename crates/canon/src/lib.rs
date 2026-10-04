#![forbid(unsafe_code)]

//! Canon: a formal language and deterministic calculus for evidence-governed protocols.
//!
//! The library is IO-free. Callers read documents and pass their contents in; nothing here reads a
//! clock, the environment, the network or the filesystem.

pub mod conform;
pub mod diff;
pub mod eval;
pub mod explain;
pub mod ir;
pub mod model;
pub mod validate;
