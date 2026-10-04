---
format: aep.planning-md/3
id: review-result:canon-kernel-acceptance-r2
kind: review-result
status: active
title: Canon kernel decomposition — acceptance critic, round 2
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

story:canon-ir — the acceptance still joins two independent outcomes in one sentence: "`canon compile` emits byte-identical `canon-ir/1` for … and `fixtures/investigation/canon-ir.yaml`; and that IR contains no filesystem path". Round 1 raised this. The "two expectations" wording and named test do not separate them, and the acceptance does not say what input path `canon compile` is given, so the second clause is not checkable — .engineering/planning/story/canon-ir.md:41
story:conformance-runner — the acceptance names no all-passing scenario directory, so it never shows `canon conform run` exiting 0 on success, and a runner that always exits non-zero passes as written — .engineering/planning/story/conformance-runner.md:55
story:conformance-runner — the acceptance says "three expectations over a scenario directory of three files" but lists four outcomes, with the exit status and the run-to-run byte-identical output folded into the third, so the count does not match what is checked — .engineering/planning/story/conformance-runner.md:55
story:conformance-suite — the acceptance states only the pass condition over the real registry and names no case with an uncovered requirement or a non-reproducible scenario, so the "exits non-zero naming the requirement" half has no check and a runner that always exits 0 passes — .engineering/planning/story/conformance-suite.md:62
story:semantic-diff — the acceptance says "five expectations, one per revision pair" but lists six changes, because the added obligation and the shortened maximum age are two pairs sharing one `TIGHTENING` clause, so the number of pairs and expectations cannot both be right — .engineering/planning/story/semantic-diff.md:49
story:evidence-revision-binding — the Extends section adds a refusal ("A record whose subject is not a declared artifact is refused naming it"), but the acceptance has no case for it, so the new behaviour can pass unchecked — .engineering/planning/story/evidence-revision-binding.md:35

What I read: 15 artifacts (the epic, 12 stories, 2 decision-blockers), each in full with `aep plan artifact show`, plus `review-result:canon-kernel-acceptance-r1`. I also ran `aep plan artifact kinds` and `aep plan artifact lifecycle story`, and read design §§ 12, 31 and 32.

Round-1 findings by story:
- Fixed: `story:outcomes` (the undeclared-outcome refusal is now a third expectation) and `story:three-valued-claims` (the fourth expectation covers evidence that disagrees with itself).
- Fixed: `story:action-admissibility` (four expectations including the denied case, with the `--authority` input) and `story:obligations`. Both now name their own fixture variant.
- Not fixed: `story:canon-ir`, repeated above.
- Not raised: `story:explanation` still leaves part of its Outcome unchecked (the instant, expiry and non-claim items). I treated that as thin ambition, as in round 1.

What I could not establish:
- The decision-blockers still have no acceptance section; I did not flag this because a blocker is cleared by the recorded answer to its question.
- The named tests and the `conformance/` and `fixtures/` paths do not exist in the tree yet, so I could not run anything the acceptances name.
- `story:conformance-suite` leaves open what "permuted equivalent input" means for a compile step; I could not decide whether that belongs in the acceptance.
- Out of my lane: the `depends_on` chain, the shared `eval/` and `canon-decision/1` surfaces, and whether the two decision-blockers leave the epic's coverage of C-007 and C-008 complete.

```findings
- file: .engineering/planning/story/canon-ir.md
  line: 41
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance still joins two independent outcomes in one sentence, byte-identical IR for the two fixtures and the IR containing no filesystem path, and does not say what input path compile is given, so the second clause is not checkable
- file: .engineering/planning/story/conformance-runner.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance names no all-passing scenario directory, so it never shows canon conform run exiting 0 on success, and a runner that always exits non-zero passes as written
- file: .engineering/planning/story/conformance-runner.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance says three expectations over three files but lists four outcomes, with the exit status and run-to-run byte-identical output folded into the third, so the count does not match what is checked
- file: .engineering/planning/story/conformance-suite.md
  line: 62
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance states only the pass condition over the real registry and names no case with an uncovered requirement or a non-reproducible scenario, so the exits-non-zero-naming-the-requirement half has no check and a runner that always exits 0 passes
- file: .engineering/planning/story/semantic-diff.md
  line: 49
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance says five expectations, one per revision pair, but lists six changes, because the added obligation and the shortened maximum age are two pairs sharing one TIGHTENING clause, so the number of pairs and expectations cannot both be right
- file: .engineering/planning/story/evidence-revision-binding.md
  line: 35
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the Extends section adds a refusal for an evidence record whose subject is not a declared artifact, but the acceptance has no case for it, so the new behaviour can pass unchecked
```
