---
format: aep.planning-md/3
id: review-result:issue5-design-r1
kind: review-result
status: active
title: Issue 5 stories, design critic, round 1
relations:
- reviews: story:protocol-imports
- reviews: story:protocol-floor
- reviews: story:semantic-diff
- reviews: story:case-composition
revision: 1
---
needs-revision

story:protocol-floor — expectation 5 and the "declarations only" claim read how story:semantic-diff's classifier treats a changed protocol id, and story:semantic-diff states no such rule: it covers revision pairs and a description-only edit, with no differing ids. The floor body must say where id, revision and description are neutralised before classification (in the floor module, or as a stated classifier rule), or story:semantic-diff must state the rule. — .engineering/planning/story/protocol-floor.md:48, .engineering/planning/story/protocol-floor.md:100, .engineering/planning/story/semantic-diff.md:71-75
story:protocol-floor — its ESS-first section says `canon floor`'s command surface "is declared in `ess/` by story:ess-command-surface, as `canon diff` is". That story's outcome lists only `validate`, `compile`, `evaluate`, `conform run` and `diff`, and no edge runs between it and the floor. The body must either say the floor's declaration is out of that story's list and who adds it, or record the order with a `depends_on` edge between the two. — .engineering/planning/story/protocol-floor.md:81, .engineering/planning/story/ess-command-surface.md:20

What I read: 5 artifacts in full (story:protocol-imports, story:protocol-floor, story:semantic-diff, story:case-composition, story:ess-command-surface), plus issue 5, ADR 0086, design § 39.5 and `aep plan artifact relations`, `graph`, `waves` and `validate` (valid, 89 artifacts).

I walked every `depends_on` and `decomposes` edge in the store (about 100, including all outside the set) and found no cycle. The set is not a serialising chain: each new edge has its reason written beside it (shared `diff.rs`, shared `ess/model/validate/ir`, or an acceptance that reads the producer).

I traced 5 acceptance reads to a producer in the set, and an edge records all 5:
- Floor expectations 1, 3 and 4 read the classifier of story:semantic-diff.
- Floor expectation 2 reads `--import` from story:protocol-imports.
- The imports and case-composition acceptances read nothing a sibling produces.

What I could not establish:
- Out of my lane, parallel safety. story:protocol-imports says "All three of story:protocol-imports, story:case-composition and every other story" share `ess/` and `model/`, but it names two, and story:protocol-floor changes no `ess/` or `model/`. The same body says "this one is unblocked and goes first" while it depends on story:semantic-diff. `waves` reports no collision between story:protocol-floor and story:case-composition, which supports the claim that they can share a wave. The floor's `website/docs/reference/` regeneration and case-composition's ESS change both touch the generated reference, so check that claim.
- Out of my lane, acceptance. story:protocol-floor says `canon floor` checks that a refinement only tightens, but none of its five expectations covers a refinement. A refined outcome is always `BREAKING` under story:semantic-diff, so it can never pass.

```findings
- file: .engineering/planning/story/protocol-floor.md
  line: 48
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "expectation 5 and the 'declarations only' claim read how story:semantic-diff's classifier treats a changed protocol id, and story:semantic-diff states no such rule: it covers revision pairs and a description-only edit, with no differing ids. The floor body must say where id, revision and description are neutralised before classification (in the floor module, or as a stated classifier rule), or story:semantic-diff must state the rule."
- file: .engineering/planning/story/protocol-floor.md
  line: 81
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its ESS-first section says `canon floor`'s command surface 'is declared in `ess/` by story:ess-command-surface, as `canon diff` is'. That story's outcome lists only validate, compile, evaluate, conform run and diff, and no edge runs between it and the floor. The body must either say the floor's declaration is out of that story's list and who adds it, or record the order with a depends_on edge between the two."
```
