---
format: aep.planning-md/3
id: review-result:issue5-design-r2
kind: review-result
status: active
title: Issue 5 stories, design critic, round 2
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

story:protocol-imports — its Outcome 2 and 3 name no rule for how an imported `case: inputs` section composes (inlined, redeclarable under `refines`, or refused when root and import both declare an input id), although the story is ordered after story:case-inputs and the downstream profile it serves needs the library protocol's inputs to be visible to the profile's `input` predicates; with `refines` limited to evidence kind, claim, obligation, action and outcome, the seam between the two stories is left undefined — .engineering/planning/story/protocol-imports.md:63-73 and :86-88, .engineering/planning/story/case-inputs.md:53-59

**What I read:** 9 artifacts. I ran `aep plan artifact show` on the three drafts and on story:semantic-diff, story:case-composition and story:ess-command-surface. I ran `aep plan artifact relations`, `graph` and `validate` (94 artifacts, valid). I read `review-result:issue5-{design,acceptance,scope,parallel-safety}-r1`, `aep plan artifact waves`, issue 5, `crates/canon-cli/src/lib.rs`, `crates/canon/src/model/mod.rs`, design § 39.5 and § 541/632, and the downstream story:software-change-profiles.
- I walked all 128 `depends_on`, `decomposes` and `blocks` edges in the store, including every edge to and from artifacts outside the set, with a script. There is no cycle.
- I traced 9 acceptance reads to a producer in the set, and an edge records all 9.
- Both round-1 design findings are fixed. The floor body now says where the header is dropped (protocol-floor.md Outcome), and the `depends_on` edge floor → ess-command-surface now exists.
- The set is not a serialising chain. Each ordering edge has its reason written beside it (the shared `ess/`, `model/`, `validate/` and `ir/`, or a read of the producer's output).

**What I could not establish:**
- Out of my lane (parallel safety): story:ess-command-surface has no scope, so `waves` lists it as "unassessed". Nothing orders it against story:case-inputs, and both edit `ess/` and both depend only on story:semantic-diff. `waves` cannot see that collision.
- Out of my lane (acceptance): story:semantic-diff says "eight expectations, one per revision pair in the directory". story:case-inputs adds two pairs to that directory and to the same named test, so the count is no longer eight once that story lands.
- Out of my lane (scope or acceptance): story:case-inputs Outcome 5 classifies only `applicable_when` gained or lost. It names no category for a widened `in` list on an `input` leaf, for a changed input `values` list, or for an action `precondition` change. The floor's verdict is only as complete as the classifier. story:semantic-diff's acceptance names no action-precondition case either.
- Out of my lane (scope): story:software-change-profiles also wants authority requirements to vary by risk. story:case-inputs derives obligations and preconditions only, so it has no `input` guard on an action's authority requirement.
- I did not check whether the same gap applies to the `artifacts` and `invalidation` sections of `protocol/1` under imports. I put it in the same finding as one seam rather than a second line.

```findings
- file: .engineering/planning/story/protocol-imports.md
  line: 63
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its Outcome 2 and 3 name no rule for how an imported `case: inputs` section composes (inlined, redeclarable under `refines`, or refused when root and import both declare an input id), although the story is ordered after story:case-inputs and the downstream profile it serves needs the library protocol's inputs visible to the profile's `input` predicates; with `refines` limited to evidence kind, claim, obligation, action and outcome, the seam between the two stories is left undefined"
```
