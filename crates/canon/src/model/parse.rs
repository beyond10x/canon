//! Reading `protocol/1` source text into the model. Pure: the caller supplies the text.

use std::fmt;

use super::Protocol;

/// The source format this model reads.
pub const FORMAT: &str = "protocol/1";

/// The document is not well-formed `protocol/1` YAML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parses `protocol/1` source text.
pub fn parse(source: &str) -> Result<Protocol, ParseError> {
    serde_yaml_ng::from_str(source).map_err(|error| ParseError {
        message: super::one_line(&error.to_string()),
    })
}
