---
format: aep.planning-md/3
id: review-result:issue5-parallel-safety-r1
kind: review-result
status: active
title: Issue 5 stories, parallel-safety critic, round 1
relations:
- reviews: story:protocol-imports
- reviews: story:protocol-floor
- reviews: story:semantic-diff
- reviews: story:case-composition
revision: 1
---
needs-revision

story:protocol-imports — claims "this one is unblocked and goes first" among every story that changes `protocol/1`, but names none of the four drafts it shares `ess/`, `model/`, `validate/` and `ir/` with (story:capability-scope, story:evidence-independence, story:evidence-order-predicate, story:revision-ordering) and has no edge to them, and `aep plan artifact waves` puts it in wave 16, behind capability-scope (13), evidence-independence (14) and evidence-order-predicate (15); add the ordering edges with the shared directories as their reason, or drop "goes first" (cited, both bodies) — .engineering/planning/story/protocol-imports.md:82-84, .engineering/planning/story/capability-scope.md:83-84
story:protocol-imports — adds `--import` to `validate` and `compile`, whose bodies are `validate_command` and `compile_command` in `crates/canon-cli/src/main.rs`, plus the `Command` variants in `lib.rs`, yet its Surfaces omit main.rs, and story:canon-mutate lands on main.rs (its dispatch arm) and the same `Command` enum with no edge and no mention in either body (main.rs listed by canon-mutate: cited; for imports: inferred from the tree); state the shared files and add an ordering edge, or give the two disjoint surfaces — .engineering/planning/story/protocol-imports.md:97-99, .engineering/planning/story/canon-mutate.md:56
story:protocol-floor — the new subcommand needs a `Command` variant in `crates/canon-cli/src/lib.rs` and a `mod floor;` and match arm in `crates/canon-cli/src/main.rs`, but its Surfaces list only lib.rs, and story:canon-mutate lands on the same enum and dispatch with no edge and no mention in either body (cited for canon-mutate, inferred for floor's main.rs); name main.rs and the collision, then add an ordering edge or split the surface — .engineering/planning/story/protocol-floor.md:74, crates/canon-cli/src/main.rs:56-58
story:protocol-imports — its scenario `imports` list and `CANON-IMPORT-001` change `crates/canon/src/conform/`, which story:conformance-suite also owns (this one adds coverage and permutation checks), and neither body mentions the other and no edge orders them (cited, both bodies); add an ordering edge recording the directory, or split it — .engineering/planning/story/protocol-imports.md:98, .engineering/planning/story/conformance-suite.md:32
story:case-composition — says it and story:protocol-floor "touch no common source file and may share a wave", but its scope lists no `website/` path while AGENTS.md says a model or CLI change regenerates the whole owned-whole generated set (`website/docs/reference/`, `website/data/`) in the same change and a landing story adds a row to `website/status.yaml`, which floor also lists, so the two collide on generated files at merge (cited, AGENTS.md; case-composition's own touch is inferred); name the shared generated files and who regenerates last, or add an ordering edge — .engineering/planning/story/case-composition.md:102-103, AGENTS.md:116-118

What I read: 5 artifacts (the four stories in the set plus epic:canon-kernel as parent), with `git diff` and `cat` on the drafts, `aep plan artifact graph` and `aep plan artifact waves`, `gh issue view 5`, the scope blocks and Order text of every other story in the store, and `crates/canon-cli/src/{main,lib,diff}.rs`, `Taskfile.yml` and `AGENTS.md` in the tree. Surfaces established: 4 stories cited, 0 inferred, 0 unplaced. Acceptance reads traced to a producer in the set: 4 (floor to semantic-diff's classifier, floor to the imports `--import`, imports to semantic-diff's diff.rs, case-composition to imports). `depends_on` edges record 4 of those 4.

What I could not establish:
- Surface of story:ess-command-surface: it has no scope. Story:protocol-floor says that story declares `canon floor` in `ess/`, but its Outcome lists only validate, compile, evaluate, conform run and diff. If it also declares flags, `--import` and floor would land on a command-declaration file that does not exist yet, and I cannot place it.
- Out of my lane, for the design or acceptance critic: `canon diff --from --to` takes compiled `canon-ir/1`, which is already closed, so `--import` on `diff` (story:protocol-imports Outcome 2) may have nothing to resolve (crates/canon-cli/src/lib.rs:134-141). Also, the new scope blocks mark every path `confidence: inferred` although the bodies' Surfaces sections cite them, which is why `waves` prints "(inferred)".
- I did not check the open decision-blockers on case-composition or whether `canon check` accepts a root with imports (out of lane).

```findings
- file: .engineering/planning/story/protocol-imports.md
  line: 82
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "claims 'this one is unblocked and goes first' among every story that changes protocol/1, but names none of story:capability-scope, story:evidence-independence, story:evidence-order-predicate, story:revision-ordering (shared ess/, model/, validate/, ir/), has no edge to them, and aep plan artifact waves puts it in wave 16 behind three of them; add the ordering edges with the shared directories as reason, or drop the claim"
- file: .engineering/planning/story/protocol-imports.md
  line: 97
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "adds --import to validate and compile (validate_command and compile_command live in crates/canon-cli/src/main.rs) and Command variants in lib.rs, but Surfaces omit main.rs (inferred), and story:canon-mutate lands on main.rs and the same Command enum with no edge and no mention in either body; name the shared files and add an ordering edge, or split the surface"
- file: .engineering/planning/story/protocol-floor.md
  line: 74
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the subcommand needs a Command variant in crates/canon-cli/src/lib.rs and a mod line and match arm in crates/canon-cli/src/main.rs, but Surfaces list only lib.rs (main.rs omission inferred), and story:canon-mutate lands on the same enum and dispatch with no edge and no mention in either body; name main.rs and the collision, then add an ordering edge or split the surface"
- file: .engineering/planning/story/protocol-imports.md
  line: 98
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its scenario imports list and CANON-IMPORT-001 change crates/canon/src/conform/, which story:conformance-suite also owns (cited in both bodies); neither body mentions the other and no edge orders them; add an ordering edge recording the directory, or split the surface"
- file: .engineering/planning/story/case-composition.md
  line: 102
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "says it and story:protocol-floor touch no common source file and may share a wave, but its scope lists no website/ path while AGENTS.md says a model or CLI change regenerates the owned-whole website/docs/reference and website/data files in the same change and a landing story adds a row to website/status.yaml, which floor also lists; the shared generated files are inferred for case-composition; name them and who regenerates last, or add an ordering edge"
```
