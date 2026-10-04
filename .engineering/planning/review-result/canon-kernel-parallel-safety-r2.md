---
format: aep.planning-md/3
id: review-result:canon-kernel-parallel-safety-r2
kind: review-result
status: active
title: Canon kernel decomposition — parallel-safety critic, round 2
relations:
- reviews: epic:canon-kernel
- reviews: story:action-admissibility
- reviews: story:canon-ir
- reviews: story:conformance-runner
- reviews: story:conformance-suite
- reviews: story:evidence-freshness
- reviews: story:evidence-revision-binding
- reviews: story:explanation
- reviews: story:obligations
- reviews: story:outcomes
- reviews: story:protocol-source-model
- reviews: story:semantic-diff
- reviews: story:three-valued-claims
revision: 1
---
needs-revision

story:conformance-suite — its scope omits `crates/canon-cli/`, which story:semantic-diff says both stories change, so the waves derivation records no collision between them and rests only on the depends_on edge (cited, both bodies). The fix is to add `crates/canon-cli/` to this story's surfaces, or to say in story:semantic-diff that the edge has another reason — .engineering/planning/story/conformance-suite.md:58 (against .engineering/planning/story/semantic-diff.md:34)

**What I read:** 12 stories, the epic, 2 decision-blockers and the round-1 parallel-safety result. I ran `aep plan artifact list`, `show` on 14 artifacts, `graph`, `waves --kind story` and `validate` (valid). The depends_on edges form one total order: protocol-source-model, canon-ir, conformance-runner, three-valued-claims, evidence-revision-binding, obligations, action-admissibility, outcomes, evidence-freshness, explanation, conformance-suite, semantic-diff. That matches the 12 single-story waves, so no two stories can run at once. `waves` prints 54 collisions and 0 unassessed, and every colliding pair is edge-ordered.

**Round-1 findings:** all five are resolved.
- **Fixtures:** each story now owns a separate variant file under `fixtures/investigation/`.
- **Shared evaluator surface:** the evaluator chain is ordered and the shared surface is named in each body.
- **Source model, IR and CLI:** the evidence-freshness and conformance-suite/semantic-diff pairs are ordered.
- **Crate layout:** it is stated.
- **Scopes:** all stories now carry scopes.

**Placement:** 12 of 12 stories placed from cited scopes, 0 inferred, 0 unplaceable. The two decision-blockers are not stories and carry no scope.

**Could not establish:**
- Which files `story:protocol-source-model` creates for the five empty modules (`ir`, `eval`, `explain`, `diff`, `conform`). Its scope lists only `lib.rs`, `model/` and `validate/`. This is ordered, so it is safe, and I am not raising it as a finding.
- Whether later stories add dependencies that touch `crates/canon/Cargo.toml` and `Cargo.lock`, for example a time parser for `--at` in `story:evidence-freshness`. Only `story:protocol-source-model` scopes them. This is ordered, so it is safe, and it is inferred.
- Out of my lane: whether `story:conformance-runner` can run before `conformance/scenarios/` has any file, and whether ownership of `canon-case/1` and `canon-evidence/1` is split cleanly between `story:three-valued-claims` and `story:evidence-revision-binding`.

```findings
- file: .engineering/planning/story/conformance-suite.md
  line: 58
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the Surfaces line omits crates/canon-cli/, which story:semantic-diff says both stories change (semantic-diff.md:34), so waves records no collision between them and the ordering rests on the depends_on edge alone; add crates/canon-cli/ to this scope or correct the reason in story:semantic-diff (cited, both bodies)
```
