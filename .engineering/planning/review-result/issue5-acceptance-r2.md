---
format: aep.planning-md/3
id: review-result:issue5-acceptance-r2
kind: review-result
status: active
title: Issue 5 stories, acceptance critic, round 2
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

story:case-inputs — expectations 5 and 6 require refusals "naming `severity`" and "naming `severity` and `medium`", but a conformance evaluate step matches a refusal by its code alone, so a refusal naming neither the input nor the value still passes CANON-INPUT-001 — .engineering/planning/story/case-inputs.md:134-135 vs crates/canon/src/conform/mod.rs:111-114
story:case-inputs — no expectation exercises the validator refusals of Outcome 1 (empty `values`, a value listed twice), Outcome 3 (unknown input id, undeclared value in an `input` predicate) or Outcome 2's case input the protocol does not declare, so all of them could be absent with the seven expectations passing — .engineering/planning/story/case-inputs.md:54-64 vs :122-136
story:case-inputs — "`task check` exits 0 ... and its run includes `CANON-INPUT-001` passing under `canon conform run`" names no test, and `task check` runs no registry-wide conform step (only per-story tests invoke `canon conform run`), so `task check` can exit 0 with the scenario never run — .engineering/planning/story/case-inputs.md:122 vs Taskfile.yml:11-17 and crates/canon-cli/tests/decision_outcomes.rs:200
story:protocol-imports — "The same run includes `CANON-IMPORT-001` passing under `canon conform run`" has the same defect: no named test runs the scenario, so `task check` exiting 0 does not show it passed — .engineering/planning/story/protocol-imports.md:151 vs Taskfile.yml:11-17

Round-1 findings:
- Fixed: the `--import` coverage finding on story:protocol-imports. The acceptance now covers `validate`, `check` and `generate`, and `diff` and `floor` take no `--import`.
- Fixed: the finding that the refinement check was untested in story:protocol-floor. Expectation 6 now covers it.
- Resolved as drafted: the finding that story:protocol-imports bundled independent checks. Both are now under one `task check exits 0` umbrella. That umbrella is what findings 3 and 4 above say does not run the scenario.

**What I read:** all 6 ids, in full, by `aep plan artifact show`: story:protocol-imports, story:protocol-floor, story:case-inputs, story:semantic-diff, story:case-composition and story:ess-command-surface. I also read review-result:issue5-acceptance-r1, the `aep plan artifact kinds` and `lifecycle story` output, `Taskfile.yml`, and `crates/canon/src/conform/mod.rs`. I grepped `crates/*/tests` for how scenarios are run, and read the `Command` enum in `crates/canon-cli/src/lib.rs`.

**What I could not establish:**
- `story:semantic-diff` Acceptance and `story:ess-command-surface` Outcome are clean (one statement each, checkable). The eight pairs listed in semantic-diff match its count. I did not read the GitHub issue or the ADR, since they are scope or design material.
- The case-inputs fixture description names no outcome and no claim-reading `applicable_when` obligation, yet expectations 1 and 7 need both. The fixture is written in the first commit, so I did not make it a finding.
- `story:semantic-diff` "ESS first" says "the six revision pairs" while its Acceptance, edited this round, says eight (.engineering/planning/story/semantic-diff.md:64). That section is outside the acceptance, so I raised no finding.
- Out of my lane: the wave and shared-surface claims in the Order sections, and `story:case-composition` Order and Scope.

```findings
- file: .engineering/planning/story/case-inputs.md
  line: 134
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "expectations 5 and 6 require refusals naming `severity` and `medium`, but a conformance evaluate step matches a refusal by its code alone (crates/canon/src/conform/mod.rs:111-114), so a refusal naming neither passes CANON-INPUT-001"
- file: .engineering/planning/story/case-inputs.md
  line: 122
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "no expectation exercises the validator refusals of Outcome 1 (empty values, a value listed twice), Outcome 3 (unknown input id, undeclared value in an input predicate) or Outcome 2's undeclared case input, so all could be absent with the seven expectations passing"
- file: .engineering/planning/story/case-inputs.md
  line: 122
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance says the `task check` run includes CANON-INPUT-001 passing under `canon conform run` but names no test, and `task check` runs no registry-wide conform step, so it can exit 0 with the scenario never run"
- file: .engineering/planning/story/protocol-imports.md
  line: 151
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance says the same run includes CANON-IMPORT-001 passing under `canon conform run` but names no test, and `task check` runs no registry-wide conform step, so it can exit 0 with the scenario never run"
```
