---
format: aep.planning-md/3
id: review-result:canon-kernel-scope-r1
kind: review-result
status: active
title: Canon kernel decomposition — scope critic, round 1
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
epic:canon-kernel — the summary promises "Typed ids" and no story claims them: `protocol-source-model` only says the bootstrap `ProtocolId`/`CaseId`/`ClaimId`/`ActionId` types are replaced, and the other ten stories do not mention typed identifiers either; `story:protocol-source-model` is the natural owner — .engineering/planning/epic/canon-kernel.md:7

```findings
- file: .engineering/planning/epic/canon-kernel.md
  line: 7
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the summary promises "Typed ids" (also ROADMAP.md Phase 1 "typed identifiers") and no story claims them; story:protocol-source-model, which replaces the bootstrap ProtocolId/CaseId/ClaimId/ActionId types, would most naturally take it
```

What I read: the parent epic, `epic:ga-canon-kernel`, all 11 stories and both decision-blockers, in full via `aep plan artifact show`. I also ran `aep plan artifact graph`, `kinds` and `relations`, and `aep plan artifact validate` (valid). In addition I read TASKBOARD § Canon, ROADMAP Phase 1, the design doc §§ 29, 32 and 41, and `crates/canon/src/lib.rs`. I extracted 14 promises from the parent. These are the 11 C-items, no engineering vocabulary, the three acceptance clauses (byte-identical, superseded revision gives `UNKNOWN`, blocked with reason), and typed ids. I traced 13 to a story; typed ids is the one that traces to nothing.

What I could not establish: none.

Out-of-lane unease, not setting the verdict:
- `story:action-admissibility` cites design § 41 items 8–10, but item 10 ("Frontier calculation", `canon frontier`) is not delivered or excluded in its body. The parent does not promise a frontier, so I did not call it a gap.
- The blocker edges attach to the epic rather than to `story:evidence-freshness` or `story:outcomes`. That is a design question for the design critic.

Everything else holds. The two decision-blockers name the narrowed halves of C-007 and C-008, and the stories' Scope sections point back at them, so those are honest omissions. I found no item that reaches beyond the parent. I found no two stories claiming the same outcome.
