//! Bounded, typed catalog expressions. Compilation and evaluation perform no IO.
mod compiler;
mod eval;
mod model;
mod number;
mod parser;

pub use compiler::{CheckedExpr, Plan, check, compile, plan, validate_catalog};
pub use eval::evaluate;
pub use model::*;
pub use parser::{ParsedExpr, parse};

pub const LANGUAGE: &str = "canon-expr/1";
pub const MODEL_SPEC: &str = include_str!("../../../ess/domains/expr.yaml");
pub const MAX_SOURCE: usize = 65536;
pub const MAX_TOKENS: usize = 16384;
pub const MAX_DEPTH: usize = 32;
pub const MAX_NODES: usize = 4096;
pub const MAX_ASSERTIONS: usize = 1024;
pub const MAX_REQUESTS: usize = 4096;
pub const MAX_BOUND_BYTES: usize = 1024 * 1024;

pub(crate) fn error(code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        message: message.into(),
        start: 0,
        end: 0,
        expression: None,
    }
}

pub(crate) fn digest(value: &impl serde::Serialize) -> Result<String, Diagnostic> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(value).map_err(|e| error("encoding", e.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}
