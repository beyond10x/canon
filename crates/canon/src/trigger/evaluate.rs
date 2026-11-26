//! Evaluating a rule set against a snapshot (story:trigger-evaluate, `docs/design/trigger-rules.md`
//! § 6): [`evaluate`], the readers of the documents it reads, and [`render_plan`]. What it decides
//! is written in the module docs of `crate::trigger`, which the reference page is built from.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use canon_expr::{Bindings, CheckedExpr, EvalContext, Truth};
use serde::de::DeserializeOwned;

use super::{ROW, check, compile_where, first_of_each};
use crate::ir::json::Value;
use crate::model::{
    self, Declarations, Instant, Json, TRIGGER_HISTORY_FORMAT, TRIGGER_PLAN_FORMAT,
    TRIGGER_PROTOCOLS_FORMAT, TRIGGER_SET_FORMAT, TRIGGER_SNAPSHOT_FORMAT, TriggerFieldName,
    TriggerFieldType, TriggerHistory, TriggerInput, TriggerKind, TriggerMode, TriggerPlan,
    TriggerPlanEntry, TriggerProtocols, TriggerReason, TriggerRefire, TriggerRow, TriggerRule,
    TriggerRuleState, TriggerSet, TriggerSnapshot, TriggerVerdict,
};

/// One rule of the set [`evaluate`] reads, with its lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleEntry {
    pub rule: TriggerRule,
    pub state: TriggerRuleState,
}

/// Why a document, or a whole evaluation, is refused: its code and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub code: &'static str,
    pub message: String,
}

impl Refusal {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Refusal {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// Reads a `canon-trigger-set/1` document.
pub fn read_set(source: &str) -> Result<TriggerSet, Refusal> {
    read(source, TRIGGER_SET_FORMAT, |set: &TriggerSet| &set.format)
}

/// Reads a `canon-trigger-snapshot/1` document.
pub fn read_snapshot(source: &str) -> Result<TriggerSnapshot, Refusal> {
    read(
        source,
        TRIGGER_SNAPSHOT_FORMAT,
        |snapshot: &TriggerSnapshot| &snapshot.format,
    )
}

/// Reads a `canon-trigger-history/1` document.
pub fn read_history(source: &str) -> Result<TriggerHistory, Refusal> {
    read(
        source,
        TRIGGER_HISTORY_FORMAT,
        |history: &TriggerHistory| &history.format,
    )
}

/// Reads a `canon-trigger-protocols/1` document, as `canon trigger protocols` prints it.
pub fn read_protocols(source: &str) -> Result<TriggerProtocols, Refusal> {
    read(
        source,
        TRIGGER_PROTOCOLS_FORMAT,
        |protocols: &TriggerProtocols| &protocols.format,
    )
}

/// Reads one document of `format` from YAML or JSON text: refused as `malformed-input` when it does
/// not read as one, or names another format.
fn read<T: DeserializeOwned>(
    source: &str,
    format: &str,
    format_of: impl Fn(&T) -> &str,
) -> Result<T, Refusal> {
    let document: T = serde_yaml_ng::from_str(source).map_err(|error| {
        Refusal::new(
            "malformed-input",
            format!(
                "not a {format} document: {}",
                model::one_line(&error.to_string())
            ),
        )
    })?;
    expect_format("", format_of(&document), format)?;
    Ok(document)
}

/// Refuses a document whose `format` is not `format`, naming it by `what` when not empty.
fn expect_format(what: &str, found: &str, format: &str) -> Result<(), Refusal> {
    if found == format {
        return Ok(());
    }
    let prefix = if what.is_empty() {
        String::new()
    } else {
        format!("{what}: ")
    };
    Err(Refusal::new(
        "malformed-input",
        format!(
            "{prefix}format is `{}`, expected `{format}`",
            model::one_line(found)
        ),
    ))
}

fn invalid_instant(what: &str, instant: &Instant) -> Refusal {
    Refusal::new(
        "invalid-instant",
        format!(
            "{what}: `{}` is not an instant (YYYY-MM-DDTHH:MM:SSZ, optionally with a fraction)",
            model::one_line(instant.as_str())
        ),
    )
}

/// What the history says about one rule key: the start of its latest activation, and every row an
/// activation of it lists.
struct Activation<'a> {
    latest: i128,
    listed: BTreeSet<&'a str>,
}

/// Whether a row of a rule's kind matches, or cannot be decided and why.
enum Assessment {
    Match,
    Undetermined(TriggerReason, Option<TriggerFieldName>),
}

/// Evaluates a rule set against a snapshot (see the module docs of `crate::trigger`): every match
/// of every rule, and every row whose match cannot be decided, with its firing id and verdict,
/// ordered by rule key, kind and row id. A pure function of its arguments: `now` is the only time
/// it reads, and the order of the rules, kinds, rows, firings and activations changes no byte of
/// the plan [`render_plan`] writes.
///
/// Refused as a whole, the first in this order deciding: `now` that is not an instant
/// (`invalid-instant`); a snapshot, protocols or history document of another format
/// (`malformed-input`); a rule that is not a valid `canon-trigger/1` rule (`invalid-rule`), in
/// rule-key order; two rules of one rule key (`duplicate-rule`); a kind listed twice, or a row id
/// listed twice within its kind (`duplicate-row`), a kind or row id that is not an identifier
/// (`malformed-input`), or an `entered_at` that is not an instant (`invalid-instant`), in kind and
/// row-id order; a firing's or an activation's `at` that is not an instant (`invalid-instant`).
pub fn evaluate(
    rules: &[RuleEntry],
    snapshot: &TriggerSnapshot,
    protocols: &TriggerProtocols,
    history: &TriggerHistory,
    now: &Instant,
) -> Result<TriggerPlan, Refusal> {
    let Some(now_seconds) = now.seconds() else {
        return Err(invalid_instant("now", now));
    };
    expect_format("snapshot", &snapshot.format, TRIGGER_SNAPSHOT_FORMAT)?;
    expect_format("protocols", &protocols.format, TRIGGER_PROTOCOLS_FORMAT)?;
    expect_format("history", &history.format, TRIGGER_HISTORY_FORMAT)?;

    let mut sorted: Vec<(String, &RuleEntry)> = rules
        .iter()
        .map(|entry| (entry.rule.key(), entry))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, entry) in &sorted {
        if let Some(problem) = check(&entry.rule).first() {
            return Err(Refusal::new(
                "invalid-rule",
                format!(
                    "rule `{}`: error[{}]: {problem}",
                    model::one_line(key),
                    problem.code
                ),
            ));
        }
    }
    if let Some(pair) = sorted.windows(2).find(|pair| pair[0].0 == pair[1].0) {
        return Err(Refusal::new(
            "duplicate-rule",
            format!("rule `{}` is given twice", model::one_line(&pair[0].0)),
        ));
    }

    let kinds = rows_by_kind(snapshot)?;
    let activations = activations(history)?;

    let mut entries = Vec::new();
    for (key, entry) in &sorted {
        let rule = &entry.rule;
        let checked = compile_where(rule).map_err(|diagnostic| {
            Refusal::new(
                "invalid-rule",
                format!("rule `{}`: where: {diagnostic}", model::one_line(key)),
            )
        })?;
        let Some(rows) = kinds.get(&rule.subject.kind) else {
            continue;
        };
        for row in rows {
            let context = format!("{}:{}", rule.subject.kind, row.id);
            let Some(assessment) = assess(rule, checked.as_ref(), row, key, &context, now_seconds)
            else {
                continue;
            };
            entries.push(plan_entry(
                entry,
                key,
                row,
                assessment,
                activations.get(key.as_str()),
            ));
        }
    }
    entries.sort_by(|a, b| {
        (a.rule.as_str(), &a.kind, &a.row).cmp(&(b.rule.as_str(), &b.kind, &b.row))
    });
    Ok(TriggerPlan {
        format: TRIGGER_PLAN_FORMAT.to_owned(),
        entries,
    })
}

/// The rows of each kind, each kind's rows in row-id order; refuses a kind or a row id listed
/// twice, a kind or row id that is not an identifier, and an `entered_at` that is not an instant,
/// checked in kind and row-id order.
fn rows_by_kind(
    snapshot: &TriggerSnapshot,
) -> Result<BTreeMap<&TriggerKind, Vec<&TriggerRow>>, Refusal> {
    let mut names: Vec<&TriggerKind> = snapshot.kinds.ids().collect();
    names.sort();
    if let Some(pair) = names.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(Refusal::new(
            "duplicate-row",
            format!(
                "snapshot: kind `{}` is listed twice",
                model::one_line(pair[0].as_str())
            ),
        ));
    }
    let mut kinds = BTreeMap::new();
    for (kind, rows) in snapshot.kinds.iter() {
        let mut rows: Vec<&TriggerRow> = rows.iter().collect();
        rows.sort_by(|a, b| a.id.cmp(&b.id));
        kinds.insert(kind, rows);
    }
    for (kind, rows) in &kinds {
        if let Some(pair) = rows.windows(2).find(|pair| pair[0].id == pair[1].id) {
            return Err(Refusal::new(
                "duplicate-row",
                format!(
                    "snapshot: row `{}:{}` is listed twice",
                    model::one_line(kind.as_str()),
                    model::one_line(pair[0].id.as_str())
                ),
            ));
        }
        for row in rows {
            if !model::is_identifier(kind.as_str()) || !model::is_identifier(row.id.as_str()) {
                return Err(Refusal::new(
                    "malformed-input",
                    format!(
                        "snapshot: row `{}:{}`: a kind and a row id are identifiers, without \
                         whitespace",
                        model::one_line(kind.as_str()),
                        model::one_line(row.id.as_str())
                    ),
                ));
            }
            if let Some(entered_at) = &row.entered_at
                && entered_at.interval().is_none()
            {
                let what = format!(
                    "snapshot: row `{}:{}`: entered_at",
                    model::one_line(kind.as_str()),
                    model::one_line(row.id.as_str())
                );
                return Err(invalid_instant(&what, entered_at));
            }
        }
    }
    Ok(kinds)
}

/// Each rule key's activations; refuses a firing's or an activation's `at` that is not an instant,
/// checked in sorted order.
fn activations(history: &TriggerHistory) -> Result<BTreeMap<&str, Activation<'_>>, Refusal> {
    let mut firings: Vec<_> = history.firings.iter().collect();
    firings.sort_by(|a, b| (&a.firing_id, &a.rule, &a.at).cmp(&(&b.firing_id, &b.rule, &b.at)));
    for firing in firings {
        if firing.at.interval().is_none() {
            let what = format!(
                "history: firing `{}`: at",
                model::one_line(&firing.firing_id)
            );
            return Err(invalid_instant(&what, &firing.at));
        }
    }
    let mut sorted: Vec<_> = history.activations.iter().collect();
    sorted.sort_by(|a, b| (&a.rule, &a.at).cmp(&(&b.rule, &b.at)));
    let mut activations: BTreeMap<&str, Activation<'_>> = BTreeMap::new();
    for activation in sorted {
        let Some((start, _)) = activation.at.interval() else {
            let what = format!(
                "history: activation of `{}`: at",
                model::one_line(&activation.rule)
            );
            return Err(invalid_instant(&what, &activation.at));
        };
        let entry = activations
            .entry(activation.rule.as_str())
            .or_insert(Activation {
                latest: start,
                listed: BTreeSet::new(),
            });
        entry.latest = entry.latest.max(start);
        entry
            .listed
            .extend(activation.baseline.iter().map(String::as_str));
    }
    Ok(activations)
}

/// The row's value of a field, `None` when it is absent or null.
fn present<'a>(row: &'a TriggerRow, field: &TriggerFieldName) -> Option<&'a Json> {
    row.fields.get(field).filter(|value| !value.is_null())
}

/// Whether a row of the rule's kind matches the rule (`Some(Match)`), does not (`None`), or cannot
/// be decided: its state field absent or not a string, then, for a row in the state, a declared
/// required field absent or a declared field of another type, in declaration order, without calling
/// `canon-expr/1`; then a `where` that is not true or false.
fn assess(
    rule: &TriggerRule,
    checked: Option<&CheckedExpr>,
    row: &TriggerRow,
    source: &str,
    context: &str,
    now: i64,
) -> Option<Assessment> {
    let state = &rule.subject.state;
    match present(row, &state.field) {
        None => {
            return Some(Assessment::Undetermined(
                TriggerReason::FieldMissing,
                Some(state.field.clone()),
            ));
        }
        Some(Json::String(value)) if *value == state.is => {}
        Some(Json::String(_)) => return None,
        Some(_) => {
            return Some(Assessment::Undetermined(
                TriggerReason::FieldType,
                Some(state.field.clone()),
            ));
        }
    }
    let mut record = BTreeMap::new();
    for (name, field) in rule.subject.fields.iter() {
        if record.contains_key(name.as_str()) {
            continue;
        }
        let bound = match (present(row, name), field.optional) {
            (None, true) => canon_expr::Value::Optional(None),
            (None, false) => {
                return Some(Assessment::Undetermined(
                    TriggerReason::FieldMissing,
                    Some(name.clone()),
                ));
            }
            (Some(value), optional) => match typed(value, field.r#type) {
                Some(value) if optional => canon_expr::Value::Optional(Some(Box::new(value))),
                Some(value) => value,
                None => {
                    return Some(Assessment::Undetermined(
                        TriggerReason::FieldType,
                        Some(name.clone()),
                    ));
                }
            },
        };
        record.insert(name.as_str().to_owned(), bound);
    }
    let Some(checked) = checked else {
        return Some(Assessment::Match);
    };
    match truth(checked, record, source, context, now) {
        Truth::True => Some(Assessment::Match),
        Truth::False => None,
        Truth::Unknown => Some(Assessment::Undetermined(TriggerReason::WhereUnknown, None)),
    }
}

/// A row's value as the `canon-expr/1` value of its declared type, or `None` when it is not of
/// that type: a string for `string`, a number with no fraction or exponent for `integer`, any
/// number for `decimal`, `true` or `false` for `bool`.
fn typed(value: &Json, field_type: TriggerFieldType) -> Option<canon_expr::Value> {
    match field_type {
        TriggerFieldType::String => value
            .as_str()
            .map(|text| canon_expr::Value::String(text.to_owned())),
        TriggerFieldType::Bool => value.as_bool().map(canon_expr::Value::Bool),
        TriggerFieldType::Integer => {
            let text = value.as_number()?.to_string();
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            canon_expr::Value::from_json(value, &canon_expr::Type::Integer).ok()
        }
        TriggerFieldType::Decimal => {
            value.as_number()?;
            canon_expr::Value::from_json(value, &canon_expr::Type::Decimal).ok()
        }
    }
}

/// The truth of a rule's `where` with `$row` bound to the record; `Unknown` when `canon-expr/1`
/// says so, or cannot evaluate it (a value over its bounds).
fn truth(
    checked: &CheckedExpr,
    record: BTreeMap<String, canon_expr::Value>,
    source: &str,
    context: &str,
    now: i64,
) -> Truth {
    let bindings = Bindings {
        values: BTreeMap::from([(ROW.to_owned(), canon_expr::Value::Record(record))]),
        source_identity: source.to_owned(),
        context_identity: context.to_owned(),
    };
    let Ok(plan) = canon_expr::plan(checked.clone(), &bindings) else {
        return Truth::Unknown;
    };
    let evaluation = EvalContext {
        now,
        source_identity: source.to_owned(),
        context_identity: context.to_owned(),
    };
    canon_expr::evaluate(&plan, &[], &evaluation).map_or(Truth::Unknown, |report| report.truth)
}

/// The plan entry of one assessed row: its firing id, its input and its verdict, the first that
/// applies.
fn plan_entry(
    entry: &RuleEntry,
    key: &str,
    row: &TriggerRow,
    assessment: Assessment,
    activation: Option<&Activation<'_>>,
) -> TriggerPlanEntry {
    let rule = &entry.rule;
    let kind = &rule.subject.kind;
    let (verdict, reason, field) = if let Some(reason) = refusal(rule, row) {
        (TriggerVerdict::Refused, Some(reason), None)
    } else if let Assessment::Undetermined(reason, field) = assessment {
        (TriggerVerdict::Undetermined, Some(reason), field)
    } else if is_baseline(activation, kind, row) {
        (TriggerVerdict::Baseline, None, None)
    } else {
        // The fourth verdict, `held`, is decided here by story:trigger-holds-and-refusals: a
        // firing with this firing id in the history (`already-fired`), or the rule's firings on
        // the UTC day of `now` at its cap (`cap-reached`).
        match rule.mode {
            TriggerMode::Offer => (TriggerVerdict::Offer, None, None),
            TriggerMode::Auto => (TriggerVerdict::WouldStart, None, None),
        }
    };
    let input = rule
        .start
        .input
        .iter()
        .map(|(id, input)| {
            let value = match input {
                TriggerInput::Field(field) => row.fields.get(field).cloned().unwrap_or(Json::Null),
                TriggerInput::Const(value) => Json::String(value.clone()),
            };
            (id.clone(), value)
        })
        .collect();
    TriggerPlanEntry {
        rule: key.to_owned(),
        rule_state: entry.state,
        kind: kind.clone(),
        row: row.id.clone(),
        protocol: rule.start.protocol.clone(),
        input: Declarations::new(input),
        firing_id: firing_id(rule, key, row),
        verdict,
        reason,
        field,
    }
}

/// `<rule key>/<kind>:<row id>/<entered_at>` under `once_per_state_entry`, `None` without
/// `entered_at`; `<rule key>/<kind>:<row id>` under `once_per_subject`. `entered_at` is written as
/// the snapshot writes it.
fn firing_id(rule: &TriggerRule, key: &str, row: &TriggerRow) -> Option<String> {
    let subject = format!("{key}/{}:{}", rule.subject.kind, row.id);
    match rule.refire {
        TriggerRefire::OncePerStateEntry => row
            .entered_at
            .as_ref()
            .map(|entered_at| format!("{subject}/{entered_at}")),
        TriggerRefire::OncePerSubject => Some(subject),
    }
}

/// Why an entry is refused, the first verdict. This story decides one reason, a row without
/// `entered_at` under `once_per_state_entry` (`entered-at-missing`); story:trigger-holds-and-refusals
/// adds the others here, ahead of it: the rule `disabled`, its protocol not in `protocols`, a
/// guarded protocol in mode `auto`, an `access` that differs from the protocol's class, and a
/// `write` rule in mode `auto` without an approval in the history.
fn refusal(rule: &TriggerRule, row: &TriggerRow) -> Option<TriggerReason> {
    (rule.refire == TriggerRefire::OncePerStateEntry && row.entered_at.is_none())
        .then_some(TriggerReason::EnteredAtMissing)
}

/// Whether the rule key's latest activation is at or after the row's `entered_at` (compared by
/// the start of the interval each instant names), or an activation lists the row as
/// `<kind>:<row id>`.
fn is_baseline(activation: Option<&Activation<'_>>, kind: &TriggerKind, row: &TriggerRow) -> bool {
    let Some(activation) = activation else {
        return false;
    };
    let entered = row
        .entered_at
        .as_ref()
        .and_then(Instant::interval)
        .map(|(start, _)| start);
    entered.is_some_and(|entered| activation.latest >= entered)
        || activation
            .listed
            .contains(format!("{kind}:{}", row.id).as_str())
}

/// The canonical JSON of a `canon-trigger-plan/1` document, as `canon-ir/1` is written: keys in
/// code-point order, two-space indent, one trailing newline. `firing_id`, `reason` and `field` are
/// written only when the entry has them.
pub fn render_plan(plan: &TriggerPlan) -> String {
    let entries = plan
        .entries
        .iter()
        .map(|entry| {
            let input = entry
                .input
                .iter()
                .map(|(id, value)| (id.as_str().to_owned(), Value::json(value)));
            let mut members = vec![
                ("input", first_of_each(input)),
                ("kind", Value::string(entry.kind.as_str())),
                ("protocol", Value::string(entry.protocol.as_str())),
                ("row", Value::string(entry.row.as_str())),
                ("rule", Value::string(entry.rule.as_str())),
                ("rule_state", Value::string(entry.rule_state.as_str())),
                ("verdict", Value::string(entry.verdict.as_str())),
            ];
            if let Some(firing_id) = &entry.firing_id {
                members.push(("firing_id", Value::string(firing_id.as_str())));
            }
            if let Some(reason) = entry.reason {
                members.push(("reason", Value::string(reason.as_str())));
            }
            if let Some(field) = &entry.field {
                members.push(("field", Value::string(field.as_str())));
            }
            Value::object(members)
        })
        .collect();
    Value::object([
        ("entries", Value::Array(entries)),
        ("format", Value::string(plan.format.as_str())),
    ])
    .render()
}
