---
format: aep.planning-md/3
id: review-result:adversary-supplied-evidence-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:supplied-evidence (wave 2026-11-20-w3): red, 4 findings, all introduced'
relations:
- reviews: story:supplied-evidence
revision: 1
---
Adversary pass 1 on story:supplied-evidence, wave 2026-11-20-w3, `aep:adversary` on Opus. The
report below is as the agent returned it.

unit: story:supplied-evidence, branch impl/supplied-evidence at 282edbc (+ adversary commit 2c212ec)
verdict: red
cases: executed 976→979, red 1
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/supplied-evidence/adversary-p1/ (logs, probe/, msg-1.txt, wait.sh)
needs-coordinator: yes. The story and the code disagree on whether a supplied kind is governed (finding 1); the coordinator picks which one changes.

**1. Diff against the base** (`git diff --stat 282edbc HEAD`). Only a test file:
```
 crates/canon/tests/adversary_supplied_evidence.rs | 105 ++++++++++++++++++++++
```
Committed as the bot: `2c212ec test: adversary pass 1 on supplied evidence`.

**2. Cases added** in `crates/canon/tests/adversary_supplied_evidence.rs`
- `a_supplied_kind_is_not_governed_even_when_an_authority_action_produces_it`: **red**. Kind `s` declares `supplied_by: q`, and `approve` (requires capability `c`) may also produce it. Without the field, check reports 0 findings. With it, check reports `authority-bypass`. The story (§ Outcome 3, story line 55) says "a supplied kind is not governed in this protocol". First run of the case on its own (`cargo test -p b10x-canon --test adversary_supplied_evidence`):
```
assertion `left == right` failed: story § Outcome 3: a supplied kind is not governed in this protocol
  left: "authority-bypass: outcome `done` is legitimate without authority in state {evidence `s`}\nchecked: protocol `p` revision 1: 6 states, 0 properties, 1 finding\n"
 right: "checked: protocol `p` revision 1: 6 states, 0 properties, 0 findings\n"
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
- `a_supplied_record_about_an_undeclared_subject_is_refused`: green probe. Still refused as `undeclared-artifact`.
- `an_import_supplied_by_the_root_composes_into_a_self_supplied_protocol`: green probe. The composed document is refused as `self-supplied-evidence`.

**3. Suite**, run after the cases existed: `export PATH=$HOME/.cache/ess/toolchains/0.77.0:$PATH; cargo test --workspace --no-fail-fast`, exit 101. The first run without `--no-fail-fast` stopped at my red binary.
```
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
error: 1 target failed:
    `-p b10x-canon --test adversary_supplied_evidence`
```
Totals over 138 summaries: 978 passed, 1 failed, 3 ignored.

**4. Findings** (they cover 282edbc)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/check/mod.rs:750 | acceptance | warning | INFEASIBLE | introduced | A protocol where a kind is both supplied and produced by an action that requires a capability. No such protocol found: the SDC receivers produce `roadmap_approval` only through `roadmap.read`, which needs no capability. Red case above. |
| conformance/requirements.yaml:146 | acceptance | note | CONFIRMED | introduced | `canon conform run`. CANON-SUPPLY-001 still passes with the check rule or the self-id refusal removed: scenarios have only compile and evaluate steps. Only the evaluator exception is pinned. CANON-IMPORT-001 has the same gap. |
| crates/canon-cli/src/lib.rs:128 | contract-drift | note | CONFIRMED | introduced | `canon check --help` and the generated `website/docs/reference/cli.md` list what counts as produced (`may_produce`, `case_open`) and leave out `supplied_by`. |
| crates/canon/src/eval/mod.rs:93 | contract-drift | note | CONFIRMED | introduced | The evaluator docs still say every upstream artifact must be declared. The implementor already flagged this in needs-coordinator.patch; it is not their file. |

On finding 1: the code treats a supplied kind as produced without authority but leaves it governed. The unit's own docs (`check/mod.rs`, `ess/domains/protocol.yaml`) say "produced without authority" and never "not governed", and the implementor's report doesn't mention the gap. The literal fix (take supplied kinds out of `governed`) would hide a real bypass when a capability-free local action also produces the kind. The precise fix is to take a supplied kind out of `governed` only when no capability-free action produces it. Otherwise the story text should change instead.

**Judgement on claim 4 (`canon diff`)**
- EXPANSION when a supplier is added and BREAKING when it is removed are right. They match the classifier's own precedents: a value added to or removed from a case input, and an evidence kind added or removed. Record sets once refused are now read, and no earlier decision changes.
- NO SEMANTIC CHANGE for a changed supplier is true for evaluation. But it departs from the closest precedent: changing the protocol in `case_open`, a name the runner acts on and nothing resolves, is BREAKING (`diff/mod.rs:44`, `:1063`). The coordinator should pick one consistently.
- `canon check` reading the field should not change the category. Adding a `may_produce` entry is EXPANSION and can also make `unproduced-evidence` go away and `authority-bypass` appear; the classifier classes by evaluation, not by check findings.

**5. Attacked and not broken**
- Claim 1, compile: 68 fixtures without `supplied_by` compile byte-identical to the oracle, and 24 refusals have identical text. Check output matches on 74 fixtures; the 5 that differ are slicing effects already on the base, and `supplied()` is empty without the field.
- Read-back through `--ir`: 5 values tried; `""`, `"q q"`, the protocol's own id and `null` are refused, the valid value round-trips.
- Option B evaluation, 7 probes: an invalidation rule on a declared artifact next to an undeclared entry; order forms and `since` (they don't read `upstream_revisions`); a non-identifier name; the same name twice; an undeclared subject; a producer; 10,000 undeclared entries (0.7 s on a debug build).
- Imports, 4 probes: self-supply composed from an import; self-supply inside an import; a root refine that drops the supplier; a root refine that names itself.
- Check: 3 protocols; a supplied `case_outcome` read by a `children` form; a protocol whose only kind is supplied; probe states.
- `canon generate` then `canon conform run` on 3 supplied protocols: 4 scenarios, all pass.
- `supplied_by` values in `protocol/1`: 10 tried. `7` and `true` are accepted as strings, the same as other id fields on the base.
- Mutants of the unit's own tests: 6 checked by reading. Each is caught.

**6. Paths written outside the worktree**
- $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/supplied-evidence/adversary-p1/red-alone.log, suite.log, suite-nff.log, msg-1.txt, wait.sh
- $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w3/supplied-evidence/adversary-p1/probe/ (scripts, protocols, IRs, evidence, generated scenarios, logs)

```findings
[{"file": "crates/canon/src/check/mod.rs", "line": 750, "category": "acceptance", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A kind that declares supplied_by stays governed when an authority-requiring action also produces it, so canon check reports authority-bypass where story Outcome 3 says a supplied kind is not governed."}, {"file": "conformance/requirements.yaml", "line": 146, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "CANON-SUPPLY-001 states the self-id refusal and the check rule as MUSTs, yet its scenario still passes with either removed."}, {"file": "crates/canon-cli/src/lib.rs", "line": 128, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The canon check help and generated cli.md list may_produce and case_open as what produces a kind, and leave out supplied_by."}, {"file": "crates/canon/src/eval/mod.rs", "line": 93, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The evaluator docs still require every upstream artifact to be declared, without the supplied-kind exception; this is already in the implementor's needs-coordinator.patch."}]
```
