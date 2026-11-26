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
/// policy, in mode `auto`, `read_only` against [`reader_protocols`] (so it needs no approval), cap
/// 50.
fn evaluated_rule(id: &str, fields: &str, condition: &str, refire: &str) -> RuleEntry {
    let source = format!(
        "format: canon-trigger/1\nid: {id}\nversion: 1\nowner: o\nsubject:\n  kind: invoice\n  \
         state: {{field: status, is: overdue}}\n  fields:\n{fields}\nwhere: {condition}\nstart:\n  \
         protocol: send-reminder\n  input:\n    note: {{field: note}}\n    channel: {{const: email}}\n\
         mode: auto\naccess: read_only\nrefire: {refire}\ncap: 50\n"
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

/// `send-reminder`, the protocol [`evaluated_rule`] starts, read-only and not guarded.
fn reader_protocols() -> model::TriggerProtocols {
    read_protocols(
        "{format: canon-trigger-protocols/1, protocols: {send-reminder: {class: read_only, \
         guarded: false, inputs: {}}}}",
    )
    .expect("protocols read")
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
        render_plan(&evaluate(&rules, &snapshot, &reader_protocols(), &history, &now()).unwrap());
    let again =
        render_plan(&evaluate(&rules, &snapshot, &reader_protocols(), &history, &now()).unwrap());

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
            &reader_protocols(),
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
    let entries = evaluate(&rules, &snapshot, &reader_protocols(), &history, &now()).unwrap();
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
    let plan = evaluate(&rules, &snapshot, &reader_protocols(), &history(""), &now()).unwrap();
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
    let plan = evaluate(&rules, &snapshot, &reader_protocols(), &history(""), &now()).unwrap();
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

/// Every rule state is evaluated and carried by its entries; a disabled rule's entries are refused,
/// `disabled` (story:trigger-holds-and-refusals).
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
    let plan = evaluate(&rules, &snapshot, &reader_protocols(), &history(""), &now()).unwrap();
    let states: Vec<_> = plan
        .entries
        .iter()
        .map(|entry| (entry.rule.as_str(), entry.rule_state.as_str()))
        .collect();
    assert_eq!(
        states,
        [("d@1", "draft"), ("e@1", "active"), ("f@1", "disabled")]
    );
    assert_eq!(
        verdicts(&plan),
        [
            ("a".to_owned(), "would-start", None, None),
            ("a".to_owned(), "would-start", None, None),
            ("a".to_owned(), "refused", Some("disabled"), None),
        ]
    );
}

// Holds and refusals (story:trigger-holds-and-refusals). Each reason is a step of the conformance
// scenario conformance/scenarios/trigger-holds-and-refusals.yaml; these tests hold the loop an
// evaluator runs, the order of the input, and how firings are counted against the cap.

/// A rule over `invoice` rows in state `overdue` reading only `status`, with no input.
fn rule_with(
    id: &str,
    protocol: &str,
    mode: &str,
    access: &str,
    refire: &str,
    cap: u32,
) -> RuleEntry {
    let (id, version) = id.split_once('@').expect("a rule key");
    let source = format!(
        "format: canon-trigger/1\nid: {id}\nversion: {version}\nowner: o\nsubject: {{kind: invoice, \
         state: {{field: status, is: overdue}}, fields: {{status: {{type: string}}}}}}\nstart: \
         {{protocol: {protocol}}}\nmode: {mode}\naccess: {access}\nrefire: {refire}\ncap: {cap}\n"
    );
    RuleEntry {
        rule: read(&source).expect("valid rule"),
        state: model::TriggerRuleState::Active,
    }
}

/// `writer` writes, `reader` only reads, `guarded` writes and is guarded; written in the given
/// order.
fn protocols_in(order: &[&str]) -> model::TriggerProtocols {
    let entry = |id: &str| match id {
        "writer" => "writer: {class: write, guarded: false, inputs: {}}",
        "reader" => "reader: {class: read_only, guarded: false, inputs: {}}",
        "guarded" => "guarded: {class: write, guarded: true, inputs: {}}",
        other => panic!("no protocol {other}"),
    };
    let entries: Vec<&str> = order.iter().map(|id| entry(id)).collect();
    read_protocols(&format!(
        "{{format: canon-trigger-protocols/1, protocols: {{{}}}}}",
        entries.join(", ")
    ))
    .expect("protocols read")
}

/// A history of the given firings and approvals, each a flow mapping, in the given order.
fn history_of(firings: &[&str], approvals: &[&str]) -> model::TriggerHistory {
    history(&format!(
        "firings: [{}]\napprovals: [{}]\n",
        firings.join(", "),
        approvals.join(", ")
    ))
}

const THREE_ROWS: &str = "    - {id: a, entered_at: 2026-11-20T01:00:00Z, fields: {status: overdue}}\n    \
                          - {id: b, entered_at: 2026-11-20T02:00:00Z, fields: {status: overdue}}\n    \
                          - {id: c, entered_at: 2026-11-20T03:00:00Z, fields: {status: overdue}}\n";

/// An evaluator's loop: it submits the `would-start` entries, records their firings and evaluates
/// again; then every match is held, `already-fired` (CANON-TRIGGER-014).
#[test]
fn evaluating_again_with_the_plans_firings_holds_every_match() {
    let rules = [rule_with(
        "again@1",
        "writer",
        "auto",
        "write",
        "once_per_state_entry",
        50,
    )];
    let rows = snapshot(THREE_ROWS);
    let protocols = protocols_in(&["writer"]);
    let approval = ["{rule: again@1, by: someone, at: 2026-11-19T00:00:00Z}"];
    let mut recorded = history_of(&[], &approval);
    let first = evaluate(&rules, &rows, &protocols, &recorded, &now()).unwrap();
    let starts: Vec<_> = verdicts(&first)
        .into_iter()
        .map(|(row, verdict, ..)| (row, verdict))
        .collect();
    assert_eq!(
        starts,
        [
            ("a".to_owned(), "would-start"),
            ("b".to_owned(), "would-start"),
            ("c".to_owned(), "would-start"),
        ]
    );
    for entry in &first.entries {
        recorded.firings.push(model::TriggerFiring {
            firing_id: entry.firing_id.clone().expect("a firing id"),
            rule: entry.rule.clone(),
            at: now(),
        });
    }
    let second = evaluate(&rules, &rows, &protocols, &recorded, &now()).unwrap();
    let held = |row: &str| (row.to_owned(), "held", Some("already-fired"), None);
    assert_eq!(verdicts(&second), [held("a"), held("b"), held("c")]);
}

/// The holds and refusals of one input give the same bytes whatever the order of the rules, the
/// rows, the protocols, the firings and the approvals; the cap is counted in plan order, not input
/// order (CANON-TRIGGER-008, CANON-TRIGGER-015).
#[test]
fn holds_and_refusals_do_not_depend_on_input_order() {
    let mut rules = vec![
        rule_with("capped@1", "writer", "auto", "write", "once_per_subject", 2),
        rule_with(
            "unapproved@2",
            "writer",
            "auto",
            "write",
            "once_per_subject",
            50,
        ),
        rule_with(
            "guarded@1",
            "guarded",
            "auto",
            "write",
            "once_per_subject",
            50,
        ),
        rule_with(
            "mismatch@1",
            "reader",
            "offer",
            "write",
            "once_per_subject",
            50,
        ),
        rule_with(
            "missing@1",
            "absent",
            "auto",
            "read_only",
            "once_per_subject",
            50,
        ),
        rule_with(
            "offered@1",
            "writer",
            "offer",
            "write",
            "once_per_subject",
            50,
        ),
    ];
    let mut off = rule_with("off@1", "writer", "auto", "write", "once_per_subject", 50);
    off.state = model::TriggerRuleState::Disabled;
    rules.push(off);
    let rows = snapshot(THREE_ROWS);
    let mut firings = vec![
        "{firing_id: capped@1/invoice:z, rule: capped@1, at: 2026-11-26T01:00:00Z}",
        "{firing_id: capped@1/invoice:a, rule: capped@1, at: 2026-11-20T01:00:00Z}",
    ];
    let mut approvals = vec![
        "{rule: capped@1, by: someone, at: 2026-11-19T00:00:00Z}",
        "{rule: unapproved@1, by: someone, at: 2026-11-19T00:00:00Z}",
        "{rule: guarded@1, by: someone, at: 2026-11-19T00:00:00Z}",
        "{rule: off@1, by: someone, at: 2026-11-19T00:00:00Z}",
    ];
    let mut order = vec!["writer", "reader", "guarded"];
    let forward = evaluate(
        &rules,
        &rows,
        &protocols_in(&order),
        &history_of(&firings, &approvals),
        &now(),
    )
    .unwrap();

    rules.reverse();
    let reversed_rows = snapshot(
        &THREE_ROWS
            .split_inclusive('\n')
            .rev()
            .map(|line| format!("    {}", line.trim_start()))
            .collect::<String>(),
    );
    firings.reverse();
    approvals.reverse();
    order.reverse();
    let backward = evaluate(
        &rules,
        &reversed_rows,
        &protocols_in(&order),
        &history_of(&firings, &approvals),
        &now(),
    )
    .unwrap();
    assert_eq!(render_plan(&forward), render_plan(&backward));

    let entry = |row: &str, verdict: &'static str, reason: Option<&'static str>| {
        (row.to_owned(), verdict, reason, None)
    };
    let refused = |row: &str, reason: &'static str| entry(row, "refused", Some(reason));
    let mut expected = vec![
        // capped@1: one firing today (z); the firing of a on 2026-11-20 is another day but holds
        // a's firing id. b would start and fills the cap of 2; c is held.
        entry("a", "held", Some("already-fired")),
        entry("b", "would-start", None),
        entry("c", "held", Some("cap-reached")),
    ];
    for reason in [
        "guarded-auto",
        "access-mismatch",
        "unknown-protocol",
        "disabled",
    ] {
        expected.extend(["a", "b", "c"].map(|row| refused(row, reason)));
    }
    expected.extend(["a", "b", "c"].map(|row| entry(row, "offer", None)));
    expected.extend(["a", "b", "c"].map(|row| refused(row, "write-unapproved")));
    let rule_order: Vec<&str> = forward
        .entries
        .iter()
        .map(|entry| entry.rule.as_str())
        .collect();
    assert_eq!(
        rule_order,
        [
            "capped@1",
            "capped@1",
            "capped@1",
            "guarded@1",
            "guarded@1",
            "guarded@1",
            "mismatch@1",
            "mismatch@1",
            "mismatch@1",
            "missing@1",
            "missing@1",
            "missing@1",
            "off@1",
            "off@1",
            "off@1",
            "offered@1",
            "offered@1",
            "offered@1",
            "unapproved@2",
            "unapproved@2",
            "unapproved@2",
        ]
    );
    assert_eq!(verdicts(&forward), expected);
}

/// A firing is its firing id: one recorded twice counts once against the cap. Firings of the rule
/// key on another UTC day, or of another rule key, do not count; one at the first instant of the
/// day does.
#[test]
fn a_firing_recorded_twice_counts_once_against_the_cap() {
    let rules = [rule_with(
        "capped@1",
        "writer",
        "auto",
        "write",
        "once_per_subject",
        2,
    )];
    let rows = snapshot(THREE_ROWS);
    let history = history_of(
        &[
            "{firing_id: capped@1/invoice:z, rule: capped@1, at: 2026-11-26T00:00:00Z}",
            "{firing_id: capped@1/invoice:z, rule: capped@1, at: 2026-11-26T00:00:01Z}",
            "{firing_id: capped@1/invoice:y, rule: capped@1, at: 2026-11-25T23:59:59.999Z}",
            "{firing_id: capped@1/invoice:x, rule: capped@1, at: 2026-11-27T00:00:00Z}",
            "{firing_id: capped@2/invoice:w, rule: capped@2, at: 2026-11-26T08:00:00Z}",
        ],
        &["{rule: capped@1, by: someone, at: 2026-11-19T00:00:00Z}"],
    );
    let plan = evaluate(&rules, &rows, &protocols_in(&["writer"]), &history, &now()).unwrap();
    assert_eq!(
        verdicts(&plan),
        [
            ("a".to_owned(), "would-start", None, None),
            ("b".to_owned(), "held", Some("cap-reached"), None),
            ("c".to_owned(), "held", Some("cap-reached"), None),
        ]
    );
}

/// An offer is held at the cap like a start, but does not count against it: only `would-start`
/// entries of the plan do.
#[test]
fn an_offer_is_held_at_the_cap_and_does_not_count_against_it() {
    let rules = [rule_with(
        "offered@1",
        "writer",
        "offer",
        "write",
        "once_per_subject",
        1,
    )];
    let rows = snapshot(THREE_ROWS);
    let open = evaluate(
        &rules,
        &rows,
        &protocols_in(&["writer"]),
        &history_of(&[], &[]),
        &now(),
    )
    .unwrap();
    let offers: Vec<_> = verdicts(&open).into_iter().map(|entry| entry.1).collect();
    assert_eq!(offers, ["offer", "offer", "offer"]);
    let full = evaluate(
        &rules,
        &rows,
        &protocols_in(&["writer"]),
        &history_of(
            &["{firing_id: offered@1/invoice:z, rule: offered@1, at: 2026-11-26T09:00:00Z}"],
            &[],
        ),
        &now(),
    )
    .unwrap();
    let held: Vec<_> = verdicts(&full)
        .into_iter()
        .map(|entry| (entry.1, entry.2))
        .collect();
    assert_eq!(held, [("held", Some("cap-reached")); 3]);
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
            &reader_protocols(),
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
    let unknown = read_history("format: canon-trigger-history/1\ngrants: []\n").expect_err("field");
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
    let refused =
        evaluate(&rules, &rows, &reader_protocols(), &history(""), &now()).expect_err("id");
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
        TriggerVerdict::Held,
        TriggerVerdict::Offer,
        TriggerVerdict::WouldStart,
    ] {
        assert_eq!(serde_json::json!(verdict), verdict.as_str());
    }
    for reason in [
        TriggerReason::FieldMissing,
        TriggerReason::FieldType,
        TriggerReason::WhereUnknown,
        TriggerReason::Disabled,
        TriggerReason::UnknownProtocol,
        TriggerReason::GuardedAuto,
        TriggerReason::AccessMismatch,
        TriggerReason::WriteUnapproved,
        TriggerReason::EnteredAtMissing,
        TriggerReason::AlreadyFired,
        TriggerReason::CapReached,
    ] {
        assert_eq!(serde_json::json!(reason), reason.as_str());
    }
}

/// A protocols argument that lists one protocol id twice would make the plan depend on its order:
/// the evaluation is refused, `duplicate-protocol-id`.
#[test]
fn a_protocol_listed_twice_is_refused() {
    let rules = [rule_with(
        "r@1",
        "writer",
        "offer",
        "write",
        "once_per_subject",
        1,
    )];
    let mut protocols = protocols_in(&["writer", "reader"]);
    let mut entries: Vec<_> = protocols
        .protocols
        .iter()
        .map(|(id, protocol)| (id.clone(), protocol.clone()))
        .collect();
    entries.push((model::ProtocolId::new("writer"), entries[1].1.clone()));
    protocols.protocols = model::Declarations::new(entries);
    let refused = evaluate(
        &rules,
        &snapshot(THREE_ROWS),
        &protocols,
        &history_of(&[], &[]),
        &now(),
    )
    .expect_err("refused");
    assert_eq!(
        (refused.code, refused.message.as_str()),
        (
            "duplicate-protocol-id",
            "protocols: protocol `writer` is listed twice"
        )
    );
}

// story:trigger-rule-publication: the digest of a rule, the check of a rule set before it is
// published, and the input values an evaluation checks.

/// A rule `<id>@<version>` over `invoice` rows with the declared `fields`, starting `protocol`
/// with `input` (flow mappings), in `mode`.
fn published_rule(key: &str, fields: &str, protocol: &str, input: &str, mode: &str) -> String {
    let (id, version) = key.split_once('@').expect("a rule key");
    format!(
        "format: canon-trigger/1\nid: {id}\nversion: {version}\nowner: o\nsubject: {{kind: invoice, \
         state: {{field: status, is: overdue}}, fields: {{status: {{type: string}}{fields}}}}}\n\
         start: {{protocol: {protocol}, input: {{{input}}}}}\nmode: {mode}\naccess: write\n\
         refire: once_per_subject\ncap: 5\n"
    )
}

/// `send` writes and takes `channel` (email, letter) and `tone` (calm); `guarded` writes, is
/// guarded and takes nothing.
fn publication_protocols() -> model::TriggerProtocols {
    read_protocols(
        "{format: canon-trigger-protocols/1, protocols: {send: {class: write, guarded: false, \
         inputs: {channel: [email, letter], tone: [calm]}}, guarded: {class: write, guarded: true, \
         inputs: {}}}}",
    )
    .expect("protocols read")
}

fn index(entries: &str) -> model::TriggerIndex {
    read_index(&format!(
        "{{format: canon-trigger-index/1, rules: {{{entries}}}}}"
    ))
    .expect("index reads")
}

/// Each source is named `<n>` by its position, counted from 1.
fn named_rules(sources: &[String]) -> Vec<(String, TriggerRule)> {
    sources
        .iter()
        .enumerate()
        .map(|(at, source)| ((at + 1).to_string(), read(source).expect("valid rule")))
        .collect()
}

/// The `(code, rule key)` of each failure, in order.
fn failures(
    sources: &[String],
    published: &model::TriggerIndex,
) -> Vec<(&'static str, Option<String>)> {
    match check_set(&named_rules(sources), published, &publication_protocols()) {
        Ok(index) => panic!("passed: {}", render_index(&index)),
        Err(failures) => failures
            .into_iter()
            .map(|failure| (failure.code, failure.rule))
            .collect(),
    }
}

const MAPPED: &str = "channel: {const: email}, tone: {const: calm}";

/// The canonical form is the RFC 8785 JSON of every field of the rule: keys sorted, no
/// whitespace, the declared fields a list in declaration order, `optional` and `input` written
/// when left out, `where` only when the rule has one.
#[test]
fn the_canonical_form_of_a_rule_is_its_rfc_8785_json() {
    let rule = read(&published_rule(
        "r@3",
        ", amount: {type: decimal, optional: true}",
        "send",
        MAPPED,
        "offer",
    ))
    .expect("valid");
    assert_eq!(
        canonical_form(&rule),
        "{\"access\":\"write\",\"cap\":5,\"format\":\"canon-trigger/1\",\"id\":\"r\",\
         \"mode\":\"offer\",\"owner\":\"o\",\"refire\":\"once_per_subject\",\"start\":{\
         \"input\":{\"channel\":{\"const\":\"email\"},\"tone\":{\"const\":\"calm\"}},\
         \"protocol\":\"send\"},\"subject\":{\"fields\":[{\"name\":\"status\",\
         \"optional\":false,\"type\":\"string\"},{\"name\":\"amount\",\"optional\":true,\
         \"type\":\"decimal\"}],\"kind\":\"invoice\",\"state\":{\"field\":\"status\",\
         \"is\":\"overdue\"}},\"version\":3}"
    );
    let with_where = read(
        &published_rule("r@3", "", "send", MAPPED, "offer")
            .replace("mode: offer", "where: '$row.status == \"x\"'\nmode: offer"),
    )
    .expect("valid");
    assert!(
        canonical_form(&with_where)
            .ends_with(",\"version\":3,\"where\":\"$row.status == \\\"x\\\"\"}"),
        "{}",
        canonical_form(&with_where)
    );
}

/// The layout, the key order, comments and defaults written out change no digest; a change of any
/// field, the order of the declared fields included, changes it (CANON-TRIGGER-016).
#[test]
fn the_digest_changes_with_the_rule_and_not_with_its_text() {
    let base = published_rule("r@1", ", amount: {type: decimal}", "send", MAPPED, "offer");
    let digest_of = |source: &str| digest(&read(source).expect(source));
    let original = digest_of(&base);
    assert!(
        original.starts_with("sha256:") && original.len() == 7 + 64,
        "{original}"
    );
    let same = [
        format!("# a comment\n{base}"),
        base.replace(
            "amount: {type: decimal}",
            "amount: {optional: false, type: decimal}",
        ),
        base.replace("cap: 5\n", "cap: 5 # five\n")
            .replace("owner: o\n", "")
            .replace(
                "format: canon-trigger/1\n",
                "format: canon-trigger/1\nowner: \"o\"\n",
            ),
    ];
    for source in &same {
        assert_eq!(digest_of(source), original, "{source}");
    }
    let no_input = published_rule("r@1", "", "send", "", "offer");
    assert_eq!(
        digest_of(&no_input),
        digest_of(&no_input.replace(", input: {}", "")),
        "an absent start.input is {{}}"
    );
    let changed = [
        base.replace("version: 1", "version: 2"),
        base.replace("owner: o", "owner: p"),
        base.replace("cap: 5", "cap: 6"),
        base.replace("mode: offer", "mode: auto"),
        base.replace("access: write", "access: read_only"),
        base.replace("once_per_subject", "once_per_state_entry"),
        base.replace("kind: invoice", "kind: bill"),
        base.replace("is: overdue", "is: late"),
        base.replace("type: decimal", "type: integer"),
        base.replace(
            "amount: {type: decimal}",
            "amount: {type: decimal, optional: true}",
        ),
        base.replace(
            "{status: {type: string}, amount: {type: decimal}}",
            "{amount: {type: decimal}, status: {type: string}}",
        ),
        base.replace("protocol: send", "protocol: other"),
        base.replace("const: email", "const: letter"),
        base.replace("{const: email}", "{field: status}"),
        base.replace("mode: offer", "where: 'true'\nmode: offer"),
    ];
    let mut seen = std::collections::BTreeSet::from([original.clone()]);
    for source in &changed {
        assert!(seen.insert(digest_of(source)), "unchanged digest: {source}");
    }
}

/// Two versions of one id pass; the set's index lists each with its digest, in key order,
/// whatever the order of the set; the index holding a key with its own digest passes.
#[test]
fn a_set_that_passes_gives_its_own_index_in_any_order() {
    let one = published_rule("stale@1", "", "send", MAPPED, "offer");
    let two = published_rule("stale@2", "", "send", MAPPED, "auto");
    let digests = |source: &String| digest(&read(source).expect("valid"));
    let expected = format!(
        "{{\n  \"format\": \"canon-trigger-index/1\",\n  \"rules\": {{\n    \"stale@1\": \"{}\",\n    \
         \"stale@2\": \"{}\"\n  }}\n}}\n",
        digests(&one),
        digests(&two)
    );
    let published = index(&format!("stale@1: '{}'", digests(&one)));
    for set in [[one.clone(), two.clone()], [two.clone(), one.clone()]] {
        let checked =
            check_set(&named_rules(&set), &published, &publication_protocols()).expect("passes");
        assert_eq!(render_index(&checked), expected);
    }
}

/// Every failure is reported, in rule-key order and within a key in the order of the checks, each
/// naming its key; a key two documents share is `duplicate-key` and nothing else
/// (CANON-TRIGGER-017 to 020).
#[test]
fn every_failure_names_its_rule_key_in_check_order() {
    let edited = published_rule("a@1", "", "send", MAPPED, "offer");
    let published = index("a@1: sha256:0000");
    let everything = published_rule(
        "b@1",
        ", amount: {type: integer}",
        "send",
        "zone: {const: x}, channel: {field: amount}",
        "offer",
    );
    let guarded = published_rule("c@1", "", "guarded", "", "auto");
    let unknown = published_rule("d@1", "", "nowhere", "anything: {const: x}", "auto");
    let twice = published_rule("e@1", "", "send", MAPPED, "offer");
    let twice_other = published_rule("e@1", "", "guarded", "", "auto");
    let fine = published_rule(
        "f@1",
        ", kind: {type: string}",
        "send",
        "channel: {field: kind}, tone: {const: calm}",
        "offer",
    );
    let set = [
        fine.clone(),
        twice_other.clone(),
        unknown.clone(),
        everything.clone(),
        guarded.clone(),
        twice.clone(),
        edited.clone(),
    ];
    let key = |key: &str| Some(key.to_owned());
    let expected = vec![
        ("key-republished", key("a@1")),
        ("unknown-input", key("b@1")),
        ("missing-input", key("b@1")),
        ("input-value-not-allowed", key("b@1")),
        ("guarded-auto", key("c@1")),
        ("unknown-protocol", key("d@1")),
        ("duplicate-key", key("e@1")),
    ];
    assert_eq!(failures(&set, &published), expected);
    let mut reversed = set.to_vec();
    reversed.reverse();
    assert_eq!(failures(&reversed, &published), expected);

    let rules = named_rules(&set);
    let all = check_set(&rules, &published, &publication_protocols()).expect_err("fails");
    let duplicate = all.last().expect("a failure");
    assert_eq!(
        duplicate.to_string(),
        "e@1: the rule key is given by 2 documents: 2, 6"
    );
}

/// A constant among the input's values passes and one outside fails; a `{field}` input of a
/// declared string passes here, one of another type fails (CANON-TRIGGER-020).
#[test]
fn an_input_value_is_checked_against_the_values_of_the_input() {
    let fields = ", s: {type: string}, n: {type: integer}, b: {type: bool}, d: {type: decimal}, \
                  o: {type: string, optional: true}";
    let nothing = index("");
    let passes = |input: &str| {
        check_set(
            &named_rules(&[published_rule("r@1", fields, "send", input, "offer")]),
            &nothing,
            &publication_protocols(),
        )
        .is_ok()
    };
    for input in [
        "channel: {const: letter}, tone: {const: calm}",
        "channel: {field: s}, tone: {const: calm}",
        "channel: {field: o}, tone: {field: s}",
    ] {
        assert!(passes(input), "{input}");
    }
    for input in [
        "channel: {const: fax}, tone: {const: calm}",
        "channel: {const: Email}, tone: {const: calm}",
        "channel: {field: n}, tone: {const: calm}",
        "channel: {field: b}, tone: {const: calm}",
        "channel: {field: d}, tone: {const: calm}",
    ] {
        assert_eq!(
            failures(
                &[published_rule("r@1", fields, "send", input, "offer")],
                &nothing
            ),
            [("input-value-not-allowed", Some("r@1".to_owned()))],
            "{input}"
        );
    }
}

/// An index or protocols document of another format, a key the index lists twice and a protocol
/// id the protocols list twice refuse the whole set, naming no rule.
#[test]
fn a_malformed_index_or_protocols_refuses_the_whole_set() {
    let rules = named_rules(&[published_rule("r@1", "", "send", MAPPED, "offer")]);
    let code = |published: &model::TriggerIndex, protocols: &model::TriggerProtocols| {
        let failures = check_set(&rules, published, protocols).expect_err("refused");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].rule, None);
        failures[0].code
    };
    let mut other = index("");
    other.format = "canon-trigger-index/2".to_owned();
    assert_eq!(code(&other, &publication_protocols()), "malformed-input");
    let twice = model::TriggerIndex {
        format: model::TRIGGER_INDEX_FORMAT.to_owned(),
        rules: model::Declarations::new(vec![
            ("r@1".to_owned(), "sha256:a".to_owned()),
            ("r@1".to_owned(), "sha256:a".to_owned()),
        ]),
    };
    assert_eq!(code(&twice, &publication_protocols()), "malformed-input");
    let mut protocols = publication_protocols();
    let mut entries: Vec<_> = protocols
        .protocols
        .iter()
        .map(|(id, protocol)| (id.clone(), protocol.clone()))
        .collect();
    entries.push(entries[0].clone());
    protocols.protocols = model::Declarations::new(entries);
    assert_eq!(code(&index(""), &protocols), "duplicate-protocol-id");
    assert!(read_index("format: canon-trigger-index/1\nrules: {}\nextra: 1\n").is_err());
}

/// At evaluation, the value of an input the protocol declares must be among its values: a row
/// value outside them, absent or not a string is refused `input-value-not-allowed`, after
/// `entered-at-missing`; an input the protocol does not declare is not checked
/// (CANON-TRIGGER-021).
#[test]
fn an_evaluated_input_value_outside_the_inputs_values_is_refused_last() {
    let rule = |input: &str, refire: &str| RuleEntry {
        rule: read(
            &published_rule(
                "v@1",
                ", channel: {type: string, optional: true}",
                "send",
                input,
                "offer",
            )
            .replace("once_per_subject", refire),
        )
        .expect("valid"),
        state: model::TriggerRuleState::Active,
    };
    let rows = snapshot(
        "    - {id: a, entered_at: 2026-11-20T01:00:00Z, fields: {status: overdue, channel: email}}\n    \
         - {id: b, entered_at: 2026-11-20T02:00:00Z, fields: {status: overdue, channel: fax}}\n    \
         - {id: c, entered_at: 2026-11-20T03:00:00Z, fields: {status: overdue}}\n    \
         - {id: d, fields: {status: overdue, channel: fax}}\n",
    );
    let protocols = publication_protocols();
    let reasons = |entries: &[RuleEntry]| {
        verdicts(&evaluate(entries, &rows, &protocols, &history(""), &now()).expect("plan"))
            .into_iter()
            .map(|(row, verdict, reason, _)| (row, verdict, reason))
            .collect::<Vec<_>>()
    };
    let refused = Some("input-value-not-allowed");
    assert_eq!(
        reasons(&[rule(
            "channel: {field: channel}, tone: {const: calm}, other: {const: x}",
            "once_per_state_entry"
        )]),
        [
            ("a".to_owned(), "offer", None),
            ("b".to_owned(), "refused", refused),
            ("c".to_owned(), "refused", refused),
            ("d".to_owned(), "refused", Some("entered-at-missing")),
        ]
    );
    assert_eq!(
        reasons(&[rule(
            "channel: {field: channel}, tone: {const: loud}",
            "once_per_subject"
        )]),
        [
            ("a".to_owned(), "refused", refused),
            ("b".to_owned(), "refused", refused),
            ("c".to_owned(), "refused", refused),
            ("d".to_owned(), "refused", refused),
        ]
    );
}
