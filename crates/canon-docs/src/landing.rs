//! The inputs the docs-system product landing page renders: a `b10x-protocol-graph/1` document of
//! the investigation protocol, built from Canon's own compile, and a `b10x-terminal/1` recording
//! of real `canon` runs. Both formats forbid extra keys, so neither file carries the generated-file
//! marker; `website/data/` belongs to `canon-docs` as a whole instead.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, OutcomeRequirement, Predicate, Truth};

use crate::json::{self, Json, obj};

/// The directory every file of which `canon-docs` generates.
pub const DATA_DIR: &str = "website/data";
pub const GRAPH_FILE: &str = "investigation.protocol-graph.json";
pub const TERMINAL_FILE: &str = "canon.terminal.json";

const FIXTURE: &str = "fixtures/investigation/protocol.yaml";
/// How many lines of `canon compile` output the recording keeps.
const COMPILE_LINES: usize = 8;

/// Node kinds in the order the graph format sorts them.
const NODE_ORDER: [&str; 5] = ["action", "evidence", "claim", "outcome", "obligation"];
/// Edge kinds in the order this generator writes them.
const EDGE_ORDER: [&str; 5] = ["produces", "establishes", "supports", "requires", "gates"];

/// A predicate in the `canon-ir/1` shape the graph format keeps, defaults left out: an evidence
/// match without a result has no `result`, and a claim test for `true` has no `is`.
fn predicate(predicate: &Predicate) -> Json {
    match predicate {
        Predicate::All(members) => obj([(
            "all",
            Json::Arr(members.iter().map(self::predicate).collect()),
        )]),
        Predicate::Any(members) => obj([(
            "any",
            Json::Arr(members.iter().map(self::predicate).collect()),
        )]),
        Predicate::Not(inner) => obj([("not", self::predicate(inner))]),
        Predicate::Evidence(matching) => {
            let mut members = vec![("kind".to_owned(), json::str(matching.kind.as_str()))];
            if let Some(result) = &matching.result {
                members.push(("result".to_owned(), json::str(result.clone())));
            }
            if let Some(subject) = &matching.subject {
                members.push(("subject".to_owned(), json::str(subject.as_str())));
            }
            obj([("evidence", Json::Obj(members))])
        }
        Predicate::Claim(test) => {
            let mut members = vec![("id".to_owned(), json::str(test.claim.as_str()))];
            if test.is != Truth::True {
                members.push(("is".to_owned(), json::str(test.is.to_string())));
            }
            obj([("claim", Json::Obj(members))])
        }
    }
}

/// The evidence matches (kind, result) and the claim tests (claim, value) inside a predicate.
type Leaves = (Vec<(String, Option<String>)>, Vec<(String, Truth)>);

/// Every evidence match and claim test inside a predicate.
fn leaves(predicate: &Predicate) -> Leaves {
    let mut evidence = Vec::new();
    let mut claims = Vec::new();
    predicate.visit(&mut |node| match node {
        Predicate::Evidence(matching) => {
            evidence.push((matching.kind.as_str().to_owned(), matching.result.clone()))
        }
        Predicate::Claim(test) => claims.push((test.claim.as_str().to_owned(), test.is)),
        _ => {}
    });
    (evidence, claims)
}

fn is_empty_all(predicate: &Predicate) -> bool {
    matches!(predicate, Predicate::All(members) if members.is_empty())
}

struct Node {
    kind: &'static str,
    name: String,
    members: Vec<(String, Json)>,
}

/// One edge: kind, from, to and an optional qualifier.
type Edge = (&'static str, String, String, Option<String>);

fn qualifier(value: Truth) -> Option<String> {
    (value != Truth::True).then(|| value.to_string())
}

/// The protocol graph of a compiled protocol.
pub fn graph(compiled: &Ir, source: &str) -> Json {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: BTreeSet<(usize, String, String, Option<String>)> = BTreeSet::new();
    let mut edge = |(kind, from, to, qualifier): Edge| {
        let rank = EDGE_ORDER
            .iter()
            .position(|known| *known == kind)
            .unwrap_or(EDGE_ORDER.len());
        edges.insert((rank, from, to, qualifier));
    };
    let described = |description: &Option<String>| -> Vec<(String, Json)> {
        description
            .iter()
            .map(|text| ("description".to_owned(), json::str(text.clone())))
            .collect()
    };

    for (id, action) in &compiled.actions {
        let mut members = described(&action.description);
        if !is_empty_all(&action.precondition) {
            members.push(("predicate".to_owned(), predicate(&action.precondition)));
            for (claim, value) in leaves(&action.precondition).1 {
                edge((
                    "gates",
                    format!("claim:{claim}"),
                    format!("action:{id}"),
                    qualifier(value),
                ));
            }
        }
        if let Some(effect) = &action.effect {
            members.push(("effect".to_owned(), json::str(effect.as_str())));
        }
        if !action.requires.is_empty() {
            members.push((
                "capabilities".to_owned(),
                Json::Arr(
                    action
                        .requires
                        .iter()
                        .map(|c| json::str(c.as_str()))
                        .collect(),
                ),
            ));
        }
        for kind in &action.may_produce {
            edge((
                "produces",
                format!("action:{id}"),
                format!("evidence:{kind}"),
                None,
            ));
        }
        nodes.push(Node {
            kind: "action",
            name: id.as_str().to_owned(),
            members,
        });
    }
    for (id, kind) in &compiled.evidence_kinds {
        nodes.push(Node {
            kind: "evidence",
            name: id.as_str().to_owned(),
            members: described(&kind.description),
        });
    }
    for (id, claim) in &compiled.claims {
        let mut members = described(&claim.description);
        members.push(("predicate".to_owned(), predicate(&claim.true_when)));
        let (evidence, claims) = leaves(&claim.true_when);
        for (kind, result) in evidence {
            edge((
                "establishes",
                format!("evidence:{kind}"),
                format!("claim:{id}"),
                result,
            ));
        }
        for (other, value) in claims {
            edge((
                "supports",
                format!("claim:{other}"),
                format!("claim:{id}"),
                qualifier(value),
            ));
        }
        nodes.push(Node {
            kind: "claim",
            name: id.as_str().to_owned(),
            members,
        });
    }
    for (id, outcome) in &compiled.outcomes {
        let mut members = described(&outcome.description);
        match &outcome.requires {
            OutcomeRequirement::Predicate(requires) => {
                members.push(("predicate".to_owned(), predicate(requires)));
                for (claim, value) in leaves(requires).1 {
                    edge((
                        "requires",
                        format!("claim:{claim}"),
                        format!("outcome:{id}"),
                        qualifier(value),
                    ));
                }
            }
            OutcomeRequirement::Decision(name) => {
                members.push(("decision".to_owned(), json::str(name.as_str())));
            }
        }
        nodes.push(Node {
            kind: "outcome",
            name: id.as_str().to_owned(),
            members,
        });
    }
    for (id, obligation) in &compiled.obligations {
        let mut members = described(&obligation.description);
        members.push((
            "predicate".to_owned(),
            predicate(&obligation.discharged_when),
        ));
        nodes.push(Node {
            kind: "obligation",
            name: id.as_str().to_owned(),
            members,
        });
    }
    nodes.sort_by(|a, b| {
        let rank = |kind: &str| NODE_ORDER.iter().position(|known| *known == kind);
        rank(a.kind)
            .cmp(&rank(b.kind))
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut protocol = vec![
        ("id".to_owned(), json::str(compiled.protocol.id.as_str())),
        ("revision".to_owned(), Json::Int(compiled.protocol.revision)),
    ];
    if let Some(description) = &compiled.protocol.description {
        protocol.push(("description".to_owned(), json::str(description.clone())));
    }
    protocol.push(("source".to_owned(), json::str(source)));
    obj([
        ("format", json::str("b10x-protocol-graph/1")),
        ("protocol", Json::Obj(protocol)),
        (
            "nodes",
            Json::Arr(
                nodes
                    .into_iter()
                    .map(|node| {
                        let mut members = vec![
                            (
                                "id".to_owned(),
                                json::str(format!("{}:{}", node.kind, node.name)),
                            ),
                            ("kind".to_owned(), json::str(node.kind)),
                            ("name".to_owned(), json::str(node.name)),
                        ];
                        members.extend(node.members);
                        Json::Obj(members)
                    })
                    .collect(),
            ),
        ),
        (
            "edges",
            Json::Arr(
                edges
                    .into_iter()
                    .map(|(rank, from, to, qualifier)| {
                        let mut members = vec![
                            ("from".to_owned(), json::str(from)),
                            ("to".to_owned(), json::str(to)),
                            ("kind".to_owned(), json::str(EDGE_ORDER[rank])),
                        ];
                        if let Some(qualifier) = qualifier {
                            members.push(("qualifier".to_owned(), json::str(qualifier)));
                        }
                        Json::Obj(members)
                    })
                    .collect(),
            ),
        ),
    ])
}

/// The protocol graph of the investigation fixture, compiled by Canon.
pub fn investigation_graph(root: &Path) -> Result<String, String> {
    let path = root.join(FIXTURE);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("reading {}: {error}", path.display()))?;
    let protocol = model::parse(&text).map_err(|error| format!("{FIXTURE}: {error}"))?;
    let compiled = ir::compile(&protocol).map_err(|problems| {
        let shown: Vec<String> = problems.iter().map(ToString::to_string).collect();
        format!("{FIXTURE} does not compile: {}", shown.join("; "))
    })?;
    Ok(graph(&compiled, &format!("canon compile {FIXTURE}")).pretty())
}

/// What one recorded command printed and how it exited.
pub struct Run {
    pub output: String,
    pub exit_code: i32,
}

/// The recorded terminal session. `run` runs `canon` with the given arguments from the repository
/// root; `version` is what `canon --version` printed.
pub fn terminal(
    run: impl Fn(&[&str]) -> Result<Run, String>,
    version: &str,
) -> Result<String, String> {
    let entry = |command: String, run: Run| {
        obj([
            ("command", json::str(command)),
            ("output", json::str(run.output.trim_end().to_owned())),
            (
                "exitCode",
                Json::Int(u64::try_from(run.exit_code).unwrap_or(u64::MAX)),
            ),
        ])
    };
    let validate = run(&["validate", "--path", FIXTURE])?;
    let compiled = run(&["compile", "--path", FIXTURE])?;
    let head: Vec<&str> = compiled.output.lines().take(COMPILE_LINES).collect();
    let compiled = Run {
        output: head.join("\n"),
        exit_code: compiled.exit_code,
    };
    let conform = run(&["conform", "run"])?;
    if validate.exit_code != 0 || compiled.exit_code != 0 || conform.exit_code != 0 {
        return Err(
            "a recorded canon command failed; the recording shows only passing runs".to_owned(),
        );
    }
    Ok(obj([
        ("format", json::str("b10x-terminal/1")),
        ("title", json::str("canon / repository root")),
        (
            "entries",
            Json::Arr(vec![
                entry(format!("canon validate --path {FIXTURE}"), validate),
                entry(
                    format!("canon compile --path {FIXTURE} | head -n {COMPILE_LINES}"),
                    compiled,
                ),
                entry("canon conform run".to_owned(), conform),
            ]),
        ),
        (
            "recordedWith",
            json::str(format!(
                "{}, recorded by canon-docs from the repository root",
                version.trim()
            )),
        ),
        (
            "tones",
            Json::Obj(
                TONES
                    .iter()
                    .map(|(word, tone)| ((*word).to_owned(), json::str(*tone)))
                    .collect(),
            ),
        ),
    ])
    .pretty())
}

/// The output words `canon` prints and the tone the terminal renderer gives every occurrence: a
/// verdict that passed reads as true, one that failed or a rejection as false.
const TONES: [(&str, &str); 4] = [
    ("valid:", "true"),
    ("passed:", "true"),
    ("failed:", "false"),
    ("error", "false"),
];

/// Runs the `canon` binary at `canon` from `root`, recording standard output then standard error.
pub fn run_canon(canon: &Path, root: &Path, args: &[&str]) -> Result<Run, String> {
    let output = Command::new(canon)
        .args(args)
        .current_dir(root)
        .env("NO_COLOR", "1")
        .output()
        .map_err(|error| format!("running {} {}: {error}", canon.display(), args.join(" ")))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(Run {
        output: text,
        exit_code: output.status.code().unwrap_or(-1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The graph keeps an evidence match's subject, and writes none for a match without one.
    #[test]
    fn an_evidence_match_keeps_its_subject_in_the_graph() {
        let protocol = model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\nclaims:\n  \
             bound: {true_when: {evidence: {kind: k, result: pass, subject: a}}}\n  \
             unbound: {true_when: {evidence: {kind: k, result: pass}}}\n",
        )
        .expect("parses");
        let rendered: Vec<String> = protocol
            .claims
            .iter()
            .map(|(_, claim)| predicate(&claim.true_when).pretty())
            .collect();
        assert!(
            rendered[0].contains("\"subject\": \"a\""),
            "{}",
            rendered[0]
        );
        assert!(!rendered[1].contains("subject"), "{}", rendered[1]);
    }

    #[test]
    fn the_terminal_keeps_the_head_of_compile_and_every_line_of_the_rest() {
        let recorded = terminal(
            |args| {
                Ok(Run {
                    output: match args[0] {
                        "compile" => (1..=30).map(|n| format!("line {n}\n")).collect(),
                        other => format!("{other} output\n"),
                    },
                    exit_code: 0,
                })
            },
            "canon 0.0.0\n",
        )
        .expect("records");
        assert!(recorded.contains("\"format\": \"b10x-terminal/1\""));
        assert!(recorded.contains("line 8\""));
        assert!(!recorded.contains("line 9"));
        assert!(recorded.contains("\"output\": \"conform output\""));
        assert!(recorded.contains("\"recordedWith\": \"canon 0.0.0, recorded by canon-docs"));
        let compact: String = recorded.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains(
            "\"tones\":{\"valid:\":\"true\",\"passed:\":\"true\",\"failed:\":\"false\",\"error\":\"false\"}"
        ));
    }

    #[test]
    fn a_failing_run_is_not_recorded() {
        let failed = terminal(
            |_| {
                Ok(Run {
                    output: String::new(),
                    exit_code: 1,
                })
            },
            "canon 0.0.0",
        );
        assert!(failed.is_err());
    }
}
