---
format: aep.planning-md/3
id: review-result:issue5-acceptance-r1
kind: review-result
status: active
title: Issue 5 stories, acceptance critic, round 1
relations:
- reviews: story:protocol-imports
- reviews: story:protocol-floor
- reviews: story:semantic-diff
- reviews: story:case-composition
revision: 1
---
needs-revision
story:protocol-imports — the acceptance exercises only `canon compile`, so `--import` on `validate`, `check` and `diff` (and the `generate` refusal naming the first import) could be absent and both named checks would still pass — .engineering/planning/story/protocol-imports.md:62 vs :116-129
story:protocol-imports — the acceptance is two independent checks (scenario `CANON-IMPORT-001`, and test `compile_refuses_unclosed_imports` plus `ess_model_matches.rs`), so one can pass while the other fails and the story is neither done nor not done — .engineering/planning/story/protocol-imports.md:116-129
story:protocol-floor — the Outcome promises that `canon floor` with the import as the floor checks a refinement only tightens, but no expectation uses a refinement (expectation 2 is an import plus an added action, not a `refines` redeclaration), so that use is untested — .engineering/planning/story/protocol-floor.md:53 vs :88-101

**What I read:** 4 of 4 ids (`story:protocol-imports`, `story:protocol-floor`, `story:semantic-diff`, `story:case-composition`) through `aep plan artifact show`. I also read `aep plan artifact kinds` and `aep plan artifact lifecycle story`, GitHub issue 5, the critic rubric, `conformance/scenarios/invalidation-rules.yaml`, `crates/canon/src/conform/mod.rs`, and `git grep` for `CANON-IMPORT`, `imports` and `64`. I also checked the status of the four dependency stories.
- `story:semantic-diff` and `story:case-composition`: the drafted ordering sections change no acceptance, and each acceptance is still one checkable statement. No finding.
- No clash: no `CANON-IMPORT-*` scenario exists yet. The "exit 64" in `story:protocol-floor` Red matches `USAGE: u8 = 64` in `crates/canon-cli/src/lib.rs:18`.

**What I could not establish:**
- `CANON-IMPORT-001` expects "equals byte for byte the compile of `flattened.yaml`". The harness (`crates/canon/src/conform/mod.rs:148`) compares a step against literal IR bytes from one `fixture`, so I could not tell how the scenario ties to `flattened.yaml`'s compile. If the literal is hand-pinned, the equality is not checked. This is a wording or mechanism question and I did not make it a finding.
- Out of my lane: the claim that `story:case-composition` and `story:protocol-floor` may share a wave, and the shared-surface reasoning in the Order sections (parallel-safety and design critics).

```findings
- file: .engineering/planning/story/protocol-imports.md
  line: 116
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance exercises only `canon compile`, so `--import` on `validate`, `check` and `diff` and the `generate` refusal naming the first import (Outcome 2, Scope) are unobserved and could be absent with every named check passing"
- file: .engineering/planning/story/protocol-imports.md
  line: 125
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins two independent checks, scenario CANON-IMPORT-001 and the test compile_refuses_unclosed_imports plus ess_model_matches.rs, so one can pass while the other fails"
- file: .engineering/planning/story/protocol-floor.md
  line: 88
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome says the same command checks that a refinement only tightens what it imports, but none of the five expectations uses a `refines` redeclaration, so that use is untested"
```
