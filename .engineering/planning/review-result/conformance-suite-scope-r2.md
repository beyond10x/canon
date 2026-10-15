---
format: aep.planning-md/3
id: review-result:conformance-suite-scope-r2
kind: review-result
status: active
title: Conformance suite, scope critic, round 2
relations:
- reviews: story:conformance-suite
revision: 1
---
approve

I read the epic's promises before the story and found no gap, no reach beyond the parent, no overlap and no narrowing. Revision 12 is what the store holds, not the 11 you named. I judged 12, the version in the store's working tree. It is uncommitted over revision 8.

**What I read:** 8 artifacts. They are `epic:canon-kernel`, `story:conformance-suite`, `review-result:conformance-suite-scope-r1`, and the ess-command-surface, protocol-imports, capability-scope, revision-ordering and case-composition stories. I ran `aep plan artifact show` on the epic, the story and the round-1 review, plus `aep plan artifact graph`, `git diff` on the story file, and `grep` on the siblings. I read design § 32 (lines 1414-1458) and the TASKBOARD line `- [ ] C-010 Create core conformance suite.`. I listed `conformance/scenarios/` and counted its `covers:` ids.

**Promises and tracing:** I extracted 7 promises from the epic and traced all 7 to an item. The other six promises have no `path:line`, because they are the epic's own outcome and acceptance sentences.
- **C-010 as one of C-001 … C-011:** this story, with `refs: taskboard:C-010`.
- **Conformance suite runs a non-engineering investigation fixture:** the real registry uses `protocol: investigation` in 44 places. The suite checks coverage over it.
- **Byte-identical normalized evaluation:** the determinism check on every evaluate step, and CANON-DETERMINISM-001 in the catalogue.
- **Superseded-revision evidence gives `UNKNOWN`:** CANON-EVIDENCE-001 is an implemented, covered catalogue entry.
- **An action whose required claim is not `TRUE` is blocked with its reason:** CANON-AUTHORITY-001 is an implemented, covered catalogue entry.
- **No engineering vocabulary in the kernel:** unchanged by this story, which adds only a requirement catalogue and checks.
- **The remaining C-001 … C-011 items:** claimed by sibling stories in the graph.

**Design § 32 coverage:**
- The catalogue's 15 ids match the 15 distinct `covers:` ids in the registry.
- All nine § 32 ids are in it, including CANON-INDEPENDENCE-001 and CANON-INVALIDATION-001. Revision 8 had left those two out, and they are now in.
- The six non-§-32 ids are named by `source` as coming from the introducing stories. They are catalogue entries, not new work.
- The `canon conform synthesize` command (§ 29) and the runner are in Scope Out. The additive-to-the-runner rule is cited to the 2026-10-04 operator decision.
- The extra scope paths follow from the CLI flag and the ESS-first rule in AGENTS.md, so they are not reach.

**What I could not establish:**
- **Out of my lane (design or parallel-safety):** the story's Order section says capability-scope, revision-ordering, case-composition and protocol-imports each add their requirement to `conformance/requirements.yaml` in their own first commit. Those four bodies only gained a `depends_on` edge. Their scope lists and bodies do not name the catalogue, so nothing in them says they will do it.
- **Out of my lane (design):** the story's `depends_on` edges omit the stories behind CANON-INDEPENDENCE-001, CANON-INPUT-001, CANON-ORDER-001 and CANON-EVIDENCE-003. This is the same point round 1 raised.
- **Not established:** I did not re-check whether round 1's other premises still hold. They are the 16-promise count and the `conformance-runner` split, which I took from its review.

```findings
[]
```
