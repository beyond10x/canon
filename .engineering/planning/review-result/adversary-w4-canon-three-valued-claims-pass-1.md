---
format: aep.planning-md/3
id: review-result:adversary-w4-canon-three-valued-claims-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w4 adversary, canon story:three-valued-claims, pass 1
relations:
- reviews: story:three-valued-claims
revision: 1
---
unit: canon/three-valued-claims, working tree on top of 4c33a4d (phase-2 changes not committed)
verdict: NEEDS-CHANGE
cases: executed 190→200, red 7
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 14 paths, all under the brief's scratch directory (part 6)
needs-coordinator: none

**1. Changes in the worktree**

`git --no-pager diff --stat` lists 9 tracked files with 1026 insertions and 150 deletions. All of them are the implementor's phase 2. I added only 2 new, untracked test files:
- `crates/canon/tests/adversary_tvc_eval.rs`
- `crates/canon-cli/tests/adversary_tvc_cli.rs`

I edited no implementation file.

**2. Cases added (each run alone first; logs `scratch/adversary-tvc-eval-red.log` and `scratch/adversary-tvc-cli-red.log`)**

| Case | Now | Red output, verbatim |
|---|---|---|
| `a_claim_defined_as_another_claim_is_unknown_when_that_claim_is_unknown` | red | `left: False right: Unknown` |
| `with_no_evidence_a_negated_claim_reference_is_not_true` | red | `` `release.blocked: {not: {claim: tests.pass}}` is TRUE with no evidence at all `` |
| `a_claim_reference_agrees_with_the_predicate_it_names_on_every_evidence_set` | red | `evidence [] left: False right: Unknown` |
| `every_ir_canon_compile_prints_reads_back` | red | `next line U+0085: … not the canonical text`; `delete U+007F` / `C1 control U+0080` / `noncharacter U+FFFE: … not well-formed JSON` |
| `a_document_reads_the_same_from_text_and_from_a_value` | red | text gives `Ok(… Revision("1"))`, value gives `Err(malformed-input … invalid type: integer`1`, expected a string)` |
| `what_canon_compile_prints_canon_evaluate_accepts` (CLI) | red | `del: … error[malformed-input]: not canon-ir/1: not well-formed JSON`, exit 1 |
| `a_refutation_in_the_evidence_directory_is_never_silently_dropped` (CLI) | red | `exit Some(0)` with `"value": "true"` |
| `every_identifier_the_refusal_list_names_is_checked` | green | catches mutants M1–M5 |
| `an_evidence_record_with_an_unknown_field_is_refused` | green | catches mutant M6 |
| `a_missing_ir_or_case_file_exits_2` (CLI) | green | catches mutant M7 |

**3. Suite, run after the cases existed**

Command: `cargo test --workspace --locked --no-fail-fast` with the brief's `CARGO_TARGET_DIR`. Result: exit 101, `summed: passed 193, failed 7`. The 7 failures are exactly the red cases above; every other test passes. `-- --list` shows all 10 new cases in this tree.

The 190 before-count is 200 minus my 10 cases. I took it from `--list`; I did not make a separate run with my files deselected.

**4. Findings (cover the working tree above)**

| # | file:line | Verdict | What was measured | What reaches it |
|---|---|---|---|---|
| F1 | `crates/canon/src/eval/claims.rs:61` | NEEDS-CHANGE, blocker | A plain claim reference `{claim: c}` (`is` left at its default) only checks whether `c` is TRUE. So when `c` is UNKNOWN, a claim defined as `{claim: c}` comes out FALSE, and `not {claim: c}` comes out TRUE with an empty evidence set. That breaks design § 8 ("tests never run -> UNKNOWN", "TRUE: applicable evidence establishes") and the story's own "UNKNOWN and FALSE are never collapsed". | `{claim: c}` is the one protocol/1 syntax for building one claim on another (`predicate.rs:5`). No fixture in the tree does this yet, and the decision format is fixed here as "extended, never replaced". Suggested fix: without `is`, or with `is: true`, return `c`'s own value; `is: false` returns `not(c)`. |
| F2 | `crates/canon/src/eval/read.rs:56,72` | CONFIRMED, warning | `read_ir` parses the IR as YAML. The IR writer (`ir/json.rs:106`) prints DEL, C1 controls and U+FFFE as raw characters, which YAML rejects outright, and YAML turns NEL into a space. Result: `canon evaluate` refuses IR that `canon compile` printed, exit 1. | Any protocol description holding one of these characters, e.g. the source escape `"\x7f"`. Rare. Suggested fix: read the IR with a JSON parser. |
| F3 | `crates/canon/src/eval/read.rs:42` | CONFIRMED, warning | `read_case`/`read_evidence` (text, used by the CLI) accept numbers as identifiers (`revision: 1` becomes `"1"`). `case_from_value`/`evidence_from_value` (YAML values, used by `conform run`) refuse them as `malformed-input`. The same document passes one path and fails the other. | The wave-3 adversary file wrote `subject_revision: 1` and `{explanation: 1}`, and the coordinator's patch had to change them to `r1`. |
| F4 | `crates/canon-cli/src/main.rs:118` | NEEDS-CHANGE, warning | Only `*.yaml` files in the evidence directory are read; anything else is dropped without a word. A refuting record saved as `falsification-2.json` turns the `conflicting` step's UNKNOWN into TRUE, with exit 0. | Design § 29 names inputs `case.json`, and `read.rs` says documents may be written as JSON. Suggested fix: refuse any other file in the evidence directory, or read `.json`/`.yml` as well. |
| F5 | `crates/canon/src/eval/mod.rs:155,157,213-215` | CONFIRMED, note | Mutants M1–M5 (each deletes one identifier check: case protocol, case artifact, evidence id, kind, subject) keep the unit's own suite green. With the subject check removed, a record with an invalid subject is evaluated. | Covered now by `every_identifier_the_refusal_list_names_is_checked`. |
| F6 | `crates/canon/src/model/evidence.rs:14` | CONFIRMED, note | Mutant M6 (drops `deny_unknown_fields`) keeps the unit's suite green, so a misspelt `reslut: refuted` would be read as a record with no result. | Covered now by `an_evidence_record_with_an_unknown_field_is_refused`. |
| F7 | `crates/canon-cli/src/main.rs:88` | CONFIRMED, note | Mutant M7 (a missing IR or case file exits 1 instead of 2) keeps the unit's suite green. | Covered now by `a_missing_ir_or_case_file_exits_2`. |

Mutant M8 (conflicting evidence evaluates FALSE) is caught by the unit's own tests; I ran it as a check that the mutant setup works. The run log is `scratch/adversary-mutants.log`, plus `scratch/adversary-mutants-rerun.log` for M6 and M7 after I synced my newer test files into the mutant copy.

**5. Attacked and not broken**

- all/any/not follow strong Kleene logic, including the empty cases.
- Evidence matching by kind and result follows the story's rule, and conflicting records give UNKNOWN.
- Duplicate evidence ids are refused.
- The decision does not depend on evidence order.
- The library does no IO and uses no hash-ordered maps.
- `read_ir` rejects non-canonical IR. It accepts hand-edited IR only when the edit is itself canonical, which describes a different protocol and is not a defect.
- U+2028 and U+2029 survive the IR round trip.
- `eval/json.rs` escapes strings the same way the IR writer does.
- The conform evaluate step compares decision bytes and refusal codes correctly.
- `ess_model_matches` reports an entity declared in `ess/` but missing from `ENTITIES`, and covers both documents and the `NON_NEGATIVE` list.

I did not acquire a worktree session lease.

**6. Paths written outside the worktree**

- Under `~/.cache/ga-wave-2026-10-04-w4/canon-three-valued-claims/scratch/`: `adversary-tvc-eval-red.log`, `adversary-tvc-cli-red.log`, `adversary-suite.log`, `adversary-mutants.log`, `adversary-mutants-rerun.log`, `run-mutants.sh` and `mutant-M0…M8-*.out` (9 files). That is 14 paths in total.
- Deleted when done: the mutant copy `scratch/mutant-tree/` and the mutant build directory `~/.cache/b10x-target/canon-w4-mutants`.
- Test scratch also went under the brief's build directory, in `CARGO_TARGET_TMPDIR/adversary-tvc-*`.

**7. Findings block**

```findings
- file: crates/canon/src/eval/claims.rs
  line: 61
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A claim test without `is` is two-valued, so a claim defined as {claim: c} is FALSE and not {claim: c} is TRUE when c is UNKNOWN with no evidence, collapsing UNKNOWN into FALSE against design section 8 and the story outcome."
- file: crates/canon/src/eval/read.rs
  line: 56
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "read_ir parses canon-ir/1 as YAML, so IR that canon compile prints with DEL, C1 controls or U+FFFE in a description is refused as not well-formed JSON, and NEL is folded and refused as non-canonical."
- file: crates/canon/src/eval/read.rs
  line: 42
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The text readers used by canon evaluate accept a numeric identifier or revision as a string while the value readers used by canon conform run refuse it as malformed-input, so one document gets two answers."
- file: crates/canon-cli/src/main.rs
  line: 118
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "canon evaluate silently ignores evidence files not named *.yaml, so a refuting record saved as .json turns an UNKNOWN claim into TRUE with exit 0."
- file: crates/canon/src/eval/mod.rs
  line: 215
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Deleting any of the case protocol, case artifact, evidence id, kind or subject identifier checks leaves the unit's suite green; every_identifier_the_refusal_list_names_is_checked now catches all five."
- file: crates/canon/src/model/evidence.rs
  line: 14
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Dropping deny_unknown_fields on EvidenceRecord leaves the unit's suite green, so a misspelt result key would be read as a record with no result; an_evidence_record_with_an_unknown_field_is_refused now catches it."
- file: crates/canon-cli/src/main.rs
  line: 88
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Making a missing --ir or --case file exit 1 instead of 2 leaves the unit's suite green; a_missing_ir_or_case_file_exits_2 now catches it."
```
