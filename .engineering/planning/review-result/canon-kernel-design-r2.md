---
format: aep.planning-md/3
id: review-result:canon-kernel-design-r2
kind: review-result
status: active
title: Canon kernel decomposition — design critic, round 2
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

story:evidence-freshness — it excludes expired evidence but its Extends list does not say it records those exclusions in the `claims` entries of `canon-decision/1`, so the exclusion listing is half in evidence-revision-binding (mismatch ids) and half in story:explanation (names "expiry" as a reason), and nobody owns the expiry half; the body should state that it extends the same excluded-evidence list with an expiry reason — .engineering/planning/story/evidence-freshness.md:41 (cf. .engineering/planning/story/evidence-revision-binding.md:38, .engineering/planning/story/explanation.md:33)

story:conformance-suite — it adds coverage and permutation checks to `canon conform run`, the command story:conformance-runner defines and tests over a three-file directory with no catalogue, without saying the checks are additive (for example applied only when `conformance/requirements.yaml` is supplied), so the runner's acceptance can break once the suite lands — .engineering/planning/story/conformance-suite.md:35 (cf. .engineering/planning/story/conformance-runner.md:59)

**What I read:** 21 artifacts. I ran `aep plan artifact list`, `relations`, `graph`, `validate` (valid) and `blocked`, and `show` on the 12 stories, the epic, both decision-blockers and `review-result:canon-kernel-design-r1`. I walked every depends_on edge in the graph (about 40 depends_on, plus decomposes, blocks, serves and reviews) and followed them outside the set to the epic and visions. There is no cycle. The order is psm → ir → runner → three-valued → evidence-revision-binding → obligations → action-admissibility → outcomes → freshness → explanation → suite → diff.

All four round-1 design findings are resolved:
- The input and output document shapes are now owned by story:three-valued-claims.
- story:explanation now extends `canon-decision/1` rather than introducing it.
- The runner is split out as story:conformance-runner.
- The `discharged_when` field is placed in story:obligations.

**What I could not establish:**
- The evaluator chain is fully serial (three-valued → … → explanation). I did not raise it, because every link states its reason: the shared surface is `crates/canon/src/eval/` and `canon-decision/1`. The trade-off is the recorded order against splitting `eval/` into one module and one decision section per story. That is the drafter's call.
- Out of my lane (parallel safety): story:semantic-diff justifies its edge to story:conformance-suite by saying both change `crates/canon-cli`. story:conformance-suite lists no `crates/canon-cli` surface (.engineering/planning/story/conformance-suite.md:58), so either the surface or the stated reason is wrong.
- Out of my lane (scope): `epic:canon-kernel` still says it covers TASKBOARD C-001 … C-011 whole, although the two decision-blockers defer the decision-based-outcome half of C-007 and the upstream-invalidation half of C-008.

```findings
[
  {"file": ".engineering/planning/story/evidence-freshness.md", "line": 41, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "story:evidence-freshness — it excludes expired evidence but its Extends list does not say it records those exclusions in the `claims` entries of `canon-decision/1`, so the exclusion listing is half in evidence-revision-binding (mismatch ids) and half in story:explanation (names \"expiry\" as a reason), and nobody owns the expiry half; the body should state that it extends the same excluded-evidence list with an expiry reason"},
  {"file": ".engineering/planning/story/conformance-suite.md", "line": 35, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "story:conformance-suite — it adds coverage and permutation checks to `canon conform run`, the command story:conformance-runner defines and tests over a three-file directory with no catalogue, without saying the checks are additive (for example applied only when `conformance/requirements.yaml` is supplied), so the runner's acceptance can break once the suite lands"}
]
```
