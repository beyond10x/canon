---
format: aep.planning-md/3
id: story:check-subject-slices
kind: story
status: active
title: Check a protocol beyond 65,536 states by deciding each subject over the dimensions it reads
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/tests/canon_check.rs
- confidence: cited
  path: crates/canon/src/check/mod.rs
- confidence: cited
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/check/tests.rs
- confidence: cited
  path: crates/canon/src/generate/mod.rs
- confidence: cited
  path: crates/canon/src/generate/tests.rs
- confidence: cited
  path: crates/canon/tests/adversary2_check.rs
- confidence: cited
  path: crates/canon/tests/adversary2_invalidation.rs
- confidence: cited
  path: crates/canon/tests/adversary_check_space.rs
- confidence: inferred
  path: fixtures/investigation/check/sliced/
- confidence: cited
  path: fixtures/investigation/check/state-space-bound/
- confidence: inferred
  path: fixtures/investigation/check/unread-dimension/
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 21, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 22, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

`canon check` treats each outcome, each action and each declared property as a subject, and checks each one over the product of the dimensions that subject reads (its slice). Today it checks one product of every dimension. `STATE_BOUND` (65,536) then applies to each slice, not to the whole product. The five finding codes, their order, their witness states and their counterexamples stay byte-identical to an enumeration of the whole product.

1. The slice of an action holds the evidence dimensions its precondition reads, directly or through the claims it tests, with the history, later-record, last-record and order dimensions those dimensions carry. It also holds the case inputs the precondition tests and one capability dimension per entry in its `requires`. The slice of an outcome holds the dimensions its `requires` reads, or its one decision dimension. The slice of a property is its subject's slice together with the dimensions its `independent_of` claim reads. In each slice, a dimension's classes, upstream variants and producer profiles are derived only from the evidence matches inside that slice.
2. Every dimension outside a slice holds the value the whole-product witness would hold: no record, capability undecided, decision not taken. A state is rendered as the whole-product state with those values.
3. `unreachable-outcome`, `unsatisfiable-precondition`, `authority-bypass` and `property-failed` are each decided over their subject's slice. `unproduced-evidence` stays static.
4. The summary line still gives the size of the whole product: `` checked: protocol `p` revision 1: 131072 states, … ``. `Report::states` is documented as "states covered", not "states evaluated". Every report of a protocol whose whole product is within the bound stays byte-identical.
5. If any slice exceeds the bound, the whole check is refused as `state-space-bound`, and nothing else is reported. The refusal names the first such subject in finding order, that slice's count and that slice's dimensions: `` protocol `p` revision 1: action `a` reads 196608 states, more than the bound of 65536: … ``. The numbers in the list multiply to the slice count.
6. `canon generate` keeps today's whole-product bound with its own guard. It enumerates the whole product after `check::check` accepts (`crates/canon/src/generate/mod.rs:204-213`), so without the guard it would enumerate a product beyond the bound.

No document format, command or flag changes.

## Why

- The request is item 3 of `$HOME/beyond10x/sdc/docs/canon-requests.md:24-28`. Four of the eight limitations are this bound (`docs/limitations/output.txt:2,6,10,26`).
- In two of them the bound changed what the protocol means (`docs/boundary.md:105,107,114-116`):
  - `sdc.story` dropped `plan` and `design`.
  - `sdc.intake` lost its own abandon outcome (`sdc-intake.protocol.yaml:6`).
- Three shipped protocols sit at exactly 65,536 states: `sdc.intake`, `sdc.spec` and `sdc.plan` (`docs/check-output.txt:6,8,14`; `docs/protocol-notes.md:148-151`). `sdc.spec` dropped `spec_self_conformance` to stay within the bound (`protocol-notes.md:104`). A parent's `children` form lists one outcome, because two would cost 16 × 17 (`:151`). Each 65,536-state check takes minutes (`:189`).
- Verified in the canon tree at `15cfece`:
  - The bound is `STATE_BOUND` (`crates/canon/src/check/mod.rs:339`), checked as `size <= STATE_BOUND` (`mod.rs:561`).
  - A space above it is refused before any evaluation, naming the count and every dimension (`mod.rs:562-575`, `space.rs:855-941`).
  - A state takes one value per dimension, and the size is the product of the radices (`space.rs:847-851`):
    - evidence dimensions: `2^(classes)`, `2^(c * 2^m)` with upstream artifacts, `n^(c * 2^m)` with producer profiles;
    - capabilities: 3 values;
    - decisions: 2 values;
    - case inputs: one value per declared value;
    - history, later-record, last-record and order dimensions (`mod.rs:8-33, 106-122`).
  - Every state and its evaluation are held in memory (`mod.rs:576-580`).
- Why slices give the same findings:
  - An action's status depends only on its precondition and the authority for its `requires` (`eval/actions.rs:1-30`).
  - An outcome's status depends only on its `requires` or its decision (`eval/outcomes.rs:1-14`).
  - Each of the four state-dependent findings reads only its subject's status (`mod.rs:583-598, 711-837`). `property-failed` additionally reads the dimensions erased by `independent_of`.
  - The parts a slice needs already exist: `Space::dimensions_read` (`mod.rs:802`), `dimensions_read_positively` (`mod.rs:735`) and `Space::erase` (`mod.rs:750, 808`).
- My reading of the slice sizes, before the decision dimension is removed (inferred, not run):

  | Protocol | Whole product | Largest slices | Fits after slicing |
  |---|---|---|---|
  | derived-verified-claim | 196,608 | `delivered` 32,768; `change.merge` 49,152 | yes |
  | intake-with-abandon | 131,072 | `passed` 65,536 | yes |
  | monolith | 2,654,208 | `operating` 576; `release.publish` 3,456 | yes |
  | four-gate-kinds | 786,432 | `passed` 131,072; `change.merge` 196,608 | no |

  For four-gate-kinds this matches `boundary.md:109`.
- The existing fixture `fixtures/investigation/check/state-space-bound/` is refused only because of a claim that no action, obligation or outcome reads (`measurement.graded`, `protocol.yaml:21`; `refusal.txt:1`, `measurement 2048`). Under this story it is checked.
- Boundary: this is the checker over Canon's own semantics. It is pure and needs no runner or engineering vocabulary. `canon simulate` (item 4) is a different story.
- Could not verify: no `canon` binary is on `PATH` and I ran no build, so every slice size above comes from reading the protocols. I did not check ADR 0094's 5,971,968-state protocol (`boundary.md:115`).

## Decision needed

| Option | What it can still prove | Change in `ess/` and model | Cost |
|---|---|---|---|
| A. One slice per subject, refused as a whole when any slice exceeds the bound | Everything today's check proves. Exact, because statuses depend only on what their subject reads. Fixes 3 of 4 refusals; four-gate-kinds stays refused | none; `Space` is built from a subset of readers; `Report::states` keeps its meaning as states covered | `space.rs` and `mod.rs`; the fixture and refusal texts that pin the bound change |
| B. Raise `STATE_BOUND`, or add `--max-states` | Everything, over the whole product | `max_states: Optional<String>` on `canon.cli.Check` in `ess/domains/cli.yaml` | Time and memory grow with the state count: 786,432 states are 12 × a run that takes minutes, 2,654,208 are 40 × (`mod.rs:576-580`) |
| C. A, but report the subjects that fit and refuse only the others by name | Findings for the subjects that fit. Nothing about the named ones | a report format for a partial result and its exit status | Proves less; the result is named, not silent |
| D. Split an `all` whose members read disjoint dimensions, one member at a time | `unreachable-outcome` and `unsatisfiable-precondition` only. `authority-bypass` and `property-failed` still need the joint slice | none | Would take four-gate-kinds' `passed` below the bound for two findings only |
| E. A bounded or sampled walk | Only findings of the "some state" kind. Never "no state", so `unreachable-outcome`, `unsatisfiable-precondition` and the absence of any finding are lost | item 4, `canon simulate` | Out of this story |

Recommendation: A, because it proves exactly what today's check proves, changes no format, and admits three of the four refused protocols; D can follow as its own story if four-gate-kinds must be checked without collapsing its gate kinds.

## ESS first

Option A changes nothing under `ess/`: no flag and no document changes. `ess/domains/check.yaml:3-5` leaves the state space to Canon's own suite, `crates/canon-cli/tests/canon_check.rs`.

The first commit adds only fixtures and that test's table:

- `fixtures/investigation/check/sliced/protocol.yaml` and `report.txt`:
  - The protocol's whole product exceeds 65,536 and every slice fits.
  - It produces one finding of each code.
  - `report.txt` is the output of today's whole-product enumeration with the bound raised locally. That edit is not committed, and the commit message records the command.
- `fixtures/investigation/check/unread-dimension/`: today's `state-space-bound` protocol with its `report.txt`.
- A new `state-space-bound/` protocol in which one subject's slice exceeds the bound, with the new `refusal.txt`.
- `CASES` in `canon_check.rs` grows from 7 to 9.

The test is red on that commit because today's check refuses `sliced` and `unread-dimension` with exit 1 on standard error, and because the new refusal text names a subject. If B is chosen instead, the first commit declares `max_states` in `ess/domains/cli.yaml`, and `crates/canon-cli/tests/ess_commands_match.rs` is red.

## Scope

- In:
  - slice construction and per-subject deciding in `canon check` (cited);
  - the refusal naming a subject (inferred);
  - the guard that keeps `canon generate` at the whole-product bound (cited);
  - fixtures and tests that pin the bound (cited);
  - the doc comment of `Check` and the generated page (cited);
  - a capability line in `website/status.yaml` (cited).
- Out:
  - `canon simulate` (item 4);
  - options B to E;
  - reporting `unproduced-evidence` when a check is refused;
  - slicing in `canon generate`;
  - any change under `ess/`;
  - SDC protocol edits.
- Surfaces:
  - `crates/canon/src/check/mod.rs` (cited)
  - `crates/canon/src/check/space.rs` (cited)
  - `crates/canon/src/check/tests.rs`: lines 146, 240, 436, 618 pin the refusal (cited)
  - `crates/canon/src/generate/mod.rs` (cited)
  - `crates/canon/src/generate/tests.rs`: line 221 compares generate's refusal with check's (cited)
  - `crates/canon/tests/adversary_check_space.rs` (cited)
  - `crates/canon/tests/adversary2_check.rs` (cited)
  - `crates/canon/tests/adversary2_invalidation.rs` (cited)
  - `crates/canon-cli/tests/canon_check.rs` (cited)
  - `crates/canon-cli/src/lib.rs`: the `Check` doc comment at line 120 (cited)
  - `website/docs/reference/cli.md`: generated (cited)
  - `website/status.yaml` (cited)
  - `fixtures/investigation/check/sliced/`, `unread-dimension/`, `state-space-bound/` (inferred)
- Confidence: medium. The mechanism and the call sites were read. The fixture design and the other tests that would change were not enumerated by running the suite.

## Order

- Depends on no open story. story:canon-check is implemented.
- Shares `crates/canon/src/check/space.rs` with story:revision-ordering (inferred scope).
- Shares `crates/canon-cli/src/lib.rs` and the generated `cli.md` with story:protocol-imports, story:protocol-floor and story:canon-mutate. Run it in a wave that has none of them.
- Does not contradict story:protocol-imports expectation 6: a composed root and its flattened form get one report either way.
- Does not contradict story:invalidation-evaluation-cost, which leaves the cost of `canon check` out of its scope.

## Acceptance

`crates/canon-cli/tests/canon_check.rs`:

1. The six existing `report.txt` fixtures pass unchanged, byte for byte.
2. `sliced` exits 1, prints its committed `report.txt` exactly and writes nothing to standard error. That report is the output of the whole-product enumeration.
3. The `sliced` summary gives the whole-product state count, which is above 65,536.
4. `unread-dimension` exits 0 and its summary reads `98304 states`.
5. `state-space-bound` exits 1 with `error[state-space-bound]` naming one subject. The numbers it lists multiply to the slice count it names.
6. `canon generate` over `sliced` exits 1 with `state-space-bound`.
7. Two runs over each fixture give identical bytes.

## Source

- Request item 3 of `$HOME/beyond10x/sdc/docs/canon-requests.md`, 2026-11-20.
- Read in sdc: `docs/boundary.md` §2, `docs/limitations/output.txt`, `docs/limitations/{intake-with-abandon,story-with-derived-verified-claim,story-with-four-gate-kinds,monolith}.protocol.yaml`, `docs/protocol-notes.md:80-192`, `docs/check-output.txt`, `docs/canon-simulation-request.md`.
- Read in canon at `15cfece`: `AGENTS.md`, `crates/canon/src/check/{mod,space}.rs`, `crates/canon/src/generate/mod.rs`, `crates/canon/src/eval/{actions,outcomes}.rs`, `crates/canon-cli/src/{check,lib}.rs`, `crates/canon-cli/tests/canon_check.rs`, `ess/domains/{check,cli}.yaml`, `fixtures/investigation/check/`, the stories canon-check, protocol-floor, protocol-imports, revision-ordering, invalidation-evaluation-cost and canon-mutate.

## Decided

Option A of § Decision needed: one slice per subject, and the whole check is refused when any slice exceeds the bound. Options B to E are out of this story. Decided on 2026-11-20 by the wave coordinator on the drafter's recommendation, after the operator accepted the story without choosing (`approval-record:accept-sdc-requests-2026-11-20`). The unit builds this option and does not reopen the choice.
