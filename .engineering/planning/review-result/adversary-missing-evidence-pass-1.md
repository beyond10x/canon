---
format: aep.planning-md/3
id: review-result:adversary-missing-evidence-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:missing-evidence (wave 2026-11-20-w4): green, 1 warning and 2 notes, introduced'
relations:
- reviews: story:missing-evidence
revision: 1
---
Adversary pass 1 on story:missing-evidence, wave 2026-11-20-w4, `aep:adversary` on Opus, on `impl/missing-evidence` at `617f904`. The agent was stopped once by a model limit and resumed. The report below is as the agent returned it.

unit: story:missing-evidence, findings cover impl/missing-evidence at 617f904 (my cases committed as 65f9b70)
verdict: green
cases: executed 998→1006, red 0
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w4/missing-evidence/adversary-p1/` only
needs-coordinator: no

I resumed as you asked. Before going on I checked `git status`, `git log`, the scratch directory and the suite log. Nothing had been lost.

**1. What I changed (`git --no-pager diff --stat 617f904..HEAD`)**
```
 .../tests/adversary_missing_evidence_cli.rs        |  97 ++++++++
 crates/canon/tests/adversary_missing_evidence.rs   | 245 +++++++++++++++++++++
 2 files changed, 342 insertions(+)
```
Both are test files. I edited no implementation file and none of the unit's own tests or fixtures. I added no fixture. The commit was made through `b10x-gates bot`, has a body and no attribution lines.

**2. The cases I added (all green; none went red)**
- `crates/canon/tests/adversary_missing_evidence.rs`, 7 cases:
  - A record invalidated for one claim is not applicable, even though another claim is `true` on it.
  - The same holds for a record rejected as `not_independent` by one claim.
  - A record of a kind no claim reads is applicable.
  - A stale record (revision mismatch, or expired) next to a current record of the same kind does not list the kind, in either input order.
  - Kinds come out in code-point order, with upper case, `_` and a kind listed twice; declaring actions and kinds in another order changes no byte.
  - Only an action granted at the case revision is a key. A `require_approval` entry, a grant taken at another case revision, or a denial gives no section at all.
  - An admissible action with `case_open` is a key.
- `crates/canon-cli/tests/adversary_missing_evidence_cli.rs`, 1 case: `canon evaluate` writes the section for an invalidated record, places it after `format`, and renaming the evidence files or mixing YAML and JSON changes no byte.
- The first runs of each file on its own, verbatim:
  - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`

**3. The full suite, run after the cases existed**
- `cargo test --workspace --locked --no-fail-fast`: exit 0. The 141 `test result:` lines add up to 1006 passed, 0 failed, 3 ignored. That is the unit's reported 998 plus my 8.
- `canon conform run --requirements conformance/requirements.yaml`: exit 0, with these summary lines:
  - `conform: 23 passed, 0 failed, 0 unreadable`
  - `requirements: 24 implemented, 24 covered, 0 planned`
- `cargo clippy -p b10x-canon -p canon-cli --tests -- -D warnings`: exit 0. `rustfmt --check` on my two files is clean.

**4. Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| `conformance/scenarios/missing-evidence.yaml:20` | acceptance | warning | CONFIRMED | introduced | CANON-FRONTIER-001 itself, and any implementation checked only against it |
| `crates/canon/tests/ess_model_matches.rs:1330` | mutant | note | CONFIRMED | introduced | the next edit to `authored_map_keys` or `ir_ty` |
| `crates/canon/src/model/decision.rs:51` | judgement | note | CONFIRMED | introduced | `website/docs/reference/documents.md:79`, the reference page for document readers |

- **Scenario gaps.** Three requirement sentences have no step that pins them:
  - **Approval-required.** The only approval-required action in the fixture, `publish_finding`, has no `may_produce`.
  - **Kind order.** Every action may produce exactly one kind, so nothing pins the order of kinds.
  - **Absence.** No step can pin that the section is absent, because the conformance runner fails any listed section the decision does not carry.

  Mutants that list approval-required actions, or drop the "absent when none qualifies" rule, pass the scenario. The unit tests in `missing.rs` catch both. Acceptance point 1's "the blocked `publish_finding` is not a key" proves nothing on its own, since that action has no `may_produce`. `request_peer_review` is what actually pins the blocked rule. Suggested fix: add a step with an approval-required action that may produce two kinds.
- **Parity harness.** The change still catches real differences. I mutated scratch copies and ran the built test binary against them, with `CARGO_MANIFEST_DIR` pointing at each copy. It caught all 7 mutants:
  - the map key changed, on the `ess/` side and on the Rust side;
  - the optional map made required, on each side;
  - the value type changed;
  - `Optional<List>` in place of the map;
  - `Option` added around the value.

  The control run passed. One mutant (key `string`) was refused by `ess` before any comparison, so it tests nothing. There is still no committed mutation control for an `Optional<Map>` key, which is why this is a note.
- **Reference page.** It says a record is applicable "by the rule of `eval::missing::applicable`". That function is crate-private, and the page does not state the rule; `evaluation.md` does.

**5. What I attacked and did not break**
- **Applicability:** 6 grounds hold. These are the invalidated-for-one-claim, not-independent-for-one-claim, unread-kind and stale-beside-current cases above, plus `case_outcome`, which the binding stage treats like any other record. `evaluate_with` passes `left` and every claim's entry, as the story requires.
- **Bytes against the base:** I used the older binary `canon-6f44325`, as the brief allowed, on 133 conformance evaluate steps.
  - 67 are byte-identical. Another 30 are byte-identical once I strip the `missing_evidence` block from the head's output. 12 are refused identically by both.
  - Not compared: 10 `authority-binding` steps (the old binary refuses `case_revision`) and 14 steps whose fixtures it cannot compile without `--import` or `supplied_by`.
- **Statuses:** a grant at the current revision, a lapsed grant, `require_approval`, a denial and `case_open` all behave as claimed.
- **Order:** evidence file names and formats, declaration order, and upper case against `_` against lower case all give the same, correct order. `--ir` input is recompiled, so `may_produce` is always sorted.
- **Re-pins:** the three re-pinned expectations (`adversary_skel_cli.rs`, `explanation.rs`, `explanation.yaml`) only gained lines; none lost an assertion.
- **Weak tests:** none of the unit's new tests is one that could not fail.

**6. Paths written outside the worktree**
All under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w4/missing-evidence/adversary-p1/`:
- logs: `build.log`, `case-run-1.log`, `case-run-2.log`, `clippy.log`, `suite.log`, `conform.log`, `oracle.log`
- scripts: `oracle_compare.py`, `harness_probe.sh`, `suite.sh`
- commit message: `msg-1.txt`
- probe trees: `oracle-run/`, `harness/` (copies of `ess/` and `crates/canon/src/model/`)

```findings
[{"file": "conformance/scenarios/missing-evidence.yaml", "line": 20, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "CANON-FRONTIER-001 has no step with an approval-required action that may produce evidence and none with an action of two kinds, so a mutant listing approval-required actions or leaving kinds unsorted passes the scenario."}, {"file": "crates/canon/tests/ess_model_matches.rs", "line": 1330, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The harness change has no committed mutation control for an Optional<Map> key or optionality mismatch; a scratch probe shows the harness catches all 7 such mutants today."}, {"file": "crates/canon/src/model/decision.rs", "line": 51, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The generated documents reference points readers to the crate-private eval::missing::applicable instead of stating the applicability rule."}]
```
