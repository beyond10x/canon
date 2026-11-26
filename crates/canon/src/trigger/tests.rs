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
