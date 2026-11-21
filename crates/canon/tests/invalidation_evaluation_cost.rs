//! What evaluation costs when every claim of a chain carries an invalidation rule
//! (story:invalidation-evaluation-cost).
//!
//! Adversary pass 2 of story:review-hardening-w7 timed a chain of claims with one invalidation
//! rule each: 0.37 s at 50 claims, 84 s at 400, about cubic. The chain here is that shape: claim
//! `c_i` tests `c_{i+1}`, the last claim reads the one evidence kind `k`, rule `r_i` invalidates
//! `c_i`, and one record of kind `k` is moved per rule (it records an earlier revision of the
//! rule's upstream artifact). One further record of kind `k` records no upstream revision, so every
//! claim is `true` and the explanation lists no claim.
//!
//! The record moved by `r_i` is invalidated for `c_0` to `c_i`, so the excluded evidence the
//! decision lists has `n (n + 1) / 2` entries: the output alone is quadratic, which is the bound
//! asserted. The two chain timing tests hold the evaluator to it; a diamond of the same rules holds
//! it to reusing a claim's value within a walk; the pinned decision holds every output byte for
//! byte while the cost changes.

use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, EvidenceRecord, Truth};

/// The timing tests run one at a time, so neither measures the other's work.
static TIMED: Mutex<()> = Mutex::new(());

fn timed_alone() -> std::sync::MutexGuard<'static, ()> {
    TIMED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn claim(at: usize) -> String {
    format!("c{at:05}")
}

fn upstream(at: usize) -> String {
    format!("u{at:05}")
}

/// The protocol, the case and the evidence of a chain of `claims` claims kept apart by one
/// invalidation rule each.
struct Chain {
    ir: Ir,
    case: model::Case,
    evidence: Vec<EvidenceRecord>,
}

fn invalidated_chain(claims: usize) -> Chain {
    invalidated(claims, |at| format!("{{claim: {}}}", claim(at + 1)))
}

/// A diamond of `claims` claims kept apart by one invalidation rule each: `c_i` tests `c_{i+1}` and
/// `c_{i+2}`, so a walk that does not reuse a claim's value reaches the last claims a Fibonacci
/// number of times.
fn invalidated_diamond(claims: usize) -> Chain {
    invalidated(claims, |at| {
        if at + 2 < claims {
            format!(
                "{{all: [{{claim: {}}}, {{claim: {}}}]}}",
                claim(at + 1),
                claim(at + 2)
            )
        } else {
            format!("{{claim: {}}}", claim(at + 1))
        }
    })
}

/// Claims `c_0` to `c_{n-1}`, each but the last true when `tests(i)` holds, the last reading kind
/// `k`, with rule `r_i` invalidating `c_i` and one record moved per rule.
fn invalidated(claims: usize, tests: impl Fn(usize) -> String) -> Chain {
    let mut source =
        String::from("format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts:\n  a: {}\n");
    for at in 0..claims {
        source.push_str(&format!("  {}: {{}}\n", upstream(at)));
    }
    source.push_str("evidence_kinds: {k: {}}\nclaims:\n");
    for at in 0..claims - 1 {
        source.push_str(&format!("  {}: {{true_when: {}}}\n", claim(at), tests(at)));
    }
    source.push_str(&format!(
        "  {}: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        claim(claims - 1)
    ));
    source.push_str("invalidation:\n");
    for at in 0..claims {
        source.push_str(&format!(
            "  r{at:05}: {{upstream: {}, invalidates: [{}]}}\n",
            upstream(at),
            claim(at)
        ));
    }
    let ir = ir::compile(&model::parse(&source).expect("parses")).expect("compiles");

    let mut artifacts = String::from("a: {revision: r1}");
    for at in 0..claims {
        artifacts.push_str(&format!(", {}: {{revision: new}}", upstream(at)));
    }
    let case = eval::read_case(&format!(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {{{artifacts}}}\n"
    ))
    .expect("case reads");

    let record = |id: &str, upstream: Option<String>| {
        let upstream = upstream
            .map(|artifact| format!("upstream_revisions: {{{artifact}: old}}\n"))
            .unwrap_or_default();
        eval::read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: k\nsubject: a\nsubject_revision: r1\n\
             {upstream}"
        ))
        .expect("evidence reads")
    };
    let mut evidence = vec![record("base", None)];
    for at in 0..claims {
        evidence.push(record(&format!("e{at:05}"), Some(upstream(at))));
    }
    Chain { ir, case, evidence }
}

impl Chain {
    fn evaluate(&self) -> model::Decision {
        eval::evaluate(&self.ir, &self.case, &self.evidence).expect("decides")
    }

    /// One evaluation's wall-clock time. The decision is dropped after the clock stops.
    fn time(&self) -> Duration {
        let started = Instant::now();
        let decision = self.evaluate();
        let took = started.elapsed();
        drop(decision);
        took
    }
}

/// Builds the chain of `claims` on a worker thread, which then evaluates it `runs` times and sends
/// each time. Every time must arrive within `deadline` of the one before (of the build, for the
/// first), so a red run ends at the deadline; the worker is left to the end of the process.
fn timed_on_worker(claims: usize, runs: usize, deadline: Duration) -> Result<Duration, String> {
    let (built, built_rx) = mpsc::channel();
    let (sent, times) = mpsc::channel();
    thread::Builder::new()
        .name(format!("chain-of-{claims}"))
        .spawn(move || {
            let chain = invalidated_chain(claims);
            if built.send(()).is_err() {
                return;
            }
            for _ in 0..runs {
                if sent.send(chain.time()).is_err() {
                    return;
                }
            }
        })
        .expect("a worker thread starts");
    built_rx.recv().expect("the chain builds");
    let mut fastest = Duration::MAX;
    for run in 1..=runs {
        match times.recv_timeout(deadline) {
            Ok(took) => fastest = fastest.min(took),
            Err(_) => {
                return Err(format!(
                    "run {run} of {runs} on a chain of {claims} claims took more than {deadline:?}"
                ));
            }
        }
    }
    Ok(fastest)
}

/// The name is the adversary's and is kept so it can be traced; the bound is quadratic. Each size
/// is timed as the minimum of 3 runs; the long chain runs on a worker thread with a deadline of 64
/// times the short time plus 2 s, so a red run ends in seconds.
#[test]
fn claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain() {
    let _alone = timed_alone();
    let short_chain = invalidated_chain(50);
    let short = (0..3)
        .map(|_| short_chain.time())
        .min()
        .expect("three runs");
    let deadline = short * 64 + Duration::from_secs(2);
    let long = match timed_on_worker(400, 3, deadline) {
        Ok(long) => long,
        Err(why) => panic!(
            "a chain eight times as long did not finish: {why}, against {short:?} for 50 claims: \
             more than quadratic growth"
        ),
    };
    eprintln!("invalidated chain: 50 claims {short:?}, 400 claims {long:?}");
    assert!(
        long <= short * 64,
        "a chain eight times as long took {long:?}, against {short:?}: more than quadratic growth"
    );
}

/// The depth bound admits a chain of about 4000 claims (4096 levels, `eval/depth.rs`). A 4000-claim
/// chain evaluates in under 10 s in a release build; a debug build runs the same chain against 120
/// s. The chain is evaluated once, on a worker thread with the budget as its deadline.
#[test]
fn a_chain_at_the_depth_bound_evaluates_within_budget() {
    let _alone = timed_alone();
    let budget = if cfg!(debug_assertions) {
        Duration::from_secs(120)
    } else {
        Duration::from_secs(10)
    };
    let claims = 4000;
    let (sent, received) = mpsc::channel();
    let (built, built_rx) = mpsc::channel();
    thread::Builder::new()
        .name("chain-at-the-bound".to_owned())
        .spawn(move || {
            let chain = invalidated_chain(claims);
            if built.send(()).is_err() {
                return;
            }
            let started = Instant::now();
            let decision = chain.evaluate();
            let took = started.elapsed();
            let decided = decision.claims.iter().count();
            let all_true = decision
                .claims
                .iter()
                .all(|(_, entry)| entry.value == Truth::True);
            let listed: usize = decision
                .claims
                .iter()
                .map(|(_, entry)| entry.excluded_evidence.len())
                .sum();
            // Freed before the result is sent, so while this test holds `TIMED`: freeing the
            // 8 million listed exclusions after it let go slowed the other timing test's 400-claim
            // chain about threefold.
            drop(decision);
            drop(chain);
            let _ = sent.send((took, decided, all_true, listed));
        })
        .expect("a worker thread starts");
    built_rx.recv().expect("the chain builds");
    let (took, decided, all_true, listed) = received.recv_timeout(budget).unwrap_or_else(|_| {
        panic!("a chain of {claims} claims took more than {budget:?} to evaluate")
    });
    eprintln!("invalidated chain: {claims} claims {took:?}");
    assert_eq!(decided, claims);
    assert!(
        all_true,
        "the record that recorded no upstream revision decides every claim"
    );
    assert_eq!(
        listed,
        claims * (claims + 1) / 2,
        "each moved record is listed under each claim built on its claim"
    );
    assert!(
        took < budget,
        "a chain of {claims} claims took {took:?}, over {budget:?}"
    );
}

/// A diamond of 500 claims, each with a rule of its own, so that every walk has a context of its
/// own and reuses values only through its per-walk memo. Reused, the work is quadratic and takes well
/// under a second in a debug build; not reused, the last claims are reached a Fibonacci number of
/// times (about 10^104 for 500 claims) and the walk never ends. The budget is 30 s in a debug build
/// and 5 s in a release build, on a worker thread with the budget as its deadline. Every claim is
/// `true` and lists exactly the records moved by its own rule and the rules of the claims below it.
#[test]
fn a_diamond_of_claims_kept_apart_by_invalidation_rules_evaluates_within_budget() {
    let _alone = timed_alone();
    let budget = if cfg!(debug_assertions) {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(5)
    };
    let claims = 500;
    let (sent, received) = mpsc::channel();
    let (built, built_rx) = mpsc::channel();
    thread::Builder::new()
        .name("invalidated-diamond".to_owned())
        .spawn(move || {
            let diamond = invalidated_diamond(claims);
            if built.send(()).is_err() {
                return;
            }
            let started = Instant::now();
            let decision = diamond.evaluate();
            let took = started.elapsed();
            let mut decided = 0;
            let mut wrong = Vec::new();
            for (at, (id, entry)) in decision.claims.iter().enumerate() {
                decided += 1;
                let listed: Vec<String> = entry
                    .excluded_evidence
                    .iter()
                    .map(|exclusion| {
                        assert_eq!(exclusion.reason, model::ExclusionReason::Invalidated);
                        exclusion.evidence.as_str().to_owned()
                    })
                    .collect();
                let expected: Vec<String> =
                    (at..claims).map(|moved| format!("e{moved:05}")).collect();
                if id.as_str() != claim(at) || entry.value != Truth::True || listed != expected {
                    wrong.push(format!("{id}: {:?}, {} listed", entry.value, listed.len()));
                }
            }
            drop(decision);
            drop(diamond);
            let _ = sent.send((took, decided, wrong));
        })
        .expect("a worker thread starts");
    built_rx.recv().expect("the diamond builds");
    let (took, decided, wrong) = received.recv_timeout(budget).unwrap_or_else(|_| {
        panic!("a diamond of {claims} claims took more than {budget:?} to evaluate")
    });
    eprintln!("invalidated diamond: {claims} claims {took:?}");
    assert_eq!(decided, claims);
    assert!(wrong.is_empty(), "claims decided otherwise: {wrong:?}");
    assert!(
        took < budget,
        "a diamond of {claims} claims took {took:?}, over {budget:?}"
    );
}

/// The decision for a 5-claim invalidated chain, byte for byte: what the evaluator gave before its
/// cost changed.
#[test]
fn the_decision_for_a_five_claim_invalidated_chain_is_pinned() {
    let chain = invalidated_chain(5);
    assert_eq!(
        eval::render(&chain.evaluate()),
        PINNED.replace("{canon}", env!("CARGO_PKG_VERSION"))
    );
}

/// The rendered decision, with `{canon}` for the crate version the explanation records.
const PINNED: &str = r#"{
  "case": "C",
  "claims": {
    "c00000": {
      "excluded_evidence": [
        {
          "evidence": "e00000",
          "reason": "invalidated"
        },
        {
          "evidence": "e00001",
          "reason": "invalidated"
        },
        {
          "evidence": "e00002",
          "reason": "invalidated"
        },
        {
          "evidence": "e00003",
          "reason": "invalidated"
        },
        {
          "evidence": "e00004",
          "reason": "invalidated"
        }
      ],
      "value": "true"
    },
    "c00001": {
      "excluded_evidence": [
        {
          "evidence": "e00001",
          "reason": "invalidated"
        },
        {
          "evidence": "e00002",
          "reason": "invalidated"
        },
        {
          "evidence": "e00003",
          "reason": "invalidated"
        },
        {
          "evidence": "e00004",
          "reason": "invalidated"
        }
      ],
      "value": "true"
    },
    "c00002": {
      "excluded_evidence": [
        {
          "evidence": "e00002",
          "reason": "invalidated"
        },
        {
          "evidence": "e00003",
          "reason": "invalidated"
        },
        {
          "evidence": "e00004",
          "reason": "invalidated"
        }
      ],
      "value": "true"
    },
    "c00003": {
      "excluded_evidence": [
        {
          "evidence": "e00003",
          "reason": "invalidated"
        },
        {
          "evidence": "e00004",
          "reason": "invalidated"
        }
      ],
      "value": "true"
    },
    "c00004": {
      "excluded_evidence": [
        {
          "evidence": "e00004",
          "reason": "invalidated"
        }
      ],
      "value": "true"
    }
  },
  "explanation": {
    "computed_from": {
      "case": {
        "artifacts": {
          "a": {
            "revision": "r1"
          },
          "u00000": {
            "revision": "new"
          },
          "u00001": {
            "revision": "new"
          },
          "u00002": {
            "revision": "new"
          },
          "u00003": {
            "revision": "new"
          },
          "u00004": {
            "revision": "new"
          }
        },
        "format": "canon-case/1",
        "id": "C",
        "protocol": "p"
      },
      "evidence": [
        "base",
        "e00000",
        "e00001",
        "e00002",
        "e00003",
        "e00004"
      ],
      "protocol": "p",
      "protocol_revision": 1,
      "semantics": {
        "canon": "{canon}",
        "format": "canon-decision/1"
      }
    }
  },
  "format": "canon-decision/1",
  "protocol": "p",
  "protocol_revision": 1
}
"#;
