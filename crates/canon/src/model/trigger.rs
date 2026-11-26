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
//! (story:protocol-access-class), and of the documents `canon trigger evaluate` reads and prints
//! (story:trigger-evaluate): the `canon-trigger-set/1` rule set, the `canon-trigger-snapshot/1`
//! snapshot, the `canon-trigger-history/1` history and the `canon-trigger-plan/1` plan.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use super::ids::{InputId, InputValue, Instant, ProtocolId};
use super::{AccessClass, Declarations, Json};

/// The trigger rule format `canon trigger validate` reads.
pub const TRIGGER_FORMAT: &str = "canon-trigger/1";

/// The format `canon trigger protocols` prints (story:protocol-access-class).
pub const TRIGGER_PROTOCOLS_FORMAT: &str = "canon-trigger-protocols/1";

/// The rule set format `canon trigger evaluate --rules` reads (story:trigger-evaluate).
pub const TRIGGER_SET_FORMAT: &str = "canon-trigger-set/1";

/// The snapshot format `canon trigger evaluate --snapshot` reads (story:trigger-evaluate).
pub const TRIGGER_SNAPSHOT_FORMAT: &str = "canon-trigger-snapshot/1";

/// The history format `canon trigger evaluate --history` reads (story:trigger-evaluate).
pub const TRIGGER_HISTORY_FORMAT: &str = "canon-trigger-history/1";

/// The plan format `canon trigger evaluate` prints (story:trigger-evaluate).
pub const TRIGGER_PLAN_FORMAT: &str = "canon-trigger-plan/1";

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
identifier!(
    /// The id of one row of a kind in a snapshot, unique within its kind.
    TriggerRowId
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

/// The `canon-trigger-set/1` document `canon trigger evaluate --rules` reads
/// (story:trigger-evaluate): each rule file with its lifecycle state, which is deployment state and
/// not part of the rule's document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerSet {
    /// The document format, [`TRIGGER_SET_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// The rules, in any order.
    #[serde(deserialize_with = "super::present::required")]
    pub rules: Vec<TriggerSetEntry>,
}

/// One rule of a set: its `canon-trigger/1` file, relative to the set file's directory or
/// absolute, and its state.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerSetEntry {
    /// The rule's `canon-trigger/1` file: relative to the set file's directory, or absolute.
    #[serde(deserialize_with = "super::present::required")]
    pub path: String,
    /// The rule's state: `draft`, `active` or `disabled`; the plan carries it per entry.
    #[serde(deserialize_with = "super::present::required")]
    pub state: TriggerRuleState,
}

/// A rule's lifecycle state. Every state is evaluated by the same steps; each plan entry carries
/// its rule's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerRuleState {
    /// Under test: simulated against real state, never started.
    Draft,
    /// Live.
    Active,
    /// Switched off.
    Disabled,
}

impl TriggerRuleState {
    /// The value a document writes for this state.
    pub fn as_str(self) -> &'static str {
        match self {
            TriggerRuleState::Draft => "draft",
            TriggerRuleState::Active => "active",
            TriggerRuleState::Disabled => "disabled",
        }
    }
}

/// The `canon-trigger-snapshot/1` document `canon trigger evaluate --snapshot` reads: the rows of
/// each kind.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerSnapshot {
    /// The document format, [`TRIGGER_SNAPSHOT_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// The rows of each kind, keyed by kind.
    #[serde(deserialize_with = "super::present::required")]
    pub kinds: Declarations<TriggerKind, Vec<TriggerRow>>,
}

/// One row of a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerRow {
    /// The row's id, unique within its kind.
    #[serde(deserialize_with = "super::present::required")]
    pub id: TriggerRowId,
    /// The instant the row entered its current state, when the source knows it.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub entered_at: Option<Instant>,
    /// The row's field values, keyed by field name: a string, a number, `true` or `false`, or
    /// null.
    #[serde(deserialize_with = "row_fields")]
    pub fields: Declarations<TriggerFieldName, Json>,
}

/// Reads a row's `fields`: a mapping from field name to a JSON scalar, a null value kept as null
/// (an optional field may be written so), a non-finite number or a tagged value refused.
fn row_fields<'de, D>(deserializer: D) -> Result<Declarations<TriggerFieldName, Json>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let yaml: serde_yaml_ng::Value = super::present::required(deserializer)?;
    let serde_yaml_ng::Value::Mapping(mapping) = yaml else {
        return Err(D::Error::custom(
            "the fields are a mapping from field name to value",
        ));
    };
    let mut entries = Vec::with_capacity(mapping.len());
    for (key, value) in mapping {
        let serde_yaml_ng::Value::String(key) = key else {
            return Err(D::Error::custom("a field name is a string"));
        };
        let value = super::present::json(value).map_err(D::Error::custom)?;
        entries.push((TriggerFieldName::new(key), value));
    }
    Ok(Declarations::new(entries))
}

/// The `canon-trigger-history/1` document `canon trigger evaluate --history` reads: what the
/// evaluator recorded, its firings, activations and approvals.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerHistory {
    /// The document format, [`TRIGGER_HISTORY_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// Every recorded firing; empty when left out.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub firings: Vec<TriggerFiring>,
    /// Every activation of a rule key; empty when left out.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub activations: Vec<TriggerActivation>,
    /// Every approval of a rule key; empty when left out.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub approvals: Vec<TriggerApproval>,
}

/// One recorded firing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerFiring {
    /// The firing id the evaluator submitted.
    #[serde(deserialize_with = "super::present::required")]
    pub firing_id: String,
    /// The rule key, `<id>@<version>`.
    #[serde(deserialize_with = "super::present::required")]
    pub rule: String,
    /// When it fired.
    #[serde(deserialize_with = "super::present::required")]
    pub at: Instant,
}

/// One activation of a rule key, with the rows the evaluator listed as matching then, each written
/// `<kind>:<row id>`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerActivation {
    /// The rule key, `<id>@<version>`.
    #[serde(deserialize_with = "super::present::required")]
    pub rule: String,
    /// When the rule key was activated.
    #[serde(deserialize_with = "super::present::required")]
    pub at: Instant,
    /// The rows that matched at activation; empty when left out.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub baseline: Vec<String>,
}

/// One approval of a rule key: a `write` rule in mode `auto` is refused (`write-unapproved`)
/// unless the history holds one for its exact rule key. Canon reads `by` and never decides who
/// may approve.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerApproval {
    /// The rule key, `<id>@<version>`.
    #[serde(deserialize_with = "super::present::required")]
    pub rule: String,
    /// Who approved, as the evaluator recorded it.
    #[serde(deserialize_with = "super::present::required")]
    pub by: String,
    /// When.
    #[serde(deserialize_with = "super::present::required")]
    pub at: Instant,
}

/// The `canon-trigger-plan/1` document `canon trigger evaluate` prints
/// (`crate::trigger::render_plan`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerPlan {
    /// Always [`TRIGGER_PLAN_FORMAT`].
    pub format: String,
    /// One entry per match, ordered by rule key, kind and row id; written `[]` when nothing
    /// matches.
    pub entries: Vec<TriggerPlanEntry>,
}

/// One entry of a plan: a match, or a row whose match cannot be decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerPlanEntry {
    /// The rule key, `<id>@<version>`.
    pub rule: String,
    /// The state the rule set gives the rule.
    pub rule_state: TriggerRuleState,
    /// The rule's `subject.kind`, the snapshot kind the row is from.
    pub kind: TriggerKind,
    /// The row's `id`.
    pub row: TriggerRowId,
    /// The rule's `start.protocol`.
    pub protocol: ProtocolId,
    /// The input built from `start.input`.
    pub input: Declarations<InputId, Json>,
    /// The firing id: `<rule>/<kind>:<row>/<entered_at>` under `once_per_state_entry`,
    /// `<rule>/<kind>:<row>` under `once_per_subject`; absent for a row without `entered_at`
    /// under `once_per_state_entry`.
    pub firing_id: Option<String>,
    /// What a later step does with the match.
    pub verdict: TriggerVerdict,
    /// Why the entry is undetermined or refused.
    pub reason: Option<TriggerReason>,
    /// The declared field a `field-missing` or `field-type` reason names.
    pub field: Option<TriggerFieldName>,
}

/// The verdict of a plan entry, the first that applies, in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TriggerVerdict {
    /// Not to be started, for the entry's reason.
    Refused,
    /// The match cannot be decided, for the entry's reason.
    Undetermined,
    /// The row matched when the rule key was activated: listed, never started.
    Baseline,
    /// Not to be started now, for the entry's reason: it fired already, or the rule's cap for the
    /// day is reached.
    Held,
    /// A match of a rule in mode `offer`.
    Offer,
    /// A match of a rule in mode `auto`.
    WouldStart,
}

impl TriggerVerdict {
    /// The value the plan writes, its serde name (`would-start`).
    pub fn as_str(self) -> &'static str {
        match self {
            TriggerVerdict::Refused => "refused",
            TriggerVerdict::Undetermined => "undetermined",
            TriggerVerdict::Baseline => "baseline",
            TriggerVerdict::Held => "held",
            TriggerVerdict::Offer => "offer",
            TriggerVerdict::WouldStart => "would-start",
        }
    }
}

/// Why an entry is undetermined, refused or held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TriggerReason {
    /// A declared required field is absent or null.
    FieldMissing,
    /// A field holds a value of another type than declared.
    FieldType,
    /// The `where` evaluates to neither true nor false.
    WhereUnknown,
    /// The rule's state is `disabled`.
    Disabled,
    /// The rule's protocol is not in the protocols.
    UnknownProtocol,
    /// The protocol is guarded and the rule's mode is `auto`.
    GuardedAuto,
    /// The rule's `access` is not the protocol's class.
    AccessMismatch,
    /// A `write` rule in mode `auto` without an approval of its rule key.
    WriteUnapproved,
    /// The row has no `entered_at` under `once_per_state_entry`.
    EnteredAtMissing,
    /// A firing with the entry's firing id is in the history.
    AlreadyFired,
    /// The rule's firings on the UTC day of the evaluation instant have reached its cap.
    CapReached,
}

impl TriggerReason {
    /// The value the plan writes, its serde name (`field-missing`).
    pub fn as_str(self) -> &'static str {
        match self {
            TriggerReason::FieldMissing => "field-missing",
            TriggerReason::FieldType => "field-type",
            TriggerReason::WhereUnknown => "where-unknown",
            TriggerReason::Disabled => "disabled",
            TriggerReason::UnknownProtocol => "unknown-protocol",
            TriggerReason::GuardedAuto => "guarded-auto",
            TriggerReason::AccessMismatch => "access-mismatch",
            TriggerReason::WriteUnapproved => "write-unapproved",
            TriggerReason::EnteredAtMissing => "entered-at-missing",
            TriggerReason::AlreadyFired => "already-fired",
            TriggerReason::CapReached => "cap-reached",
        }
    }
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
