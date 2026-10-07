---
format: aep.planning-md/3
id: review-result:issue5-parallel-safety-r2
kind: review-result
status: active
title: Issue 5 stories, parallel-safety critic, round 2
relations:
- reviews: story:protocol-imports
- reviews: story:protocol-floor
- reviews: story:case-inputs
- reviews: story:semantic-diff
- reviews: story:case-composition
- reviews: story:ess-command-surface
revision: 1
---
needs-revision

story:ess-command-surface — has no scope, so `aep plan artifact waves` prints it "unassessed" and puts it in none of the 20 waves, though story:protocol-imports and story:protocol-floor both wait on it. Its Outcome names `ess/` as the surface (cited), and story:case-inputs edits `ess/domains/protocol.yaml` with no edge or sentence ordering the two (the order text puts ess-command-surface only "after story:semantic-diff and before story:protocol-imports"). Give it a scope path (`ess/`, a new file) and state the order against story:case-inputs, either as an edge that records `ess/` as its reason or by naming a command-declaration file disjoint from `ess/domains/protocol.yaml`. The file does not exist yet, so say that — .engineering/planning/story/ess-command-surface.md:20-28, .engineering/planning/story/ess-command-surface.md:34 ("this story is a draft without scope"), .engineering/planning/story/case-inputs.md:33, `aep plan artifact waves` ("unassessed: story:ess-command-surface")

Round-1 findings, each checked against the revised bodies:

| Round-1 finding | Status | What I checked |
|---|---|---|
| story:protocol-imports "goes first" | Fixed | The claim is now scoped to story:case-inputs before story:protocol-imports. The four unedged `protocol/1` drafts are named. `waves` puts them in waves 13 to 17, separate from case-inputs (16) and imports (18). |
| story:protocol-imports omits `main.rs` | Fixed | `main.rs` and `lib.rs` are in its Surfaces, and story:canon-mutate is named. `waves` reports canon-mutate (14) and imports (18) in separate waves. |
| story:protocol-floor omits `main.rs` | Fixed | Same check: canon-mutate (14), floor (20). |
| story:protocol-imports and story:conformance-suite on `crates/canon/src/conform/` | Fixed by naming | The collision is now stated in the body. `waves` puts conformance-suite in 12 and imports in 18. |
| story:case-composition and story:protocol-floor on `website/` | Fixed | The shared paths are named and listed in both scopes. `waves` puts them in 19 and 20. |

I traced every edge the set relies on. `aep plan artifact graph` shows each present: case-inputs to semantic-diff, imports to case-inputs, imports to ess-command-surface, floor to semantic-diff, imports and ess-command-surface, and case-composition to imports.

**What I read:**
- Artifacts: 6 (story:protocol-imports, story:protocol-floor, story:case-inputs in full; the edited sections of story:semantic-diff, story:case-composition and story:ess-command-surface), plus the four round-1 records and story:conformance-suite.
- Commands: `aep plan artifact show` via `cat`, `git diff` and `git status`, `aep plan artifact graph`, `waves` and `relations`, `findings` on the round-1 records, `git grep` of `Predicate` and `enum Command` in the tree, and AGENTS.md § Public documentation.
- Counts:
  - Surface established **cited**: 5 (imports, floor and case-inputs by their body Surfaces; semantic-diff and case-composition by scope).
  - **Inferred**: 0.
  - Could not place: 1, story:ess-command-surface, which is placed at `ess/` only.
  - Acceptance reads traced to a producer in the set: 3 (floor to the semantic-diff classifier, floor to imports, case-inputs to the semantic-diff test). A `depends_on` edge records 3 of 3.

**What I could not establish:**
- Which file story:ess-command-surface would write under `ess/`, and whether story:canon-mutate lands before or after it. Its Outcome says it declares "every subcommand on `main` when this story lands", and no edge orders it against canon-mutate. canon-mutate is outside the set, so I did not file it.
- Out of my lane (scope or design), not counted in the verdict:
  - `website/static/schemas/*.schema.json` (generated, drift-checked by `task docs-check`) is not in the scope of case-inputs, imports or case-composition. These three are already ordered by edges, so no new collision results.
  - case-inputs omits `crates/canon/src/check/space.rs`, which matches on `Predicate` variants and so needs a case for the new `input` leaf.
  - story:protocol-imports does not say how an imported protocol's `case.inputs` compile when an imported claim reads `input:`.
  - The scope blocks mark every path `inferred` although the bodies cite them (carried over from round 1).

```findings
- file: .engineering/planning/story/ess-command-surface.md
  line: 20
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "has no scope, so waves prints it unassessed and schedules it in no wave although story:protocol-imports and story:protocol-floor depend on it; its Outcome names ess/ (cited) as the surface, and story:case-inputs edits ess/domains/protocol.yaml with no edge or sentence ordering the two; add a scope path for ess/ (a new file, not there yet) and either an ordering edge to story:case-inputs recording ess/ as the reason or a command-declaration file disjoint from protocol.yaml"
```
