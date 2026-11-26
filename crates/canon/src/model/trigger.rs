//! The `canon-trigger/1` document `canon trigger validate` reads: one trigger rule
//! (story:trigger-rule-document, `docs/design/trigger-rules.md` § 3).
//!
//! ```yaml
//! format: canon-trigger/1
//! id: stale-invoice-reminder
//! version: 3
//! owner: billing-operations
//! subject:
//!   kind: invoice
//!   state: {field: status, is: overdue}
//!   fields:
//!     id: {type: string}
//!     status: {type: string}
//!     amount: {type: decimal}
//!     segment: {type: string}
//!     note: {type: string, optional: true}
//! where: '$row.amount > 100 and not contains(["test", "internal"], $row.segment)'
//! start:
//!   protocol: send-reminder
//!   input:
//!     invoice: {field: id}
//!     channel: {const: email}
//! mode: auto
//! access: write
//! refire: once_per_state_entry
//! cap: 50
//! ```
//!
//! What the rule's fields must hold beyond their shape, and how `where` is compiled, is
//! `crate::trigger`'s; this file is the document's shape, and the shape of the
//! `canon-trigger-protocols/1` document `canon trigger protocols` prints
//! (story:protocol-access-class).

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use super::ids::{InputId, InputValue, ProtocolId};
use super::{AccessClass, Declarations};

/// The trigger rule format `canon trigger validate` reads.
pub const TRIGGER_FORMAT: &str = "canon-trigger/1";

/// The format `canon trigger protocols` prints (story:protocol-access-class).
pub const TRIGGER_PROTOCOLS_FORMAT: &str = "canon-trigger-protocols/1";

identifier!(
    /// Identifies a trigger rule; with its version, the rule key `<id>@<version>`.
    TriggerRuleId
);
identifier!(
    /// The kind name of the rows a trigger rule watches, as the snapshot names it.
    TriggerKind
);
identifier!(
    /// The name of a row field a trigger rule declares and reads.
    TriggerFieldName
);

/// A `canon-trigger/1` document as written: one rule.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerRule {
    /// The document format; only [`TRIGGER_FORMAT`] is accepted.
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// The rule's id, an identifier.
    #[serde(deserialize_with = "super::present::required")]
    pub id: TriggerRuleId,
    /// The rule's version, a positive integer; a changed rule is a new version.
    #[serde(deserialize_with = "super::present::required")]
    pub version: u64,
    /// Who answers for the rule, not empty. Canon does not resolve it.
    #[serde(deserialize_with = "super::present::required")]
    pub owner: String,
    /// The rows the rule watches.
    #[serde(deserialize_with = "super::present::required")]
    pub subject: TriggerSubject,
    /// One `canon-expr/1` expression of type Bool over `$row`, the record of the declared fields.
    /// Absent, every row in the state matches.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub r#where: Option<String>,
    /// The protocol a match starts, and its case inputs.
    #[serde(deserialize_with = "super::present::required")]
    pub start: TriggerStart,
    /// Whether a match is offered or would start the protocol.
    #[serde(deserialize_with = "super::present::required")]
    pub mode: TriggerMode,
    /// The access class the rule expects of the protocol it starts.
    #[serde(deserialize_with = "super::present::required")]
    pub access: TriggerAccess,
    /// Whether a row fires again when it re-enters its state.
    #[serde(deserialize_with = "super::present::required")]
    pub refire: TriggerRefire,
    /// Starts per rule per UTC day, a positive integer.
    #[serde(deserialize_with = "super::present::required")]
    pub cap: u64,
}

impl TriggerRule {
    /// The rule key, `<id>@<version>`.
    pub fn key(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
}

/// The rows a rule watches: their kind, the row fields the rule reads, and the state a row must be
/// in.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerSubject {
    /// The kind of the rows, as the snapshot names it.
    #[serde(deserialize_with = "super::present::required")]
    pub kind: TriggerKind,
    /// The row fields the rule reads, keyed by field name; `$row` is a record of exactly these.
    #[serde(deserialize_with = "super::present::required")]
    pub fields: Declarations<TriggerFieldName, TriggerField>,
    /// The state a row must be in.
    #[serde(deserialize_with = "super::present::required")]
    pub state: TriggerState,
}

/// The state a row must be in: its field `field`, a declared required `string` field, equals
/// `is`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerState {
    /// The declared field that holds the row's state.
    #[serde(deserialize_with = "super::present::required")]
    pub field: TriggerFieldName,
    /// The value the field holds when the row is in the state.
    #[serde(deserialize_with = "super::present::required")]
    pub is: String,
}

/// One declared row field.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerField {
    /// The field's type.
    #[serde(deserialize_with = "super::present::required")]
    pub r#type: TriggerFieldType,
    /// Whether a row may lack the field; `false` when left out.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub optional: bool,
}

/// The type of a declared row field, and so of its entry in `$row`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerFieldType {
    /// `canon-expr/1` `String`.
    String,
    /// `canon-expr/1` `Integer`.
    Integer,
    /// `canon-expr/1` `Decimal`.
    Decimal,
    /// `canon-expr/1` `Bool`.
    Bool,
}

impl TriggerFieldType {
    /// The value a document writes for this type.
    pub fn as_str(self) -> &'static str {
        match self {
            TriggerFieldType::String => "string",
            TriggerFieldType::Integer => "integer",
            TriggerFieldType::Decimal => "decimal",
            TriggerFieldType::Bool => "bool",
        }
    }
}

/// The protocol a match starts, and the case inputs it fills.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerStart {
    /// The protocol's id.
    #[serde(deserialize_with = "super::present::required")]
    pub protocol: ProtocolId,
    /// The case inputs, keyed by case input id; absent, none.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub input: Declarations<InputId, TriggerInput>,
}

/// The value of one case input. Written as a map with exactly one of the keys `field` or `const`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "WrittenInput")]
pub enum TriggerInput {
    /// The value of a declared row field.
    Field(TriggerFieldName),
    /// A constant string.
    Const(String),
}

/// Whether a match is offered for approval or would start the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerMode {
    /// A match is offered for approval.
    Offer,
    /// A match would start the protocol.
    Auto,
}

/// The access class a rule expects of the protocol it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerAccess {
    /// Every action of the protocol is read-only.
    ReadOnly,
    /// Some action of the protocol writes.
    Write,
}

/// Whether a row fires again after it leaves its state and re-enters it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerRefire {
    /// Once each time the row enters the state.
    OncePerStateEntry,
    /// Once for the row, ever.
    OncePerSubject,
}

/// The `canon-trigger-protocols/1` document `canon trigger protocols` prints
/// (story:protocol-access-class): what a trigger rule is checked against, for each protocol of a
/// set, keyed by protocol id. Canon prints it from compiled protocols (`crate::trigger::protocols`);
/// an evaluator may build it otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerProtocols {
    /// The document format, [`TRIGGER_PROTOCOLS_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// One entry per protocol, keyed by protocol id.
    #[serde(deserialize_with = "super::present::required")]
    pub protocols: Declarations<ProtocolId, TriggerProtocol>,
}

/// One protocol as a trigger rule sees it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerProtocol {
    /// `read_only` when every action of the protocol is, else `write`.
    #[serde(deserialize_with = "super::present::required")]
    pub class: AccessClass,
    /// Whether some action of the protocol requires a capability.
    #[serde(deserialize_with = "super::present::required")]
    pub guarded: bool,
    /// The case inputs the protocol declares, keyed by input id, each with its values, sorted,
    /// each once.
    #[serde(deserialize_with = "super::present::required")]
    pub inputs: Declarations<InputId, Vec<InputValue>>,
}

/// An input as written: each form a key, exactly one of them present.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenInput {
    #[serde(default, deserialize_with = "super::present::form")]
    field: Option<TriggerFieldName>,
    #[serde(default, deserialize_with = "super::present::form")]
    r#const: Option<String>,
}

impl TryFrom<WrittenInput> for TriggerInput {
    type Error = String;

    fn try_from(written: WrittenInput) -> Result<Self, Self::Error> {
        match (written.field, written.r#const) {
            (Some(field), None) => Ok(TriggerInput::Field(field)),
            (None, Some(value)) => Ok(TriggerInput::Const(value)),
            _ => Err("an input has exactly one of `field` or `const`".to_owned()),
        }
    }
}
