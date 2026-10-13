---
format: aep.planning-md/3
id: review-result:adversary-evidence-order-pass-1
kind: review-result
status: active
title: Wave evidence-order-predicate adversary, canon story:evidence-order-predicate, pass 1
relations:
- reviews: story:evidence-order-predicate
revision: 1
---
The adversary pass found one real defect, and I committed two failing tests for it: `canon check` misses an `authority-bypass` when an order form sits under a `not` or a claim test `is: false`. Everything else I attacked held.

```
unit: story:evidence-order-predicate, tree canon-unit-evidence-order, branch unit/evidence-order-predicate; findings cover 34c6035 (my commit 6a65990 on top)
verdict: NEEDS-CHANGE
cases: executed 596→598, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path
needs-coordinator: whether to keep the "positive position only" rule for order forms in authority-bypass (as the module docs now say) or count an order form's kinds in every position; the red cases assume the latter
```

**1. Diff of my commit (34c6035..HEAD):**
```
 crates/canon/tests/adversary_evidence_order.rs | 77 ++++++++++++++++++++++++++
 1 file changed, 77 insertions(+)
```
It touches one file, and it is a test file.

**2. Cases added** (`crates/canon/tests/adversary_evidence_order.rs`), both red now. The protocol has a governed kind `review`: one action produces it under a capability, and `self_review` produces it with none.

- **`an_order_form_under_not_still_reads_governed_evidence_for_a_bypass`**: the outcome `not_rejected` requires `{not: {first: {kind: review, result: rejected}}}`. With no review it is `unknown`. One review produced without authority makes it legitimate, and taking that review away blocks it. So it is a bypass, but nothing is reported. The positive control `approved` (`{first: {kind: review, result: approved}}`) is reported. Output from running it alone:
  ```
  a review produced without authority makes `not_rejected` legitimate, and taking it away blocks it, so it is a bypass:
  authority-bypass: outcome `approved` is legitimate without authority in state {evidence `review` result `approved` earlier}
  checked: protocol `p` revision 1: 216 states, 0 properties, 1 finding
  ```
- **`an_order_form_behind_a_claim_test_is_false_still_reads_governed_evidence_for_a_bypass`**: the same case through `{claim: rejected_first, is: false}`. Output from running it alone:
  ```
  `released` is legitimate on one review produced without authority and blocked without it, so it is a bypass:
  checked: protocol `p` revision 1: 60 states, 0 properties, 0 findings
  ```

**3. Suite run, after the cases existed:** `cargo test --no-fail-fast -p b10x-canon -p canon-cli`, with ess 0.53.0 first on PATH. Result: passed 596, failed 2 (the two above), `error: 1 target failed`, EXIT=101. The before count of 596 is the same run with my target taken out by arithmetic; there was no separate run without it.

**4. Findings**

| # | Severity | What | Evidence | Verdict / origin |
|---|---|---|---|---|
| 1 | warning | `positive_reads` counts an order form's kinds only in a positive position. That rule was carried over from evidence matches, but it does not hold for order forms. With no record, `first` and `before` are `unknown`, and so is their negation, so a record helps `not first` and `is: false` hold too. Fix to name: count the kinds an order form reads whatever its position, and update the doc line at `check/mod.rs:107`. | `crates/canon/src/check/space.rs:1008`; the two tests above. What reaches it: any protocol whose outcome requirement negates an order form over a governed kind, e.g. "the first review was not a rejection". | NEEDS-CHANGE / introduced |
| 2 | note | CANON-ORDER-001 has no step where `test_result` and `diff` share their earliest instant. If `<` became `<=` in `before`, the scenario would still pass; the unit test in `order.rs` ("equal") is the only thing that catches it. Changing `first` ties to FALSE is caught by step 6. Dropping excluded records from the history is caught by step 1 for `revision_mismatch`, and by the `eval/mod.rs` unit test for `expired` and `invalidated`. | `conformance/scenarios/evidence-order.yaml` (found by reading, not by running a mutant) | CONFIRMED / introduced |
| 3 | note | In `ess/domains/protocol.yaml`, the read-from comments for `FirstResult` and `EvidenceOrder` cite only story lines, not the Rust structs at `predicate.rs:92-125`. One range is loose: `:97-110` runs into the Validation section. | `ess/domains/protocol.yaml` | CONFIRMED / introduced |

**5. Attacks that found nothing**
- **Order values:** ties, missing `observed_at`, subject filtering, and `before` with K2 and no K1 (FALSE, even when K2 has no instant) all match the story's Values section (`order.rs:69-108`).
- **Excluded records:** records excluded for `revision_mismatch`, `expired` or `invalidated` are read by the order forms and not by evidence matches, and `excluded_evidence` is unaffected (`eval/mod.rs:470`, `claims.rs:277`).
- **Determinism:** I ran a probe over 3 evidence sets × 120 orderings each, including ties, unplaced records, r1 records and expired ones. The rendered decision and explanation were byte-identical every time. The probe was green and I deleted it, uncommitted.
- **`canon check` soundness of values:** I traced every combination of history, unplaced records and rank against evidence matches, invalidation vectors and multiple dimensions, and found no unreachable combination. Check over the fixture reaches `fixed` and `merge` (1296 states, only `unproduced-evidence` findings).
- **Byte-for-byte without order forms:** the golden `fixtures/investigation/check/*/report.txt` and `refusal.txt` files are unchanged since base, and `canon_check.rs` passes on them.
- **State-count bound:** the refusal names the history and order dimensions; `rankings` is only built inside the bound.
- **`canon diff`:** a changed, added or removed order form is BREAKING. Unchanged forms fall back to implication, which is sound.
- **IR:** `read_ir` recompiles and compares bytes, so a non-canonical IR or one with extra keys is refused. Form order and field order are both canonical.
- **Validation:** undeclared kind or subject, and the same kind in `before`, are refused in the order the docs give.
- **Action reasons and explanation shape:** they match the story.
- **ESS:** `ess specify validate --strict-requires`, `compile` and `conform synthesize` pass under 0.53.0 (found at `~/.cache/ess-0.53.0/`), with no `UNMAPPED:`. `ess_model_matches` and `ess_gate` pass in the suite.

**6. Paths written outside the worktree**
- `~/.cache/canon-evidence-order/adversary-suite.log` (the suite log; kept).
- My commit-message file in the same directory, already deleted.

**Process notes**
- I waited 1800 s for disk: it was 18G at the start and 25–26G when the build began.
- Load was 19–23, under 30, so `CARGO_BUILD_JOBS` was not set.
- Peak `du -sh target` was 6.3G (4.8G before).
- I made one commit, `6a65990 test: an order form under a flip still reads governed evidence for authority-bypass`, authored by b10x-bot. Nothing was pushed.

```findings
[
  {
    "file": "crates/canon/src/check/space.rs",
    "line": 1008,
    "category": "judgement",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "authority-bypass counts an order form's kinds only in a positive position, but a record makes a negated order form true as well, so a bypass through {not: {first: ...}} or a claim test is false is never reported"
  },
  {
    "file": "conformance/scenarios/evidence-order.yaml",
    "category": "mutant",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "no step puts test_result and diff at one earliest instant, so changing before's strict < to <= survives CANON-ORDER-001 and is caught only by the order.rs unit test"
  },
  {
    "file": "ess/domains/protocol.yaml",
    "category": "judgement",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "FirstResult and EvidenceOrder carry read-from comments citing only story lines, not the Rust structs they must match, and one cited story range runs into the Validation section"
  }
]
```
