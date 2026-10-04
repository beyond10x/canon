//! Independent review of wave 2026-10-04-w7: properties of the merged evaluator over generated
//! protocols, cases, evidence sets and authority decisions, with a fixed seed.
//!
//! For each generated input the evaluator must:
//!
//! 1. give the same bytes twice, and whatever order the evidence and the authority entries are in;
//! 2. decide exactly what it decides when the records the exclusion stages leave out were never
//!    given (revision binding and freshness, recomputed here from the documented rules);
//! 3. never write `admissible` for an action unless every capability it requires is granted, and
//!    write `approval-required` / a denial `blocked` exactly as the module docs of `eval/actions.rs`
//!    say;
//! 4. give every blocked outcome at least one reason;
//! 5. accept a termination through a legitimate outcome with the same decision, and refuse one
//!    through a blocked outcome as `illegitimate-termination`.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, EvidenceRecord, Json};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

const KINDS: [&str; 3] = ["k0", "k1", "k2"];
const CAPABILITIES: [&str; 3] = ["cap.x", "cap.y", "cap.z"];
const AT: &str = "2026-10-04T12:00:00Z";
const OBSERVED: [(&str, bool); 3] = [
    ("2026-10-04T10:59:59Z", true),  // older than k0's 1h at AT
    ("2026-10-04T11:00:00Z", false), // exactly 1h old: applies
    ("2026-10-04T11:59:00Z", false),
];

/// A predicate as YAML flow text. `claims` are the claims it may test (all declared earlier, so
/// the protocol is acyclic); `evidence` whether it may match evidence.
fn predicate(rng: &mut Rng, depth: usize, claims: &[String], evidence: bool) -> String {
    let leaf_only = depth == 0;
    match if leaf_only {
        3 + rng.below(2)
    } else {
        rng.below(5)
    } {
        0 => {
            let n = rng.below(3);
            let members: Vec<String> = (0..n)
                .map(|_| predicate(rng, depth - 1, claims, evidence))
                .collect();
            format!("{{all: [{}]}}", members.join(", "))
        }
        1 => {
            let n = rng.below(3);
            let members: Vec<String> = (0..n)
                .map(|_| predicate(rng, depth - 1, claims, evidence))
                .collect();
            format!("{{any: [{}]}}", members.join(", "))
        }
        2 => format!("{{not: {}}}", predicate(rng, depth - 1, claims, evidence)),
        _ if !claims.is_empty() && (!evidence || rng.chance(50)) => {
            let claim = &claims[rng.below(claims.len())];
            let is = ["true", "false", "unknown"][rng.below(3)];
            format!("{{claim: {claim}, is: {is}}}")
        }
        _ if evidence => {
            let kind = KINDS[rng.below(KINDS.len())];
            match rng.below(3) {
                0 => format!("{{evidence: {{kind: {kind}}}}}"),
                1 => format!("{{evidence: {{kind: {kind}, result: pass}}}}"),
                _ => format!("{{evidence: {{kind: {kind}, result: fail}}}}"),
            }
        }
        // No claim to test and no evidence allowed: a constant.
        _ => "{all: []}".to_owned(),
    }
}

struct Generated {
    ir: ir::Ir,
    outcomes: Vec<String>,
    case: String,
    evidence: Vec<String>,
    /// Per record: whether the exclusion stages leave it applying.
    applies: Vec<bool>,
    authority: Vec<(String, &'static str)>,
}

fn generate(rng: &mut Rng) -> Generated {
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a0: {}, a1: {}}\n\
         evidence_kinds: {k0: {max_age: 1h}, k1: {}, k2: {}}\nclaims:\n",
    );
    let mut claims: Vec<String> = Vec::new();
    for index in 0..4 {
        let id = format!("c{index}");
        let p = predicate(rng, 3, &claims, true);
        source.push_str(&format!("  {id}: {{true_when: {p}}}\n"));
        claims.push(id);
    }
    source.push_str("obligations:\n");
    for index in 0..2 {
        let p = predicate(rng, 2, &claims, false);
        source.push_str(&format!("  o{index}: {{discharged_when: {p}}}\n"));
    }
    source.push_str("actions:\n");
    for index in 0..3 {
        let p = predicate(rng, 2, &claims, true);
        let requires: Vec<String> = CAPABILITIES
            .iter()
            .filter(|_| rng.chance(50))
            .map(|cap| format!("{{capability: {cap}}}"))
            .collect();
        source.push_str(&format!(
            "  act{index}: {{precondition: {p}, requires: [{}]}}\n",
            requires.join(", ")
        ));
    }
    source.push_str("outcomes:\n");
    let mut outcomes = Vec::new();
    for index in 0..3 {
        let p = predicate(rng, 2, &claims, true);
        source.push_str(&format!("  out{index}: {{requires: {p}}}\n"));
        outcomes.push(format!("out{index}"));
    }
    let protocol = model::parse(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    let ir = ir::compile(&protocol).unwrap_or_else(|e| panic!("{e:?}\n{source}"));

    let case = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a0: {revision: r2}, a1: {revision: r1}}\n"
        .to_owned();
    let mut evidence = Vec::new();
    let mut applies = Vec::new();
    for index in 0..rng.below(7) {
        let kind = KINDS[rng.below(KINDS.len())];
        let (subject, current) = [("a0", "r2"), ("a1", "r1")][rng.below(2)];
        let revision = if rng.chance(70) { current } else { "r0" };
        let result = ["", "result: pass\n", "result: fail\n"][rng.below(3)];
        let (observed, old) = if rng.chance(60) {
            let (at, old) = OBSERVED[rng.below(OBSERVED.len())];
            (format!("observed_at: '{at}'\n"), old)
        } else {
            (String::new(), false)
        };
        evidence.push(format!(
            "format: canon-evidence/1\nid: e{index}\nkind: {kind}\n{result}subject: {subject}\n\
             subject_revision: {revision}\n{observed}"
        ));
        applies.push(revision == current && !(kind == "k0" && old));
    }
    let authority = CAPABILITIES
        .iter()
        .filter_map(|cap| match rng.below(3) {
            0 => None,
            1 => Some(((*cap).to_owned(), "granted")),
            _ => Some(((*cap).to_owned(), "denied")),
        })
        .collect();
    Generated {
        ir,
        outcomes,
        case,
        evidence,
        applies,
        authority,
    }
}

fn authority_text(entries: &[(String, &str)]) -> String {
    let lines: Vec<String> = entries
        .iter()
        .map(|(cap, decision)| format!("- {{capability: {cap}, decision: {decision}}}\n"))
        .collect();
    if lines.is_empty() {
        "[]\n".to_owned()
    } else {
        lines.concat()
    }
}

fn records(texts: &[String]) -> Vec<EvidenceRecord> {
    texts
        .iter()
        .map(|text| eval::read_evidence(text).expect("record reads"))
        .collect()
}

fn decide(
    g: &Generated,
    case: &str,
    evidence: &[String],
    authority: &str,
    at: Option<&str>,
) -> Result<model::Decision, eval::Refusal> {
    let case = eval::read_case(case).expect("case reads");
    let supplied = Supplied {
        authority: Some(authority),
        at,
        decisions: None,
    };
    eval::evaluate_with(&g.ir, &case, &records(evidence), supplied)
}

fn section(decision: &model::Decision, name: &str) -> Option<Json> {
    match name {
        "obligations" => decision.obligations.clone(),
        "actions" => decision.actions.clone(),
        "outcomes" => decision.outcomes.clone(),
        _ => unreachable!(),
    }
}

#[test]
fn generated_evaluations_keep_every_documented_invariant() {
    let mut rng = Rng(0x00c0_ffee_2026_1004);
    // How often each branch was reached, so a green run is not a vacuous one.
    let mut seen: std::collections::BTreeMap<String, usize> = Default::default();
    let mut count = |key: String| *seen.entry(key).or_default() += 1;
    for sample in 0..1500 {
        let g = generate(&mut rng);
        let authority = authority_text(&g.authority);
        let full = decide(&g, &g.case, &g.evidence, &authority, Some(AT))
            .unwrap_or_else(|r| panic!("sample {sample}: refused: {r}"));
        let bytes = eval::render(&full);

        // 1. Same bytes twice, and in any evidence and authority order.
        let again = decide(&g, &g.case, &g.evidence, &authority, Some(AT)).expect("decides");
        assert_eq!(
            eval::render(&again),
            bytes,
            "sample {sample}: not repeatable"
        );
        let mut reversed = g.evidence.clone();
        reversed.reverse();
        let mut rotated = g.evidence.clone();
        if !rotated.is_empty() {
            let by = rng.below(rotated.len());
            rotated.rotate_left(by);
        }
        let mut authority_reversed = g.authority.clone();
        authority_reversed.reverse();
        for (evidence, authority) in [
            (&reversed, authority_text(&g.authority)),
            (&rotated, authority_text(&authority_reversed)),
        ] {
            let other = decide(&g, &g.case, evidence, &authority, Some(AT)).expect("decides");
            assert_eq!(
                eval::render(&other),
                bytes,
                "sample {sample}: order changed bytes"
            );
        }

        // 2. Exclusion is removal.
        let kept: Vec<String> = g
            .evidence
            .iter()
            .zip(&g.applies)
            .filter(|(_, applies)| **applies)
            .map(|(text, _)| text.clone())
            .collect();
        let removed = decide(&g, &g.case, &kept, &authority, None).expect("decides");
        for (id, entry) in full.claims.iter() {
            let without = removed
                .claims
                .iter()
                .find(|(other, _)| *other == id)
                .expect("same claims")
                .1;
            assert_eq!(
                entry.value,
                without.value,
                "sample {sample}: claim {} differs from the removed-records evaluation",
                id.as_str()
            );
            assert!(without.excluded_evidence.is_empty());
        }
        for name in ["obligations", "actions", "outcomes"] {
            assert_eq!(
                section(&full, name),
                section(&removed, name),
                "sample {sample}: section {name} differs from the removed-records evaluation"
            );
        }
        let excluded: usize = g.applies.iter().filter(|applies| !**applies).count();
        let listed: std::collections::BTreeSet<String> = full
            .claims
            .iter()
            .flat_map(|(_, entry)| entry.excluded_evidence.iter())
            .map(|exclusion| exclusion.evidence.as_str().to_owned())
            .collect();
        assert!(
            listed.len() <= excluded,
            "sample {sample}: more listed than excluded"
        );
        if excluded > 0 {
            count("records excluded".to_owned());
        }
        for (_, entry) in full.claims.iter() {
            count(format!("claim {}", entry.value));
        }

        // 3. Authority.
        let actions = full.actions.as_ref().expect("actions");
        for (id, action) in &g.ir.actions {
            let entry = &actions[id.as_str()];
            let status = entry["status"].as_str().expect("status");
            count(format!(
                "action {status}{}",
                if any_denied_reason(entry) {
                    " (denied)"
                } else {
                    ""
                }
            ));
            let decision_of =
                |cap: &str| g.authority.iter().find(|(c, _)| c == cap).map(|(_, d)| *d);
            let all_granted = action
                .requires
                .iter()
                .all(|cap| decision_of(cap.as_str()) == Some("granted"));
            let any_denied = action
                .requires
                .iter()
                .any(|cap| decision_of(cap.as_str()) == Some("denied"));
            if status == "admissible" {
                assert!(
                    all_granted,
                    "sample {sample}: {} admissible without grants",
                    id.as_str()
                );
                assert!(entry.get("reasons").is_none());
            }
            if status == "approval-required" {
                assert!(!any_denied && !all_granted, "sample {sample}: {entry}");
            }
            if status != "admissible" {
                let reasons = entry["reasons"].as_array().expect("reasons");
                assert!(!reasons.is_empty(), "sample {sample}: {entry}");
            }
            if !all_granted {
                assert_ne!(status, "admissible", "sample {sample}");
            }
        }

        // 4 and 5. Outcomes and termination.
        let outcomes = full.outcomes.as_ref().expect("outcomes");
        for outcome in &g.outcomes {
            let entry = &outcomes[outcome.as_str()];
            let terminated = format!("{}termination: {outcome}\n", g.case);
            let result = decide(&g, &terminated, &g.evidence, &authority, Some(AT));
            count(format!("outcome {}", entry["status"]));
            match entry["status"].as_str() {
                Some("legitimate") => {
                    let mut decision = result.unwrap_or_else(|r| panic!("sample {sample}: {r}"));
                    // The explanation records the case it read, termination included
                    // (`computed_from.case`); everything else is the same decision.
                    if let Some(case) = decision
                        .explanation
                        .as_mut()
                        .and_then(|explanation| explanation.pointer_mut("/computed_from/case"))
                        .and_then(Json::as_object_mut)
                    {
                        case.remove("termination");
                    }
                    assert_eq!(eval::render(&decision), bytes, "sample {sample}");
                }
                Some("blocked") => {
                    let reasons = entry["reasons"].as_array().expect("reasons");
                    assert!(!reasons.is_empty(), "sample {sample}: {entry}");
                    let refusal = result.expect_err("a blocked termination is refused");
                    assert_eq!(
                        refusal.code(),
                        "illegitimate-termination",
                        "sample {sample}"
                    );
                    assert!(refusal.to_string().contains(&format!("`{outcome}`")));
                }
                other => panic!("sample {sample}: status {other:?}"),
            }
        }
    }
    let expected = [
        "records excluded",
        "claim true",
        "claim false",
        "claim unknown",
        "action admissible",
        "action approval-required",
        "action blocked",
        "action blocked (denied)",
        "outcome \"legitimate\"",
        "outcome \"blocked\"",
    ];
    for key in expected {
        let reached = seen.get(key).copied().unwrap_or(0);
        assert!(
            reached >= 20,
            "branch {key:?} reached {reached} times: {seen:?}"
        );
    }
    println!("branches reached: {seen:?}");
}

/// Whether an action entry's reasons include a denied capability.
fn any_denied_reason(entry: &Json) -> bool {
    entry["reasons"]
        .as_array()
        .is_some_and(|reasons| reasons.iter().any(|reason| reason["decision"] == "denied"))
}

/// YAML aliases expand when the authority document is read. A document that names one small list
/// 10^9 times through nine levels of aliases must be refused (or read) quickly, not expanded.
#[test]
fn an_alias_expansion_authority_document_is_not_expanded() {
    let mut text = String::from("a0: &a0 [{capability: x, decision: granted}]\n");
    for level in 1..10 {
        let prev = level - 1;
        text.push_str(&format!(
            "a{level}: &a{level} [*a{prev}, *a{prev}, *a{prev}, *a{prev}, *a{prev}, *a{prev}, *a{prev}, *a{prev}, *a{prev}, *a{prev}]\n"
        ));
    }
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = eval::evaluate_with(
            &ir::compile(
                &model::parse("format: protocol/1\nprotocol: {id: p, revision: 1}\n")
                    .expect("parses"),
            )
            .expect("compiles"),
            &eval::read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {}\n")
                .expect("case reads"),
            &[],
            Supplied {
                authority: Some(&text),
                ..Supplied::default()
            },
        )
        .map(|_| ())
        .map_err(|refusal| refusal.code());
        let _ = send.send(outcome);
    });
    let outcome = receive
        .recv_timeout(std::time::Duration::from_secs(20))
        .expect("the authority document is read within 20 s");
    assert_eq!(outcome, Err("malformed-input"));
}
