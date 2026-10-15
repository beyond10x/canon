---
format: aep.planning-md/3
id: review-result:conformance-suite-scope-r1
kind: review-result
status: active
title: Conformance suite, scope critic, round 1
relations:
- reviews: story:conformance-suite
revision: 1
---
approve

I found no scope defect. The story claims C-010 (the conformance suite) once and reaches nothing the parent or design § 32 did not ask for. I checked revision 10, the version in the store's working tree. The `git diff` shows it is uncommitted over revision 8.

**What I read**
- 4 artifacts: `epic:canon-kernel`, `story:conformance-suite`, `story:conformance-runner`, and the `canon-kernel-design-r1` and `canon-kernel-scope-r2` review results.
- Other commands: `aep plan artifact show` on those, `aep plan artifact graph`, `git diff` and `git log` on the story file, and the TASKBOARD line for C-010 in the atlas repo.
- Design § 28, § 29, § 32 and § 41 in `docs/design/canon-protocol-calculus-design.md`, and `conformance/scenarios/*.yaml`.

**Promises extracted from the epic.** I extracted 16 promises and traced all 16 to an item in the graph.
- 6 are this story's: the conformance outcome, C-010 inside "C-001 … C-011", byte-identical evaluation, a superseded revision giving `UNKNOWN`, a blocked action reported with its reason, and the non-engineering investigation fixture. Each traces to this story.
- The byte-identical, superseded-revision and blocked-action clauses trace to the determinism check and the catalogue entries CANON-DETERMINISM-001, CANON-EVIDENCE-001 and CANON-AUTHORITY-001. The registry's `protocol: investigation` fixtures are the non-engineering fixture.
- The other 10 are claimed by sibling stories in the graph, not by this one.

**Coverage and reach**
- All nine § 32 requirement ids are in the 15-entry catalogue, each with a registry scenario. The registry has 15 distinct covered ids, matching the catalogue.
- The six non-§-32 ids (EVIDENCE-003, OUTCOME-002, INPUT-001, ORDER-001, EXPLAIN-001, OBLIGATION-001) come from stories that decompose this epic. They are catalogue entries, not new work.
- The § 29 `canon conform synthesize` command is excluded in the story's Scope Out. So are the scenario format and runner, which belong to `story:conformance-runner`. The operator decision of 2026-10-04 records that split and the "additive when `--requirements` is given" rule.
- The extra scope paths (ESS domain, docs generation, `website/status.yaml`, `CHANGELOG.md`) follow from the CLI flag and the ESS-first rule in AGENTS.md.

**What I could not establish**
- You asked for revision 9, but the store shows revision 10, so I judged revision 10.
- The story's depends_on edges omit `story:evidence-independence`, `story:case-inputs`, `story:evidence-order-predicate` and `story:subject-bound-evidence-match`. Its catalogue lists their requirement ids: CANON-INDEPENDENCE-001, CANON-INPUT-001, CANON-ORDER-001 and CANON-EVIDENCE-003. All four stories are implemented, so this is harmless today. It belongs to the design critic, not the scope critic, and does not affect my verdict.

```findings
[]
```
