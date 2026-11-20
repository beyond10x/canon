---
format: aep.planning-md/3
id: story:supplied-evidence
kind: story
status: draft
title: 'Declare evidence kinds supplied by another protocol: supplied_by, and canon check counts them as produced'
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:protocol-imports
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: inferred
  path: conformance/scenarios/supplied-evidence.yaml
- confidence: inferred
  path: crates/canon-cli/tests/supplied_evidence.rs
- confidence: cited
  path: crates/canon/src/check/mod.rs
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/invalid/supplied-by-self.yaml
- confidence: inferred
  path: fixtures/investigation/supplied-evidence/
- confidence: inferred
  path: website/status.yaml
revision: 14
---
## Outcome

An evidence kind can declare that records of it come from outside the protocol. It names the protocol whose cases produce them. `canon check` then counts the kind as produced, and stops reporting `unproduced-evidence` for it.

1. **`supplied_by` on an evidence kind in `protocol/1`.** This is an optional field on `evidence_kinds.<id>` that holds one protocol id. An example is `roadmap_approval: {supplied_by: sdc.direction}`. `canon validate` refuses a protocol that names its own id as the supplier, as `self-supplied-evidence`, naming the kind. This mirrors `self-opened-case`.
2. **`canon-ir/1`.** The compiled evidence kind carries `supplied_by` when it is set. When it is absent there is no key, so the IR of every existing protocol stays byte-identical. The IR reads it back.
3. **`canon check`.** A kind is produced when an action lists it in `may_produce`, when it is `case_outcome` and some action has `case_open`, or when it declares `supplied_by`. The state space does not change.
   - For `authority-bypass`, a supplied kind is not governed in this protocol.
   - A state where a supplied kind is present can still be a witness. The supplier's authority is checked in the supplier's protocol.
   - Today a state with an unproduced kind present is never a witness (`check/mod.rs:745`). So without this rule, an outcome that needs a supplied kind would hide a bypass through any other kind.
4. **`canon evaluate`.** Nothing changes. Records of a supplied kind are read as given, like any other record. With no record the claim is `UNKNOWN`.
5. **`canon diff`.** Adding, removing or changing `supplied_by` is `NO SEMANTIC CHANGE`, because no evaluation reads it.

## Why

- **The request.** Item 6 of `$HOME/beyond10x/sdc/docs/canon-requests.md`.
  - `canon check` on `$HOME/beyond10x/sdc/docs/limitations/epic-imported-evidence.protocol.yaml` reports 8 `unproduced-evidence` findings for `roadmap_approval`, at 36864 states (`output.txt:14-22`).
  - The SDC works around it with a `roadmap.read` action that "copies" the record. That action is in `sdc.protocol.yaml:72`, `sdc-epic.protocol.yaml:91-96`, `sdc-spec.protocol.yaml:133-138` and `sdc-plan.protocol.yaml:94-99`.
  - Canon cannot tell that copy from a forgery (`docs/protocol-notes.md:168-173`).
- **The supplier is not always the parent.**
  - Every copy is taken from `sdc.direction` (`protocol-notes.md:59`; `sdc-epic.protocol.yaml:93`).
  - For the root `sdc`, `sdc.direction` is a child (`sdc.protocol.yaml:6`). For `sdc.epic`, it is the parent's child.
  - So "parent-to-child" is really "from another case's protocol".
- **The boundary.**
  - `boundary.md:108` classes this as a small Canon language change plus runtime work: "the copy itself is the runtime's".
  - Canon opens nothing (`crates/canon/src/model/mod.rs:213`). Canon evaluates one case from the evidence it is given.
  - Finding the record in another case and putting it unchanged into the child's evidence set stays the runner's job (Loom, `boundary.md:87`).
- **Verified in the canon tree.**
  - `unproduced-evidence` is computed from `produces` (`crates/canon/src/check/mod.rs:599-614`, `:702-708`): `may_produce`, plus `case_outcome` when an action has `case_open`. No other source exists.
  - Bypass uses `governed` and `free` from the same function (`:712-723`, `:745`).
  - `CaseOpening` names a protocol and a subject only (`model/mod.rs:228`; `ess/domains/protocol.yaml:199-219`).
  - `children` reads `case_outcome` only (`model/predicate.rs:52-59`, `:251`).
- **Case inputs cannot carry the approval.**
  - An input is fixed when the case opens and is never unknown (`model/mod.rs:126`, `predicate.rs:279`).
  - story:case-inputs § Scope Out excludes "inputs whose value changes during a case".
  - An input is one of a closed list of identifiers (`protocol.yaml:385-398`). It has no `subject_revision`, `producer` or `observed_at` (`protocol.yaml:795`, `:821`).
  - So an approval of roadmap r1 would still hold at r2, and a missing approval would be `FALSE` or a fake value, not `UNKNOWN`.
- **Not verified.**
  - I did not run `canon check`, because I am read-only.
  - The claim that the state count does not change rests on two readings, not a run: `output.txt:22` gives 36864 states with the kind unproduced, and `protocol-notes.md:148` gives 36864 with `roadmap.read`.

## Decision needed

| Option | `ess/` and model | Runner's job | Closed-IR rule (design `:1683`, D206) | Cost |
|---|---|---|---|---|
| A. `supplied_by` on the child's evidence kind | `EvidenceKind.supplied_by: Optional<ProtocolId>`; IR field; check's `produces` | Copy the record unchanged from the supplier's case | Kept. The id is a name, like `case_open.protocol`. Compile, evaluate and check each read one document; nothing resolves the name | One field, one validation code, two check rules |
| B. Mapping on the parent's `case_open` (own kinds or claims to child kinds or inputs) | `CaseOpening` gains a map of ids declared in another protocol | Same copy, now described by Canon but done by the runner | Broken, or unchecked. Compile must read the child document, pinned as in story:protocol-imports, or leave the ids unvalidated | Larger. The child's own `canon check` still reports, since check reads one IR. Does not cover the root, whose supplier is its child `sdc.direction` |
| C. Case inputs only | None | Turn a record into an input value at open | Unaffected | Loses revision binding, producer, expiry and `UNKNOWN` (see Why); each value multiplies check states |
| D. Keep the copy action (status quo) | None | Honour "copy" in an action description | Unaffected | A fake action that is admissible and listed for a runner to take; Canon cannot tell copy from forgery |

Recommendation: option A, because it removes the false findings, names the supplier and keeps every IR closed. The resolved-supplier check is left to a later set check, modelled on `ir::check_openings`. The acceptance below is for option A.

## ESS first

- **First commit.**
  - In `ess/domains/protocol.yaml`, the optional field `supplied_by` of type `canon.protocol.ProtocolId` on `canon.protocol.EvidenceKind` (`:90`).
  - The fixtures `fixtures/investigation/supplied-evidence/`.
  - The scenario file `conformance/scenarios/supplied-evidence.yaml`, with `CANON-SUPPLY-001` added to `conformance/requirements.yaml`.
- **Red on that commit.**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` fails naming `supplied_by`, because the Rust `EvidenceKind` (`model/mod.rs:159`) has no such field.
  - `canon conform run --requirements conformance/requirements.yaml` fails `CANON-SUPPLY-001` at compile, because `deny_unknown_fields` refuses `supplied_by`.
- The named test `supplied_evidence_is_produced_outside` is code and arrives with the implementation.

## Scope

- **In:** Outcome 1 to 5. — cited
- **Out:** — cited
  - Moving or copying records between cases (runner, Loom).
  - A set check that the named supplier declares and produces the kind.
  - Usefulness of a supplied kind (request item 2).
  - Child outcome bound to the parent's revision (item 5).
  - The SDC edit that drops `roadmap.read`.
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/mod.rs` — cited
  - `crates/canon/src/validate/mod.rs` — cited
  - `crates/canon/src/ir/mod.rs` — cited
  - `crates/canon/src/eval/read.rs` — inferred, it rebuilds `EvidenceKind` at `:226`
  - `crates/canon/src/check/mod.rs` — cited
  - `crates/canon/src/diff/mod.rs` — cited, exhaustive destructure at `:854`
  - `crates/canon-cli/tests/supplied_evidence.rs` (new) — inferred
  - `fixtures/investigation/supplied-evidence/` (new) — inferred
  - `fixtures/investigation/invalid/supplied-by-self.yaml` — inferred
  - `conformance/scenarios/supplied-evidence.yaml` — inferred
  - `conformance/requirements.yaml` — cited
  - Regenerated `website/docs/reference/protocol.md` and `canon-ir.md`, and `website/static/schemas/protocol-1.schema.json` — inferred
  - `website/status.yaml` — inferred
- **Fixture placement:** the fixtures stay outside `fixtures/investigation/check/`, so the `CASES` table in `crates/canon-cli/tests/canon_check.rs` does not change. — inferred
- **Confidence:** high for model, ESS and check (read); medium for diff and read.

## Order

- **Depends on:** story:protocol-imports.
  - It shares `ess/domains/protocol.yaml`, `model/mod.rs`, `validate/mod.rs` and `ir/mod.rs`.
  - Its rule refuses an id two imports declare differently, so two parts with different `supplied_by` are refused. This story must not change that.
- **Shares files without an edge:**
  - story:revision-ordering (active): `protocol.yaml`, `validate/mod.rs`, `ir/mod.rs`, `eval/read.rs`, `diff/mod.rs`.
  - story:capability-scope (implemented, same files).
  - story:check-subject-slices (draft): `check/mod.rs`.
  - story:authority-revision-binding (draft): `protocol.yaml`.
- **Waves:** `aep plan artifact waves` keeps each of these in a separate wave.

## Acceptance

The named test `supplied_evidence_is_produced_outside` in `crates/canon-cli/tests/supplied_evidence.rs` passes, with these expectations:

1. `canon conform run` reports `CANON-SUPPLY-001` passed. With a supplied kind and no record, its claim is `UNKNOWN`; with a record given, it is `TRUE`.
2. `canon compile` of the supplied fixture writes `"supplied_by": "investigation.review"` on that kind.
3. `canon compile` of a protocol without `supplied_by` gives the same bytes as before the story.
4. `canon check` on the variant with the kind unsupplied reports `unproduced-evidence` for every reader, exit 1.
5. `canon check` on the supplied variant reports 0 findings, exit 0, with the same state count as (4).
6. A variant whose outcome reads the supplied kind and a governed kind that a capability-free action may also produce reports `authority-bypass`.
7. The same variant, with the kind neither supplied nor produced, reports no `authority-bypass`.
8. `canon validate` on `fixtures/investigation/invalid/supplied-by-self.yaml` exits 1 with `self-supplied-evidence`, naming the kind.
9. `canon diff` between the unsupplied and supplied variants reports `NO SEMANTIC CHANGE` for the kind.
10. `crates/canon/tests/ess_model_matches.rs` passes.

## Source

- Request item 6, "Parent-to-child evidence", in `$HOME/beyond10x/sdc/docs/canon-requests.md`.
- Read in `$HOME/beyond10x/sdc/`:
  - `docs/limitations/output.txt`
  - `docs/limitations/epic-imported-evidence.protocol.yaml`
  - `docs/boundary.md`
  - `docs/protocol-notes.md`
  - `protocol/sdc.protocol.yaml`, `sdc-epic.protocol.yaml`, `sdc-spec.protocol.yaml`, `sdc-plan.protocol.yaml`, `sdc-direction.protocol.yaml`
  - `scripts/roadmap-read-action.yaml`
- Read in the canon tree at `15cfece`:
  - `AGENTS.md`
  - `crates/canon/src/model/mod.rs`, `model/predicate.rs`
  - `crates/canon/src/check/mod.rs`
  - `crates/canon/src/diff/mod.rs`
  - `ess/domains/protocol.yaml`
  - `crates/canon-cli/tests/canon_check.rs`
  - design § 39.5
  - story:case-composition, story:case-inputs, story:protocol-imports, story:capability-scope
- Date: 2026-11-20.
