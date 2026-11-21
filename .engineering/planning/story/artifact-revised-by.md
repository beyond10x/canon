---
format: aep.planning-md/3
id: story:artifact-revised-by
kind: story
status: active
title: Declare artifacts revised by another protocol with revised_by, and count a state that waits on that revision as waiting in canon simulate
tags:
- from-request
refs:
- provider: sdc
  reference: request-revised-by
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:simulate-revise-step
scope:
- confidence: inferred
  path: crates/canon-cli/tests/revised_by.rs
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: inferred
  path: crates/canon/src/ir/imports.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/simulate/mod.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/revised-by/
- confidence: inferred
  path: fixtures/investigation/simulate/revised-by/
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:23:18Z", actor: "human:timo", revision: 15}
- {from: "proposed", to: "active", at: "2026-11-21T23:23:18Z", actor: "human:timo", revision: 16}
---
## Outcome

An artifact of `protocol/1` can name the protocol whose cases revise it. `canon simulate` then offers a revise step for that artifact in every state, and a state that only that step can open counts as waiting and is no finding. Canon revises nothing and opens nothing. The step says that a case of the named protocol may move the artifact to a new revision between two evaluations.

1. **`revised_by` on an artifact.**
   - Optional, one protocol id. Example: `roadmap: {revised_by: sdc.direction}`.
   - `canon validate` refuses the protocol's own id as `self-revised-artifact`, naming the artifact. Own actions revise through `revises` (story:judgement-marking), so the step for them is already the revise step of story:simulate-revise-step. A self-name would hide the dead end that a revising action which no state admits leaves (class a3).
   - The id is checked as a protocol identifier (`invalid-identifier`, as `supplied_by` at `crates/canon/src/validate/mod.rs:797-800`). Any other id is valid and nothing resolves it (test at `:1632-1657`: parent, child, sibling and an undeclared id).
   - An artifact that an action also lists in `revises` is valid. It gets both steps.
2. **`canon-ir/1`.**
   - An artifact writes `"revised_by"` only when set. The IR of every existing protocol stays byte-identical. The IR read-back reads it, refuses a non-string and refuses the own id (as `eval/read.rs:609-620` for `supplied_by`).
   - The IR needs an artifact type with a field. Today it is `Described` (`crates/canon/src/ir/mod.rs:130`, `:275`, `:623`).
   - A protocol that declares it gets a new IR digest, so an importer pins the new one.
   - Imports: two parts that declare one artifact with different `revised_by` are refused as `conflicting-imports` (`ir/imports.rs:447`, which compares IR entries). An imported artifact revised by the importing root composes into `self-revised-artifact`.
3. **No evaluation meaning.** `canon evaluate`, `canon check` and `canon generate` give the same bytes with or without it, including the state count of `canon check`.
4. **`canon diff`.** Adding, removing and replacing `revised_by` is `NO SEMANTIC CHANGE` (see Decision needed 2). The precedent for `supplied_by` is at `crates/canon/src/diff/mod.rs:24` (introduced, `EXPANSION`), `:25` (removed, `BREAKING`), `:26` (changed, `BREAKING`), `supplier` at `:913-935`, and the exhaustive destructure of `EvidenceKind` at `:861-866`. The `artifact` section (`:743-755`) destructures `Described` and must change to the new type. Artifact added or removed stays `BREAKING` (`:18`).
5. **External revise step of `canon simulate`.** It is offered in every state for each artifact with `revised_by`. It builds on option A of story:simulate-revise-step and uses its state change for a revision (§ Outcome 2) unchanged.
   - Successor: the artifact is at a new revision. Current records about it leave the state. Records of a dimension that a `children` form reads become `earlier`. Records of a dimension with upstream `a` become `observed before upstream a moved`. No record is added.
   - It is written `` protocol `sdc.direction` revises `roadmap` ``, as `` protocol `s` supplies evidence `k` `` is written for a supplied record.
   - Order: after the supplied steps, before the expiry steps, artifacts in identifier order. A step that returns its state is dropped.
   - One artifact per step is enough. Revising two artifacts is two steps through a state a runner may stop in.
6. **R3 restated.** A state is *open* when some sequence of action steps (record and revise), supplied, external revise, expiry and authority steps reaches a legitimate outcome.
   - The external revise step is not an action step. It is outside the test "every step of every admissible action returns this state", as supplied steps are.
   - A state that meets the second half of `stuck-state` or `cycle`, and is open only because of this step, is counted as waiting.
   - Summary: a protocol with an artifact that declares `revised_by` prints `` W waiting on supplied evidence, expiry or a revision by another protocol `` where today it prints `` W waiting on supplied evidence or expiry `` (`simulate/mod.rs:317-336`, `:960` on `impl/canon-simulate`). Every other protocol gets today's bytes.
7. **Soundness.**
   - *Reported dead ends that stop.* A state stuck or cycling on a failed or missing record about `a` is no finding when `a` declares `revised_by`. It is a real dead end if the named protocol never revises `a`, or revises it only under conditions that never hold.
   - *Why this is the `supplied_by` trade.* A supplied step assumes that a record of the kind may arrive at any step. Nothing checks that the supplier produces it (story:supplied-evidence § Scope, Out). This step assumes that a revision may arrive at any step. Nothing checks that the named protocol revises `a`. Both rest on a declaration Canon cannot see into.
   - *Still reported.* A path to a finding may hold the step. It is a path a runner may take if the declaration holds and a revision may arrive at any step. If the named protocol revises only when this case allows it, that path may not exist. The same limit holds for supplied steps.
   - *Not silent.* Each waiting state is counted in `W`.
   - *No other dead end goes unreported.* The states that are not open are closed under every step, because a successor of such a state that was open would make the state open. The completeness argument of story:simulate-revise-step § Outcome 4 applies with the external step in the closure and outside the idle test. R5 then prints a contained state.

## Why

**The request.** Request "revised_by" in `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/17/request.md`: option C of story:simulate-revise-step, draft only. The `sdc` session decides after option A is implemented and the class a2 count is measured by the binary.

**The class a2 findings** (24 of 87, `requests/16/classification.txt` and `classify.out`, class `ext`; numbers are positions in the output at `f364b6b`). The protocol that revises each artifact is read from the action descriptions.

| Protocol | Findings | Waits on | Protocol whose cases revise it |
|---|---|---|---|
| sdc, sdc.direction | 0 | | |
| sdc.epic | 1 | `roadmap` | sdc.direction (`roadmap.write`, `sdc-direction.protocol.yaml:239-242`) |
| sdc.intake | 1-5, 11-15, 24-27, 32-35 (18) | `idea` | none found |
| sdc.plan | 1 | `roadmap` | sdc.direction |
| sdc.plan | 2 | `spec` | sdc.spec (`spec.author`, `spec.reuse`, `spec.reconstruct`, `sdc-spec.protocol.yaml:192-226`) |
| sdc.spec | 1 | `roadmap` | sdc.direction |
| sdc.story | 1 | `spec` | sdc.spec |
| sdc.release | 1 | `spec` | sdc.spec |

- By artifact: `idea` 18, `roadmap` 3, `spec` 3. sdc.epic 3 holds an a2 record as well, but is class e, and stays a finding.
- Every sdc.spec case is opened by `sdc.epic` (`sdc-epic.protocol.yaml:121`).
- No SDC action says it revises `idea`. The idea is given by the person, and a revised idea opens a new intake case (`sdc-intake.protocol.yaml:6`, "so this case declares no invalidation from the idea"; `sdc.protocol.yaml:6`). So `revised_by` cannot cover the 18 intake findings: no protocol id fits, and they are real dead ends of the intake case.
- On the 2026-11-21 classification, C would turn at most 6 of the 24 into waiting.

**The measurement that decides it.** Take it after story:simulate-revise-step is merged and the SDC protocols declare `revises` on every write action.
1. Run `canon simulate --path` on the eight protocols. Record `F` findings and `S` states from each summary line.
2. M1 is the count of printed findings with a record in the state that is about an artifact no action of that protocol lists in `revises`, and whose result is not the one an outcome needs (rule `ext` of `classify.py`).
3. M2 is the binary-only check. For each protocol, copy it and add, for every artifact no action revises, one action with no precondition, no `may_produce` and `revises: [that artifact]`. M2 is `F` before minus `F` after.
4. Report M1, M2, and for each finding the artifact and whether a protocol revises it. If M1 and M2 differ, list the findings that differ. R5 counts per kind, so a stuck state that becomes a cycle can shift `F` by a few.
5. The a2 count is M1. The number C can turn into waiting is the part of M1 whose artifact has a revising protocol (on 2026-11-21: 6).

**Verified.** I read `simulate/mod.rs` at `4b36d52` (the supplied step `:655-670`, open states `:734-757`, rendering `:788-812`), the `supplied_by` code named above and story:supplied-evidence in full. I ran `canon-sim simulate` on `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/18/doc.protocol.yaml` (artifact `doc`, a check that runs once, outcome needs `pass`). Without the declaration it gives `3 reachable states, 1 terminable (outcome `done` 1), 1 finding (1 stuck or cycle state)`, exit 1, one `stuck-state` on `{evidence `test` about `doc`}`.

**Not verified.** No build exists with a revise step, so no post-A count and no byte of the new step. The summary in Acceptance 5 is predicted. I did not construct a fixture whose printed finding path holds the new step (R5 prints the smallest state, and a path holds the step only when the state holds a variant a revision makes). I did not read the SDC driver.

## Decision needed

**1. When the external step is offered.**

| Option | Walk and model | Cost and risk |
|---|---|---|
| A. In every state | One step per declared artifact. Where no record is affected it returns the state and is dropped | No extra reachable state beyond the check's product. Matches "offered in every state" |
| B. Only while a current record about the artifact exists | The same step, with a guard | A revision of `a` also moves records of dimensions with upstream `a` and `children` outcomes (story:simulate-revise-step § Outcome 2). So B could report a state that such a revision frees, which is a false finding (read from that story, not run) |

Recommendation: A, because the step stays a statement about the other protocol and B can print a false finding.

**2. The `canon diff` class.**

| Option | Classes (add, remove, replace) | Reason |
|---|---|---|
| X | `NO SEMANTIC CHANGE` for all three | Evaluation, check and generate read nothing. The rule of `supplied_by` changed in review only because option B made `canon evaluate` read it |
| Y | `EXPANSION`, `BREAKING`, `BREAKING` (`diff/mod.rs:24-26`) | Mirrors `supplied_by`, but nothing evaluated reads the field and the runner takes no action from it |
| Z | `RELAXATION`, `TIGHTENING`, undecided (as `revises` in story:judgement-marking § Outcome 5) | Follows the follow-up checks, a step widens what may move. The story gives no class for a replacement |

Recommendation: X. story:simulate-record-groups says `NO SEMANTIC CHANGE` "as it does for `supplied_by`", which is out of date since the review of story:supplied-evidence.

## ESS first

- **First commit.**
  - In `ess/domains/protocol.yaml`, `canon.protocol.Artifact` (`:83`) gets `revised_by: Optional<canon.protocol.ProtocolId>`, with `# read from:` this story.
  - `canon.cli.Simulate` in `ess/domains/cli.yaml` gets a comment on the external step and on the summary phrase.
  - New fixtures: `fixtures/investigation/revised-by/` (a valid protocol and a pair that differs only in the declaration), `fixtures/investigation/invalid/revised-by-self.yaml`, and `fixtures/investigation/simulate/revised-by/` and `revised-by-undeclared/`, each with a `protocol.yaml` and a hand-written `report.txt`.
  - The named test of Acceptance.
- **Red on that commit.**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` fails naming `revised_by`, because the Rust `Artifact` (`crates/canon/src/model/mod.rs:175`) has no such field.
  - The named test fails because `deny_unknown_fields` refuses `revised_by` at compile.
- No `conformance/requirements.yaml` entry: evaluate, check and generate do not read the field.

## Scope

- **In:** Outcome 1 to 7. — cited
- **Out:**
  - a list of protocols;
  - a check that the named protocol declares or revises the artifact;
  - an artifact revised by no protocol, such as `idea`;
  - the SDC edits that add `revised_by`;
  - the child-outcome step;
  - any change to `canon check`. — cited
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `ess/domains/cli.yaml` — cited
  - `crates/canon/src/model/mod.rs` (`Artifact` `:175`) — cited
  - `crates/canon/src/validate/mod.rs` (`Problem` `:283`, code table `:389`, kind loop `:854`, named identifiers `:797`) — cited
  - `crates/canon/src/ir/mod.rs` (`:130`, `:275`, `:623`, header doc `:22-29`) — cited
  - `crates/canon/src/eval/read.rs` (`:220`) — cited
  - `crates/canon/src/diff/mod.rs` (`:18`, `:743-755`) — cited
  - `crates/canon/src/simulate/mod.rs` (`Step`, step loop, open states, `Report` display `:317-336`, `:960`) — cited
  - `crates/canon/src/ir/imports.rs` (generic merge, no change expected) — inferred
  - `crates/canon-cli/tests/revised_by.rs` (new) — inferred
  - `fixtures/investigation/revised-by/` and `invalid/revised-by-self.yaml` — inferred
  - `fixtures/investigation/simulate/revised-by*/` — inferred
- **At integration:** the generated `website/docs/reference/protocol.md`, `canon-ir.md`, `cli.md`, `website/static/schemas/protocol-1.schema.json` and `website/status.yaml`.
- **Confidence:** high for the declaration, validate, IR and diff, which copy `supplied_by`. Medium for the walk, which waits on story:simulate-revise-step and its `Step` and `Layout` changes.

## Order

- **Depends on:** story:simulate-revise-step. It is third after the child-outcome step and story:simulate-record-groups. This story is fourth and builds on option A of that story.
- **`crates/canon/src/simulate/mod.rs`:** shared with story:canon-simulate, the child-outcome step, story:simulate-record-groups and story:simulate-revise-step. Run it after all of them.
- **Model, IR, validate, diff, `eval/read.rs` and `ess/domains/protocol.yaml`:** shared with story:judgement-marking and story:simulate-record-groups. story:protocol-writer shares `model/mod.rs` and `protocol.yaml`, and must write the field. story:revision-ordering shares `diff/mod.rs`, `ir/mod.rs`, `validate/mod.rs`, `eval/read.rs` and `protocol.yaml`. All are active. `aep plan artifact waves` keeps them apart.
- **Gate:** not accepted until the `sdc` session has the measurement above.

## Acceptance

`crates/canon-cli/tests/revised_by.rs::artifact_revised_by_another_protocol_is_waited_on` passes:

1. `canon validate` on `fixtures/investigation/revised-by/` exits 0. On `invalid/revised-by-self.yaml` it exits 1 with `self-revised-artifact` naming the artifact. With `revised_by: 'q q'` it exits 1 with `invalid-identifier`.
2. `canon compile` writes `"revised_by": "other"` on the artifact and no key without it. The IR of every existing fixture protocol is byte-identical to before. The IR with the own id is refused on read-back.
3. `canon evaluate`, `canon check` and `canon generate` give the same bytes and exit codes on the two fixtures that differ only in `revised_by`.
4. `canon diff` between them, in each direction and with a changed protocol id, reports `NO SEMANTIC CHANGE`.
5. `canon simulate` on `simulate/revised-by-undeclared` (the `doc` protocol of the scratch directory) exits 1 with one `stuck-state` and the summary line above. On `simulate/revised-by` it exits 0, with only the summary `` simulated: protocol `p` revision 1: 3 reachable states, 1 terminable (outcome `done` 1), 1 waiting on supplied evidence, expiry or a revision by another protocol, 0 findings (0 stuck or cycle states) `` (predicted).
6. `canon simulate` on a protocol that declares `supplied_by` and no `revised_by` gives the bytes of today, with the phrase `` waiting on supplied evidence or expiry ``.
7. `simulate::tests::an_external_revision_is_written_as_protocol_revises` passes: the step renders as `` protocol `q` revises `a` `` and is tried after the supplied steps and before the expiry steps.
8. Two runs of each new fixture give identical bytes.

## Source

- **Request:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/17/request.md`, request "revised_by". Brief: `requests/drafter-brief-2.md`.
- **Stories read** (`aep plan artifact show`): story:simulate-revise-step, story:supplied-evidence, story:simulate-record-groups, story:judgement-marking, story:canon-simulate.
- **Canon files read:** `crates/canon/src/{validate,ir,diff,model}/mod.rs`, `ir/imports.rs`, `eval/read.rs`, `ess/domains/protocol.yaml`, and `simulate/mod.rs` at `4b36d52` (tree `canon-w4-canon-simulate`).
- **SDC files read** (`f364b6b`): the eight protocols; `requests/16/{classification.txt,classify.out,classify.py}`; `w5/sdc-run/*.txt`.
- **Commands:** `canon-sim validate` and `simulate` on `requests/18/doc.protocol.yaml` and a `supplied_by` variant. 2026-11-21.

## Decided

Accepted on 2026-11-22 on the operator's word in the coordinating session: "accept the main
story, and then accept all of them, put all of them into a single way" (the plan page's
§ Replan 16 holds the whole message and the coordinator's reading of it). The line of § Order,
"not accepted until the `sdc` session has the measurement", is replaced by that word
(`approval-record:accept-artifact-revised-by-2026-11-22`).

The two choices of § Decision needed, taken by the coordinator as the story recommends:

- **1 A.** The external revise step is offered in every state; a step that returns its state is
  dropped.
- **2 X.** `canon diff` reports adding, removing and replacing `revised_by` as
  `NO SEMANTIC CHANGE`.

The unit builds these and does not reopen them. The `file:line` references of this story were
read from trees older than the wave branch of 2026-11-22: the unit reads each one again before
it builds on it. story:simulate-revise-step and story:child-outcome-recovery change the same
walk; this unit merges after the second.
