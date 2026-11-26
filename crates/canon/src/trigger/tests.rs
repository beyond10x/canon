use super::*;

/// A valid rule with every field, `{fields}`, `{input}` and `{where}` replaced as given.
fn document(fields: &str, input: &str, condition: &str) -> String {
    format!(
        "format: canon-trigger/1\nid: r\nversion: 3\nowner: o\nsubject:\n  kind: invoice\n  \
         state: {{field: status, is: overdue}}\n  fields:\n{fields}\nwhere: {condition}\nstart:\n  \
         protocol: p\n  input:\n{input}\nmode: offer\naccess: write\nrefire: once_per_subject\n\
         cap: 1\n"
    )
}

const FIELDS: &str = "    status: {type: string}\n    amount: {type: decimal}\n    \
                      note: {type: string, optional: true}";
const INPUT: &str = "    channel: {const: email}";

/// The `(code, field)` of each problem `read` reports.
fn named(source: &str) -> Vec<(&'static str, String)> {
    match read(source) {
        Ok(rule) => panic!("read as rule {}", rule.key()),
        Err(problems) => problems
            .into_iter()
            .map(|problem| (problem.code, problem.field))
            .collect(),
    }
}

fn one(code: &'static str, field: &str) -> Vec<(&'static str, String)> {
    vec![(code, field.to_owned())]
}

#[test]
fn a_rule_with_every_field_reads_with_its_key() {
    let rule = read(&document(
        FIELDS,
        INPUT,
        "'present_string($row.note) and $row.amount >= 1.5'",
    ))
    .expect("valid");
    assert_eq!(rule.key(), "r@3");
}

#[test]
fn an_entry_of_subject_fields_is_named_by_its_key() {
    let bad_type = "    status: {type: string}\n    amount: {type: float}";
    assert_eq!(
        named(&document(bad_type, INPUT, "'true'")),
        one("invalid-value", "subject.fields.amount.type")
    );
    let bad_key = "    status: {type: string}\n    amount: {typ: decimal}";
    assert_eq!(
        named(&document(bad_key, INPUT, "'true'")),
        one("unknown-field", "subject.fields.amount.typ")
    );
    let no_type = "    status: {type: string}\n    amount: {optional: true}";
    assert_eq!(
        named(&document(no_type, INPUT, "'true'")),
        one("missing-field", "subject.fields.amount.type")
    );
    let null = "    status: {type: string}\n    amount:";
    assert_eq!(
        named(&document(null, INPUT, "'true'")),
        one("invalid-value", "subject.fields.amount")
    );
}

#[test]
fn an_input_that_is_not_a_field_or_a_constant_is_named_by_its_key() {
    for input in [
        "    channel: '$row.status'",
        "    channel: {const: email, field: status}",
        "    channel: {}",
        "    channel: {expression: '$row.status'}",
        "    channel:",
    ] {
        assert_eq!(
            named(&document(FIELDS, input, "'true'")),
            one("invalid-input", "start.input.channel"),
            "{input}"
        );
    }
}

#[test]
fn a_where_is_refused_naming_where() {
    for condition in [
        "'lookup($row.amount)'",
        "'$row.amount'",
        "'$row.colour == 1'",
        "'$other == 1'",
        "'$row.note == \"x\"'",
        "'present_integer($row.note)'",
        "'contains([1, 2], $row.amount)'",
        "'$row.amount >'",
    ] {
        assert_eq!(
            named(&document(FIELDS, INPUT, condition)),
            one("invalid-where", "where"),
            "{condition}"
        );
    }
}

#[test]
fn the_catalog_holds_contains_and_presence_per_optional_type_and_canon_expr_accepts_it() {
    let fields = "    status: {type: string}\n    a: {type: string, optional: true}\n    \
                  b: {type: integer, optional: true}\n    c: {type: decimal, optional: true}\n    \
                  d: {type: bool, optional: true}\n    e: {type: integer}";
    let rule = read(&document(
        fields,
        INPUT,
        "'present_string($row.a) or present_integer($row.b) or present_decimal($row.c) or \
         present_bool($row.d) or $row.e > 0'",
    ))
    .expect("valid");
    let catalog = catalog(&rule);
    assert_eq!(
        catalog.functions.keys().collect::<Vec<_>>(),
        [
            "contains",
            "present_bool",
            "present_decimal",
            "present_integer",
            "present_string"
        ]
    );
    assert!(
        catalog
            .functions
            .values()
            .all(|function| matches!(function.implementation, Implementation::Pure(_)))
    );
    canon_expr::validate_catalog(&catalog).expect("canon-expr/1 accepts the catalog unchanged");

    let without_optional = read(&document(
        FIELDS.split("\n    note").next().unwrap(),
        INPUT,
        "'true'",
    ))
    .expect("valid");
    assert_eq!(
        catalog_names(&without_optional),
        ["contains"],
        "no optional field, no presence function"
    );
}

fn catalog_names(rule: &TriggerRule) -> Vec<String> {
    catalog(rule).functions.into_keys().collect()
}

#[test]
fn row_is_a_closed_record_of_the_declared_fields() {
    let rule = read(&document(FIELDS, INPUT, "'true'")).expect("valid");
    assert_eq!(
        row_type(&rule),
        canon_expr::Type::Record(BTreeMap::from([
            ("amount".to_owned(), canon_expr::Type::Decimal),
            (
                "note".to_owned(),
                canon_expr::Type::Optional(Box::new(canon_expr::Type::String))
            ),
            ("status".to_owned(), canon_expr::Type::String),
        ]))
    );
}

#[test]
fn a_document_without_a_mapping_or_yaml_is_a_parse_error() {
    for source in [
        "- a\n- b\n",
        "format: canon-trigger/1\n\tid: x\n",
        "a: 1\na: 2\n",
    ] {
        let problems = read(source).expect_err("refused");
        assert_eq!(problems.len(), 1, "{source}");
        assert_eq!(problems[0].code, "parse", "{source}");
        assert_eq!(problems[0].field, "", "{source}");
    }
    assert_eq!(named(""), one("missing-field", "format"));
}

#[test]
fn a_where_left_out_is_no_condition() {
    let source = document(FIELDS, INPUT, "'true'").replace("where: 'true'\n", "");
    let rule = read(&source).expect("valid");
    assert!(rule.r#where.is_none());
    assert!(compile_where(&rule).expect("compiles").is_none());
}

/// The compiled protocol of a fixture source.
fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

const LOOK_UP: &str = include_str!("../../../../fixtures/trigger/access/look-up.yaml");
const LOOK_UP_AGAIN: &str = include_str!("../../../../fixtures/trigger/access/look-up-again.yaml");
const PUBLISH: &str = include_str!("../../../../fixtures/trigger/access/publish.yaml");
const SEND_REMINDER: &str = include_str!("../../../../fixtures/trigger/send-reminder.yaml");

/// story:protocol-access-class: one entry per protocol, keyed and ordered by id whatever order the
/// set is given in, its inputs' values sorted; the document reads back as itself.
#[test]
fn the_protocols_of_a_set_are_keyed_by_id_with_class_guard_and_inputs() {
    let look = compiled(LOOK_UP);
    let publish = compiled(PUBLISH);
    let forward = protocols([("a", &look), ("b", &publish)]).expect("no duplicate");
    let backward = protocols([("b", &publish), ("a", &look)]).expect("no duplicate");
    assert_eq!(forward, backward);
    let ids: Vec<&str> = forward.protocols.ids().map(|id| id.as_str()).collect();
    assert_eq!(ids, ["look-up", "publish"]);
    let text = render_protocols(&forward);
    assert_eq!(
        text,
        "{\n  \"format\": \"canon-trigger-protocols/1\",\n  \"protocols\": {\n    \"look-up\": {\n      \
         \"class\": \"read_only\",\n      \"guarded\": false,\n      \"inputs\": {\n        \
         \"region\": [\n          \"east\",\n          \"west\"\n        ]\n      }\n    },\n    \
         \"publish\": {\n      \"class\": \"write\",\n      \"guarded\": true,\n      \
         \"inputs\": {}\n    }\n  }\n}\n"
    );
    let read_back: model::TriggerProtocols = serde_json::from_str(&text).expect("reads back");
    assert_eq!(read_back, forward);
    assert_eq!(render_protocols(&read_back), text);
    let reminder = protocol(&compiled(SEND_REMINDER));
    assert_eq!(reminder.class, model::AccessClass::Write);
    assert!(!reminder.guarded);
    assert_eq!(
        protocols(std::iter::empty())
            .map(|document| render_protocols(&document))
            .expect("empty set"),
        "{\n  \"format\": \"canon-trigger-protocols/1\",\n  \"protocols\": {}\n}\n"
    );
}

/// Two members of one protocol id are refused, naming the id and every member that declares it,
/// sorted.
#[test]
fn two_members_of_one_id_are_refused() {
    let one = compiled(LOOK_UP);
    let two = compiled(LOOK_UP_AGAIN);
    let other = compiled(PUBLISH);
    let refused =
        protocols([("z.yaml", &one), ("q.yaml", &other), ("b.yaml", &two)]).expect_err("duplicate");
    assert_eq!(DuplicateProtocolId::CODE, "duplicate-protocol-id");
    assert_eq!(refused.id.as_str(), "look-up");
    assert_eq!(
        refused.to_string(),
        "protocol `look-up` is declared by b.yaml and z.yaml"
    );
}

// Evaluation (story:trigger-evaluate). The meaning of each verdict is the conformance scenario
// conformance/scenarios/trigger-evaluate.yaml; these tests hold what it cannot state.

/// A rule over `invoice` rows in state `overdue`, with the given id, fields, `where` and refire
/// policy, in mode `auto`.
fn evaluated_rule(id: &str, fields: &str, condition: &str, refire: &str) -> RuleEntry {
    let source = format!(
        "format: canon-trigger/1\nid: {id}\nversion: 1\nowner: o\nsubject:\n  kind: invoice\n  \
         state: {{field: status, is: overdue}}\n  fields:\n{fields}\nwhere: {condition}\nstart:\n  \
         protocol: send-reminder\n  input:\n    note: {{field: note}}\n    channel: {{const: email}}\n\
         mode: auto\naccess: write\nrefire: {refire}\ncap: 1\n"
    );
    RuleEntry {
        rule: read(&source).expect("valid rule"),
        state: model::TriggerRuleState::Active,
    }
}

const EVALUATED_FIELDS: &str = "    status: {type: string}\n    amount: {type: integer}\n    \
                                note: {type: string, optional: true}";

fn snapshot(rows: &str) -> model::TriggerSnapshot {
    read_snapshot(&format!(
        "format: canon-trigger-snapshot/1\nkinds:\n  invoice:\n{rows}"
    ))
    .expect("snapshot reads")
}

fn no_protocols() -> model::TriggerProtocols {
    read_protocols("{format: canon-trigger-protocols/1, protocols: {}}").expect("protocols read")
}

fn history(text: &str) -> model::TriggerHistory {
    read_history(&format!("format: canon-trigger-history/1\n{text}")).expect("history reads")
}

fn now() -> model::Instant {
    model::Instant::new("2026-11-26T12:00:00Z")
}

/// The `(row, verdict, reason, field)` of each entry, in plan order.
fn verdicts(
    plan: &model::TriggerPlan,
) -> Vec<(String, &'static str, Option<&'static str>, Option<String>)> {
    plan.entries
        .iter()
        .map(|entry| {
            (
                entry.row.to_string(),
                entry.verdict.as_str(),
                entry.reason.map(model::TriggerReason::as_str),
                entry.field.as_ref().map(ToString::to_string),
            )
        })
        .collect()
}

/// The same arguments give the same bytes on a second run, and with the rules, the kinds, each
/// kind's rows, the firings and the activations in reverse order (CANON-TRIGGER-008).
#[test]
fn the_plan_is_byte_identical_over_reruns_and_any_input_order() {
    let rules = vec![
        evaluated_rule(
            "b",
            EVALUATED_FIELDS,
            "'$row.amount > 1'",
            "once_per_state_entry",
        ),
        evaluated_rule(
            "a",
            EVALUATED_FIELDS,
            "'$row.amount > 1'",
            "once_per_subject",
        ),
    ];
    let rows = "    - {id: r3, entered_at: 2026-11-20T03:00:00Z, fields: {status: overdue, amount: 5}}\n    \
                - {id: r1, entered_at: 2026-11-20T01:00:00Z, fields: {status: overdue, amount: 5}}\n    \
                - {id: r2, fields: {status: overdue, amount: 5, note: n}}\n  \
                other:\n    - {id: r1, fields: {status: overdue}}\n";
    let snapshot = snapshot(rows);
    let history = history(
        "firings:\n  - {firing_id: x, rule: a@1, at: 2026-11-01T00:00:00Z}\n  \
         - {firing_id: y, rule: b@1, at: 2026-11-02T00:00:00Z}\nactivations:\n  \
         - {rule: a@1, at: 2026-11-19T00:00:00Z, baseline: [invoice:r2]}\n  \
         - {rule: b@1, at: 2026-11-20T02:00:00Z}\n",
    );
    let first =
        render_plan(&evaluate(&rules, &snapshot, &no_protocols(), &history, &now()).unwrap());
    let again =
        render_plan(&evaluate(&rules, &snapshot, &no_protocols(), &history, &now()).unwrap());

    let mut reversed_rules = rules.clone();
    reversed_rules.reverse();
    let mut kinds: Vec<_> = snapshot
        .kinds
        .iter()
        .map(|(kind, rows)| (kind.clone(), rows.iter().rev().cloned().collect()))
        .collect();
    kinds.reverse();
    let reversed_snapshot = model::TriggerSnapshot {
        format: snapshot.format.clone(),
        kinds: model::Declarations::new(kinds),
    };
    let mut reversed_history = history.clone();
    reversed_history.firings.reverse();
    reversed_history.activations.reverse();
    let reversed = render_plan(
        &evaluate(
            &reversed_rules,
            &reversed_snapshot,
            &no_protocols(),
            &reversed_history,
            &now(),
        )
        .unwrap(),
    );
    assert_eq!(first, again);
    assert_eq!(first, reversed);
    // Rule a (once_per_subject, activated 11-19 listing r2): r1 and r3 entered later, r2 is
    // listed. Rule b (once_per_state_entry, activated 11-20T02): r1 entered before, r2 has no
    // entered_at, r3 entered after.
    let entries = evaluate(&rules, &snapshot, &no_protocols(), &history, &now()).unwrap();
    assert_eq!(
        verdicts(&entries),
        [
            ("r1".to_owned(), "would-start", None, None),
            ("r2".to_owned(), "baseline", None, None),
            ("r3".to_owned(), "would-start", None, None),
            ("r1".to_owned(), "baseline", None, None),
            ("r2".to_owned(), "refused", Some("entered-at-missing"), None),
            ("r3".to_owned(), "would-start", None, None),
        ]
    );
}

/// A row whose declared fields exceed `canon-expr/1`'s value bounds cannot be bound as `$row`: its
/// `where` is not true or false, so the row is undetermined, `where-unknown`, not dropped.
#[test]
fn a_where_that_cannot_be_evaluated_is_undetermined() {
    let rules = [evaluated_rule(
        "r",
        EVALUATED_FIELDS,
        "'present_string($row.note)'",
        "once_per_subject",
    )];
    let long = "x".repeat(canon_expr::MAX_SOURCE + 1);
    let snapshot = snapshot(&format!(
        "    - {{id: long, fields: {{status: overdue, amount: 1, note: {long}}}}}\n    \
         - {{id: short, fields: {{status: overdue, amount: 1, note: n}}}}\n    \
         - {{id: none, fields: {{status: overdue, amount: 1}}}}\n"
    ));
    let plan = evaluate(&rules, &snapshot, &no_protocols(), &history(""), &now()).unwrap();
    assert_eq!(
        verdicts(&plan),
        [
            (
                "long".to_owned(),
                "undetermined",
                Some("where-unknown"),
                None
            ),
            ("short".to_owned(), "would-start", None, None),
        ]
    );
}

/// An `integer` is a number written without fraction or exponent: `2.0` and `1e3` are
/// `field-type`; a `decimal` takes any number. A `{field}` input of an absent optional field is
/// null, of a present one its value as read.
#[test]
fn an_integer_has_no_fraction_and_an_absent_optional_input_is_null() {
    let rules = [evaluated_rule(
        "r",
        EVALUATED_FIELDS,
        "'$row.amount > 0'",
        "once_per_subject",
    )];
    let snapshot = snapshot(
        "    - {id: a, fields: {status: overdue, amount: 2.0}}\n    \
         - {id: b, fields: {status: overdue, amount: 1e3}}\n    \
         - {id: c, fields: {status: overdue, amount: -4}}\n    \
         - {id: d, fields: {status: overdue, amount: 7, note: late}}\n    \
         - {id: e, fields: {status: overdue, amount: 7}}\n",
    );
    let plan = evaluate(&rules, &snapshot, &no_protocols(), &history(""), &now()).unwrap();
    assert_eq!(
        verdicts(&plan),
        [
            (
                "a".to_owned(),
                "undetermined",
                Some("field-type"),
                Some("amount".to_owned())
            ),
            (
                "b".to_owned(),
                "undetermined",
                Some("field-type"),
                Some("amount".to_owned())
            ),
            ("d".to_owned(), "would-start", None, None),
            ("e".to_owned(), "would-start", None, None),
        ]
    );
    let notes: Vec<_> = plan
        .entries
        .iter()
        .map(|entry| entry.input.get(&model::InputId::new("note")).cloned())
        .collect();
    assert_eq!(
        notes,
        [
            Some(serde_json::Value::Null),
            Some(serde_json::Value::Null),
            Some(serde_json::Value::String("late".to_owned())),
            Some(serde_json::Value::Null),
        ]
    );
}

/// A disabled rule is evaluated like the others until story:trigger-holds-and-refusals refuses
/// it; its entries carry its state.
#[test]
fn every_rule_state_is_evaluated_and_carried() {
    let mut rules = Vec::new();
    for (id, state) in [
        ("d", model::TriggerRuleState::Draft),
        ("e", model::TriggerRuleState::Active),
        ("f", model::TriggerRuleState::Disabled),
    ] {
        let mut entry = evaluated_rule(id, EVALUATED_FIELDS, "'true'", "once_per_subject");
        entry.state = state;
        rules.push(entry);
    }
    let snapshot = snapshot("    - {id: a, fields: {status: overdue, amount: 1}}\n");
    let plan = evaluate(&rules, &snapshot, &no_protocols(), &history(""), &now()).unwrap();
    let states: Vec<_> = plan
        .entries
        .iter()
        .map(|entry| (entry.rule.as_str(), entry.rule_state.as_str()))
        .collect();
    assert_eq!(
        states,
        [("d@1", "draft"), ("e@1", "active"), ("f@1", "disabled")]
    );
}

/// The whole evaluation is refused for an instant that is not one anywhere it is read, and for a
/// document of another format.
#[test]
fn an_instant_that_is_not_one_and_a_wrong_format_are_refused() {
    let rules = [evaluated_rule(
        "r",
        EVALUATED_FIELDS,
        "'true'",
        "once_per_subject",
    )];
    let rows = snapshot("    - {id: a, fields: {status: overdue, amount: 1}}\n");
    let code = |snapshot: &model::TriggerSnapshot, history: &model::TriggerHistory, now: &str| {
        evaluate(
            &rules,
            snapshot,
            &no_protocols(),
            history,
            &model::Instant::new(now),
        )
        .expect_err("refused")
    };
    let refused = code(&rows, &history(""), "2026-11-26 12:00");
    assert_eq!(
        (refused.code, refused.message.as_str()),
        (
            "invalid-instant",
            "now: `2026-11-26 12:00` is not an instant (YYYY-MM-DDTHH:MM:SSZ, optionally with a \
             fraction)"
        )
    );
    let late = snapshot("    - {id: a, entered_at: tomorrow, fields: {status: overdue}}\n");
    assert_eq!(
        code(&late, &history(""), "2026-11-26T12:00:00Z").code,
        "invalid-instant"
    );
    let activated = history("activations:\n  - {rule: r@1, at: soon}\n");
    assert_eq!(
        code(&rows, &activated, "2026-11-26T12:00:00Z").code,
        "invalid-instant"
    );
    let fired = history("firings:\n  - {firing_id: x, rule: r@1, at: soon}\n");
    assert_eq!(
        code(&rows, &fired, "2026-11-26T12:00:00Z").code,
        "invalid-instant"
    );
    let wrong = read_snapshot("format: canon-trigger-snapshot/2\nkinds: {}\n").expect_err("format");
    assert_eq!(
        (wrong.code, wrong.message.as_str()),
        (
            "malformed-input",
            "format is `canon-trigger-snapshot/2`, expected `canon-trigger-snapshot/1`"
        )
    );
    let unknown =
        read_history("format: canon-trigger-history/1\napprovals: []\n").expect_err("field");
    assert_eq!(unknown.code, "malformed-input");
}

/// A row id is an identifier, as the firing id embeds it: one with whitespace refuses the
/// snapshot, naming the row.
#[test]
fn a_row_id_with_whitespace_is_refused() {
    let rules = [evaluated_rule(
        "r",
        EVALUATED_FIELDS,
        "'true'",
        "once_per_subject",
    )];
    let rows = snapshot("    - {id: 'inv 1', fields: {status: overdue, amount: 1}}\n");
    let refused = evaluate(&rules, &rows, &no_protocols(), &history(""), &now()).expect_err("id");
    assert_eq!(
        (refused.code, refused.message.as_str()),
        (
            "malformed-input",
            "snapshot: row `invoice:inv 1`: a kind and a row id are identifiers, without whitespace"
        )
    );
}

/// The plan writes a verdict and a reason as their serde name, the one the reference documents.
#[test]
fn a_verdict_and_a_reason_are_written_as_their_documented_name() {
    use crate::model::{TriggerReason, TriggerVerdict};
    for verdict in [
        TriggerVerdict::Refused,
        TriggerVerdict::Undetermined,
        TriggerVerdict::Baseline,
        TriggerVerdict::Offer,
        TriggerVerdict::WouldStart,
    ] {
        assert_eq!(serde_json::json!(verdict), verdict.as_str());
    }
    for reason in [
        TriggerReason::FieldMissing,
        TriggerReason::FieldType,
        TriggerReason::WhereUnknown,
        TriggerReason::EnteredAtMissing,
    ] {
        assert_eq!(serde_json::json!(reason), reason.as_str());
    }
}
