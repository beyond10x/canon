---
format: aep.planning-md/3
id: review-result:adversary-w10-canon-explanation-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w10 adversary, canon story:explanation, pass 1
relations:
- reviews: story:explanation
revision: 1
---
```
unit: canon/explanation, working tree of canon-w10-explanation on 3245a25 plus uncommitted phase 2
verdict: NEEDS-CHANGE
cases: executed 402→410, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths (part 6)
needs-coordinator: whether "down to the evidence records" covers evidence-match reasons and true-claim reasons (findings 1–2)
```

**1. `git --no-pager diff --stat`**: I did not modify any tracked file. All 15 modified paths are the implementor's phase 2. My only change is the new untracked file `crates/canon/tests/adversary_explain_trace.rs`, which is a test file. No implementation file changed.

**2. Cases** (`crates/canon/tests/adversary_explain_trace.rs`). I ran them alone first: 4 passed, 4 failed. Log: `scratch/adversary1-red.log`.

| Case | Asserts | Now |
|---|---|---|
| `a_blocked_outcome_on_an_excluded_record_traces_to_that_record` :93 | an outcome with `requires: {evidence: {kind: k}}`, whose only record is at a superseded revision: its chain names `e1` as excluded for `revision_mismatch` | RED. `because` is `[{"evidence": "k", "present": false}]`, and `e1` appears only in `computed_from` |
| `a_blocked_action_on_an_expired_record_traces_to_that_record` :118 | the same for an action precondition and an `expired` record | RED, same shape |
| `an_open_obligation_on_a_true_claim_traces_to_its_evidence` :154 | obligation `discharged_when: {claim: c, is: false}` with `c` true: its chain reaches `e1` (applied) | RED. `because` is `[{"claim": "c", "value": "true"}]`, `claims` has no `c`, and the chain ends there |
| `an_obligation_and_an_outcome_on_one_predicate_give_the_same_reasons` :181 | `all: [a, b]` with a true and b unknown gives the same `because` for the obligation and the outcome | RED. Obligation `[a:true, b:unknown]`, outcome `[b:unknown]` |
| `every_claim_reason_carries_the_decisions_own_value` :208 | every `{claim, value}` in the explanation matches `decision.claims` (false case) | green; catches mutant M1 |
| `a_discharged_obligation_and_a_legitimate_outcome_are_not_explained` :262 | only open obligations, blocked outcomes and non-true claims are written | green; catches M2 |
| `the_case_snapshot_records_its_termination` :291 | `computed_from.case` includes `revision` and `termination` | green; catches M3 |
| `every_input_order_renders_the_same_bytes` :323 | 24 evidence orders × authority/decisions orders, non-ASCII ids, expired and mismatched records: byte-identical output, code-point order | green |

**Mutants**, applied to a scratch copy (`explain/mod.rs`):
- M1: claim reason values always `unknown` (:235).
- M2: obligation `open` filter dropped (:328).
- M3: `termination` not recorded (:158).

With all three applied, all 402 existing tests passed, along with `canon-docs generate --check` (exit 0) and `conform run` (8 passed). Only my cases failed: the 4 red ones plus the 3 mutant catchers at :223, :284 and :306.

**3. Suite** (run after the cases existed): `cargo test --workspace --locked --no-fail-fast` gave 406 passed, 4 failed, `EXIT=101`. The failures are exactly the 4 red cases. `--list` shows all 8 tests in this tree. Clippy on the file exits 0; I ran rustfmt on it and it is clean now.

**4. Findings**

| # | file:line | Verdict | Origin | What was measured / what reaches it |
|---|---|---|---|---|
| 1 | `crates/canon/src/explain/mod.rs:348` | NEEDS-CHANGE | introduced | `not_with_status` copies the outcome/action reasons as they are. A `{"evidence": <kind>, "present": ...}` reason is never traced to the records of that kind, so an excluded record and its reason (`revision_mismatch` or `expired`) are missing from the explanation of a blocked outcome or action. This breaks the story outcome and the status row at `website/docs/status/where-this-stands.md:27` ("down to each evidence record that applied or was excluded and why"). **Reaches it:** an evidence match in `requires` or `precondition` is valid, documented protocol syntax (`eval/mod.rs` Sections; `adversary2_outcomes_polarity.rs:137`). |
| 2 | `crates/canon/src/explain/mod.rs:28` | NEEDS-CHANGE | introduced | A `{claim: c, value: "true"}` reason in an open obligation, blocked outcome or blocked action does not continue under `claims`, so the chain never reaches the evidence that made `c` true. This is how the module is documented to behave, but it contradicts the story outcome. **Reaches it:** the refute-obligation shape `{claim: c, is: false}` is declared in `eval/obligations.rs`'s own tests (`m.refuted`), and `{not: {claim: c}}` requirements are tested in `adversary2_outcomes_polarity.rs:64`. |
| 3 | `crates/canon/src/explain/mod.rs:332` | CONFIRMED | introduced | The obligation `because` lists every claim tested, including ones that do not decide it. The scenario comment (`conformance/scenarios/explanation.yaml:15-17`) promises "the reasons that decide it" for outcomes, actions and obligations alike. Contract drift between the scenario and the module docs. |
| 4 | `crates/canon/src/explain/mod.rs:235` | CONFIRMED | introduced | The suite cannot fail on M1, M2 or M3: every existing explanation case has only `unknown` claims, no discharged obligation, and no termination. My 3 green cases close that gap. |
| 5 | `conformance/scenarios/explanation.yaml:14` | CONFIRMED | introduced | The scenario's description of `claims` leaves out the `{"kind", "status": "absent"}` entries the code writes. They do appear, in kind order after the records, in `adversary_skel_cli.rs` NO_EVIDENCE_DECISION. |

Note, not a finding: `crates/canon/src/eval/mod.rs:493` is a sixth changed test. It now compares only the listed sections, but it pins claim order, so nothing it guarded is lost.

**5. Attacked and could not break**
- Determinism under evidence, authority and decision order, including non-ASCII and astral-plane ids: list order matches canonical key order.
- Duplicates are refused upstream (`evidence.rs`, `authority.rs`, `decisions.rs`).
- `at` is only accepted in the exact `YYYY-MM-DDTHH:MM:SSZ` form, so recording it as given is canonical.
- `computed_from` records only supplied inputs plus the crate version.
- Size grows as claims × records with one level of nesting, so it cannot blow up.
- Claim values in the explanation agree with the decision.
- `expired` and `revision_mismatch` land under the right claim, including transitive claims.
- The 5 coordinator test edits assert `computed_from` before setting it aside, so no guarantee was weakened.
- I did not acquire a worktree lease: the brief forbids worktree commands.

**6. Paths written outside the worktree** (all under `~/.cache/ga-wave-2026-10-04-w10/canon-explanation/scratch/`)
- `adversary1-red.log`
- `adversary1-suite.log`
- `mutant-suite.log`
- `mutant/` (3.6M tree copy with the 3 mutants)
- `mutant-target/` (601M; safe to delete)
- `~/.cache/b10x-target/canon-w10-explanation` (the assigned build dir, reused)

```findings
- file: crates/canon/src/explain/mod.rs
  line: 348
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A blocked outcome or action whose reason is an evidence match is not traced to the records of that kind, so an excluded record and its revision_mismatch or expired reason never appear in its explanation."
- file: crates/canon/src/explain/mod.rs
  line: 28
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A claim reason with value true in an open obligation, blocked outcome or blocked action has no entry under claims, so its chain stops before the evidence that made the claim true."
- file: crates/canon/src/explain/mod.rs
  line: 332
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Obligation because lists every tested claim, including ones that do not decide it, while the scenario comment and the outcome reasons give only the deciding ones."
- file: crates/canon/src/explain/mod.rs
  line: 235
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Mutants writing every claim reason as unknown, explaining discharged obligations, or dropping the case termination survive all 402 existing tests, the docs check and conform run."
- file: conformance/scenarios/explanation.yaml
  line: 14
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The scenario's description of claims omits the kind-absent entries the implementation writes after the evidence records."
```
