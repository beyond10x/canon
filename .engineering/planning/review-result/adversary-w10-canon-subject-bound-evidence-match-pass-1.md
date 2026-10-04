---
format: aep.planning-md/3
id: review-result:adversary-w10-canon-subject-bound-evidence-match-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w10 adversary, canon story:subject-bound-evidence-match, pass 1
relations:
- reviews: story:subject-bound-evidence-match
revision: 1
---
```
unit: canon/subject-bound-evidence-match, working tree on 3c38c75 plus the uncommitted phase 2
verdict: NEEDS-CHANGE
cases: executed 405→412, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths, all under …/canon-subject-bound-evidence-match/scratch/adversary1/
needs-coordinator: yes. The explanation unit and this one disagree on which records a subject-bound claim reads, and the disagreement shows up only once both are merged.
```

The suite is red. When a subject-bound match blocks an action or an outcome, the reason says `present: true` even though the match reads no record. The ELS acceptance itself holds: a test result about another artifact does not satisfy `tests.pass`, whether through a claim, an obligation, an action or an outcome.

**1. `git --no-pager diff --stat`**
Unchanged from what I was handed (the same 23 implementor files, 466+/144−). My only additions are two untracked test files: `crates/canon/tests/adversary_subject_reasons.rs` and `crates/canon/tests/adversary_subject_probes.rs`. I changed no implementation file and nothing in `ess/`.

**2. Cases added (each run alone, before the suite)**

| File | Case | Now |
|---|---|---|
| `adversary_subject_reasons.rs` | action blocked by a match bound to `change`; the only record is about `docs` at its current revision | red |
| same | the same, as an outcome requirement | red |
| same | `all` of two matches that differ only in subject; only the `docs` one is unmet | red |
| `adversary_subject_probes.rs` | ELS shape: claim, obligation, action and outcome over a record about `docs` | green |
| same | subject-bound matches under `not`, `any` and `all` | green |
| same | freshness (`expired`) and binding (`revision_mismatch`) exclusions combined with subject binding, through a claim reference | green |
| same | `read_ir` round trip; matches that differ only in subject are kept apart; a protocol with no subject writes no `subject` key; an IR edited to name an undeclared subject is refused | green |

Mutants on a scratch copy: when `excluded_for` ignores the subject, the exclusions case goes red. When `evidence_match` ignores the subject, the ELS case and the `not`/`any` case go red.

Red output (`reasons-red-2.log`, EXIT=101):
```
assertion `left == right` failed: a test result about the docs is not of a match bound to the change, yet the reason says a record of the kind applies:
  left: Object {"reasons": Array [Object {"evidence": String("test_result"), "present": Bool(true)}], "status": String("blocked")}
 right: Object {"reasons": Array [Object {"evidence": String("test_result"), "present": Bool(false)}], "status": String("blocked")}
test result: FAILED. 0 passed; 3 failed
```
After the first red run I changed the expected values so they compare against what the code gives with no evidence at all, not a hard-coded reason shape. That way a fix that adds `subject` to the reason still passes. The cases failed for the same reason before and after (`reasons-red.log`).

**3. Suite**
`cargo test --workspace --locked --no-fail-fast` ran 412 cases: 409 passed, 3 failed (the three above), EXIT=101.

**4. Findings**

| # | file:line | Finding | Verdict | Reaches it |
|---|---|---|---|---|
| F1 | `crates/canon/src/eval/actions.rs:132` and `outcomes.rs:224` | `present` is computed per kind: any record of that kind counts. With a subject-bound match, a record about another artifact therefore gives `present: true`, which goes against the docs at `actions.rs:14-16` and `outcomes.rs:21-23`. Fix: compute `present` over the records the match reads, i.e. kind plus subject. | NEEDS-CHANGE, introduced | Any action precondition or outcome requirement that uses a subject-bound match directly, as `evidence: {kind, subject}`. |
| F2 | `actions.rs:111` and `outcomes.rs:157` | Reasons are keyed by kind only, so two unmet matches that differ only in subject merge into one reason. If one has a record with another result and the other has no record, one of the two facts is lost. The reason shape may need a `subject` key. That is a contract decision. | NEEDS-CHANGE, introduced | `all`/`any` over matches of the same kind with different subjects (case 3). |
| F3 | `canon-w10-explanation` tree `crates/canon/src/explain/mod.rs:243` (`reached_kinds`) | After merge, a record about `docs` excluded as `revision_mismatch` is reported as `"status": "applied"` under a claim bound to `change`. The explanation unit says it counts records "as its `excluded_evidence` list counts them", but it still reaches by kind. Its "absent" check is kind-only too. | CONFIRMED, introduced (at merge) | Every protocol that uses a subject-bound match, once both units merge. Proven on a scratch merge: `explain-merge-red.log`, EXIT=101. |
| F4 | `crates/canon-docs/src/landing.rs:43-48` and `crates/canon-docs/src/pages.rs:1142-1147` | The docs generator renders evidence matches without `subject`, so a bound match is shown as unbound. | INFEASIBLE, introduced | Nothing today: both render only `fixtures/investigation/protocol.yaml`, which has no subject. |

**5. Attacked and could not break**
- The ELS acceptance (`tests.pass`) across the claim, obligation, action and outcome.
- Subject-bound matches under `not`, `any` and `all`.
- `discharged_when` with a subject: refused as `undeclared-artifact`, then `evidence-in-discharge`.
- A subject the case snapshot omits: the case-artifact check refuses it first.
- Subject plus result.
- Duplicate matches that differ only in subject.
- `read_ir` round trip, including `"subject": null` and an undeclared subject.
- IR bytes for protocols without a subject.
- Freshness and binding exclusions combined with subject binding.
- The docs' tables and their exclusion statement match what the code does.
- The phase-2 edits to `ess/` only renumber `read from:` lines.

**6. Paths written outside the worktree** (all under `~/.cache/ga-wave-2026-10-04-w10/canon-subject-bound-evidence-match/scratch/adversary1/`)
- `reasons-red.log`, `reasons-red-2.log`, `suite.log`, `explain-merge-red.log`, `explanation-eval-mod.patch`
- `merged/`: this tree plus the explanation unit's uncommitted changes, with the probe test
- `merged-target/` (75M)
- `mutant/`: its `claims.rs` is still mutated
- `mutant-target/` (73M)

The worktree session lease was acquired and released.

**7. Findings block**
```findings
- file: crates/canon/src/eval/actions.rs
  line: 132
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A subject-bound match blocked only by a record about another artifact reports {evidence, present: true} in actions (actions.rs:132) and outcomes (outcomes.rs:224), against the documented meaning that present is false when the required record is missing."
- file: crates/canon/src/eval/outcomes.rs
  line: 157
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "Unmet evidence matches are collected by kind only (outcomes.rs:157, actions.rs:111), so two matches that differ only in subject collapse into one reason and one subject's state is lost; the reason shape may need the subject."
- file: crates/canon/src/explain/mod.rs
  line: 243
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: "Once merged with the explanation unit, whose reached_kinds is kind-only, a record about another artifact excluded as revision_mismatch is explained as applied under a subject-bound claim (scratch merge, explain-merge-red.log)."
- file: crates/canon-docs/src/landing.rs
  line: 43
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "canon-docs renders evidence matches without subject (landing.rs:43-48, pages.rs:1142-1147), so a bound match would appear unbound; today nothing renders a protocol that has a subject."
```
