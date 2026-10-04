---
format: aep.planning-md/3
id: review-result:canon-kernel-design-r1
kind: review-result
status: active
title: Canon kernel decomposition — design critic, round 1
relations:
- reviews: epic:canon-kernel
- reviews: story:action-admissibility
- reviews: story:canon-ir
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

story:three-valued-claims — `canon evaluate --case --evidence` takes two input documents whose shape this body never states, while story:evidence-revision-binding defines the evidence record and the case snapshot's artifact revisions as its own in-scope work and story:outcomes adds a termination field to the case snapshot, so the input records are defined piecemeal with no owner; the body should state the minimal case snapshot and evidence record (kind, result) and the later stories should say they extend them — .engineering/planning/story/evidence-revision-binding.md:25
story:explanation — it introduces `canon-decision/1` as what `canon evaluate` emits, yet story:three-valued-claims, story:action-admissibility and story:outcomes each already acceptance-test a `canon evaluate` report with reasons, so the output document is half-defined in five stories and replaced in the last; the body should say it extends the document story:three-valued-claims first emits, or story:three-valued-claims should define the envelope — .engineering/planning/story/explanation.md:21
story:conformance-suite — it owns the scenario format and `canon conform run`, but the acceptances of story:three-valued-claims, story:evidence-revision-binding, story:evidence-freshness, story:obligations, story:action-admissibility and story:outcomes all require "a conformance scenario" and the edges run only from the suite to them; either split the scenario format and runner out as the first item those stories depend on, or the body should say earlier stories write plain tests that this story adopts — .engineering/planning/story/conformance-suite.md:22
story:obligations — its outcome presumes an obligation "carries a discharge predicate", but story:protocol-source-model lists "obligations" without one and this body does not say it adds the field to the source model and IR, as story:evidence-freshness does for its maximum age; the body should state where the discharge predicate lands — .engineering/planning/story/obligations.md:18

What I read: 16 artifacts (11 stories, the epic, 2 decision-blockers, 2 visions), by `aep plan artifact list`, `show` on each of the 14 non-vision artifacts, `relations`, `graph`, `validate` (valid) and `blocked`. I walked all 30 edges in the graph (15 depends_on, 11 decomposes, 2 blocks, 2 serves), including those to the epic and visions outside the 11-story set. There is no cycle. The depends_on edges fan out from story:three-valued-claims, so they do not form a queue. I checked the sources at docs/design/canon-protocol-calculus-design.md §§ 12, 14, 29–32, 39, 41, docs/contracts/protocol-core.md, AGENTS.md and crates/canon/src/lib.rs.

What I could not establish:
- The edge from story:semantic-diff to story:evidence-freshness is needed, because that acceptance names a shortened maximum age. I did not raise it.
- Out of my lane (scope): design § 41 item 10 (frontier calculation) is cited by story:action-admissibility but appears in no outcome or acceptance. The epic says it covers C-007 and C-008 whole, yet the deferred halves (decision-based outcomes, upstream invalidation) have no story. Both blockers attach to the epic rather than to the stories they bound.
- Out of my lane (parallel safety): stories obligations, action-admissibility, outcomes and evidence-freshness all extend `canon evaluate` and `crates/canon/src/lib.rs`. I did not judge whether they collide.

```findings
[
  {"file": ".engineering/planning/story/evidence-revision-binding.md", "line": 25, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "story:three-valued-claims — `canon evaluate --case --evidence` takes two input documents whose shape this body never states, while story:evidence-revision-binding defines the evidence record and the case snapshot's artifact revisions as its own in-scope work and story:outcomes adds a termination field to the case snapshot, so the input records are defined piecemeal with no owner; the body should state the minimal case snapshot and evidence record (kind, result) and the later stories should say they extend them"},
  {"file": ".engineering/planning/story/explanation.md", "line": 21, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "story:explanation — it introduces `canon-decision/1` as what `canon evaluate` emits, yet story:three-valued-claims, story:action-admissibility and story:outcomes each already acceptance-test a `canon evaluate` report with reasons, so the output document is half-defined in five stories and replaced in the last; the body should say it extends the document story:three-valued-claims first emits, or story:three-valued-claims should define the envelope"},
  {"file": ".engineering/planning/story/conformance-suite.md", "line": 22, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "story:conformance-suite — it owns the scenario format and `canon conform run`, but the acceptances of story:three-valued-claims, story:evidence-revision-binding, story:evidence-freshness, story:obligations, story:action-admissibility and story:outcomes all require \"a conformance scenario\" and the edges run only from the suite to them; either split the scenario format and runner out as the first item those stories depend on, or the body should say earlier stories write plain tests that this story adopts"},
  {"file": ".engineering/planning/story/obligations.md", "line": 18, "category": "design", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "story:obligations — its outcome presumes an obligation \"carries a discharge predicate\", but story:protocol-source-model lists \"obligations\" without one and this body does not say it adds the field to the source model and IR, as story:evidence-freshness does for its maximum age; the body should state where the discharge predicate lands"}
]
```
