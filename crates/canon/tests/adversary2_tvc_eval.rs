//! Adversary pass 2 for story:three-valued-claims, against the evaluator library: nesting depth,
//! claim-reference cycles and memoization.

use std::sync::mpsc;
use std::time::Duration;

use b10x_canon::eval::{self, Refusal};
use b10x_canon::ir;
use b10x_canon::model::{self, ClaimId, ClaimTest, Decision, Predicate, Truth};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

const RECORD: &str =
    "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\nsubject_revision: r1\n";

fn header() -> String {
    "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\nevidence_kinds: {k: {}}\nclaims:\n".to_owned()
}

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("protocol parses")).expect("protocol compiles")
}

fn decide(ir: &ir::Ir) -> Result<Decision, Refusal> {
    let case = eval::read_case(CASE)?;
    let evidence = vec![eval::read_evidence(RECORD)?];
    eval::evaluate(ir, &case, &evidence)
}

/// `eval::read` docs: `read_ir` "accepts exactly the bytes `canon compile` prints". A predicate
/// nested 123 `not` deep parses and compiles (the YAML reader's own recursion limit is not reached),
/// so its IR is compiler output and must read back. `serde_json`'s default recursion limit (128)
/// refuses it as "not well-formed JSON", which it is.
#[test]
fn the_ir_of_a_deeply_nested_predicate_the_compiler_accepts_reads_back() {
    let depth = 123;
    let source = format!(
        "{}  c: {{true_when: {}{{evidence: {{kind: k, result: pass}}}}{}}}\n",
        header(),
        "{not: ".repeat(depth),
        "}".repeat(depth)
    );
    let compiled = compiled(&source);
    let text = compiled.canonical_json();
    serde_json::from_str::<serde_json::Value>(&text)
        .map(|_| ())
        .unwrap_or_else(|error| eprintln!("serde_json alone: {error}"));
    assert_eq!(
        eval::read_ir(&text),
        Ok(compiled),
        "the IR canon compile printed for a {depth}-deep `not` is refused"
    );
}

/// `eval::read` docs: an IR "that describes a protocol the validator rejects (an undeclared
/// reference, a claim cycle), is refused before evaluation". The IR text of a self-referencing
/// claim is refused naming the cycle.
#[test]
fn ir_text_describing_a_claim_cycle_is_refused_as_a_cycle() {
    let mut cyclic = compiled(&format!(
        "{}  c: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        header()
    ));
    cyclic
        .claims
        .get_mut(&ClaimId::new("c"))
        .expect("c")
        .true_when = Predicate::Claim(ClaimTest {
        claim: ClaimId::new("c"),
        is: Truth::True,
    });
    let refusal = eval::read_ir(&cyclic.canonical_json()).expect_err("refused");
    assert_eq!(refusal.code(), "malformed-input");
    assert!(refusal.to_string().contains("claim-cycle"), "{refusal}");
}

const CHILD: &str = "ADVERSARY2_TVC_CYCLE_CHILD";

/// `eval::evaluate` is public and takes an `ir::Ir`, whose fields are public, so a library caller
/// can hand it an IR with a claim cycle (here a claim that tests itself). The module docs promise
/// "a decision ... or a Refusal"; `claims::value_of` assumes the cycle away ("A compiled protocol
/// has no claim cycle") and recurses until the stack overflows, which aborts the whole process.
/// The evaluation runs in a child process so the abort is observed rather than suffered.
#[test]
fn an_ir_with_a_claim_cycle_is_refused_not_a_stack_overflow() {
    if std::env::var_os(CHILD).is_some() {
        let mut cyclic = compiled(&format!(
            "{}  c: {{true_when: {{evidence: {{kind: k}}}}}}\n",
            header()
        ));
        cyclic
            .claims
            .get_mut(&ClaimId::new("c"))
            .expect("c")
            .true_when = Predicate::Claim(ClaimTest {
            claim: ClaimId::new("c"),
            is: Truth::True,
        });
        let outcome = decide(&cyclic);
        println!("child outcome: {outcome:?}");
        return;
    }
    let child = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "an_ir_with_a_claim_cycle_is_refused_not_a_stack_overflow",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, "1")
        .output()
        .expect("child runs");
    let stderr = String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success(),
        "evaluating a cyclic IR crashed the process ({}): {}",
        child.status,
        stderr.lines().rev().take(3).collect::<Vec<_>>().join(" | ")
    );
}

/// Claim values are memoized (`claims::value_of`). Forty claims, each testing the next twice (once
/// as is, once negated, so the compiler keeps both members), evaluate at once with memoization and
/// in 2^40 steps without it. Nothing else in the suite would notice the memo going away.
#[test]
fn a_claim_tested_twice_at_every_level_is_evaluated_once() {
    let levels = 40;
    let mut source = header();
    for i in 0..levels {
        source.push_str(&format!(
            "  c{i:02}: {{true_when: {{any: [{{claim: c{next:02}}}, {{claim: c{next:02}, is: false}}]}}}}\n",
            next = i + 1
        ));
    }
    source.push_str(&format!(
        "  c{levels:02}: {{true_when: {{evidence: {{kind: k, result: pass}}}}}}\n"
    ));
    let ir = compiled(&source);
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = send.send(decide(&ir));
    });
    let decision = receive
        .recv_timeout(Duration::from_secs(20))
        .expect("the evaluation finishes within 20 s")
        .expect("decides");
    let first = decision
        .claims
        .iter()
        .next()
        .map(|(id, entry)| (id.as_str().to_owned(), entry.value));
    assert_eq!(first, Some(("c00".to_owned(), Truth::True)));
}
