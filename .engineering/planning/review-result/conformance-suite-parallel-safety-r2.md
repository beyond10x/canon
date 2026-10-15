---
format: aep.planning-md/3
id: review-result:conformance-suite-parallel-safety-r2
kind: review-result
status: active
title: Conformance suite, parallel-safety critic, round 2
relations:
- reviews: story:conformance-suite
revision: 1
---
needs-revision

ess-command-surface — its Scope still says "Shared files: `ess/system.yaml` with any other story that adds a domain; none in the 2026-10-07 order does", but story:conformance-suite adds the `canon.conform` domain to that file. The suite's own body says this claim "no longer holds". The `depends_on` edge now orders the pair, but this body still states the opposite. Cited on both sides. The body should name story:conformance-suite as the one that shares `ess/system.yaml` and `conform run`, and say the edge orders them. — .engineering/planning/story/ess-command-surface.md:60, .engineering/planning/story/conformance-suite.md:177-178

protocol-imports — its Scope says `crates/canon/src/conform/` is shared with story:conformance-suite and that "Neither has an edge to this story". Its own relations now carry `depends_on story:conformance-suite`, so the sentence is false for the suite. It also omits the suite from the `lib.rs`, `main.rs` and `website/` shares it lists with story:canon-mutate. The body should say the suite pair is ordered by the edge, which is the file-sharing reason. Cited on both sides. — .engineering/planning/story/protocol-imports.md:105-107, .engineering/planning/story/protocol-imports.md:18

**What I read:** the suite's current body, which the store shows at revision 12 (you said 11). I also read the r1 parallel-safety record and the other 8 not-yet-implemented stories: canon-mutate, capability-scope, case-composition, ess-command-surface, invalidation-evaluation-cost, protocol-floor, protocol-imports and revision-ordering. Commands: `aep plan artifact list`, `waves`, `graph`, `validate`, and `show` on each. I also ran `git grep` on `~/beyond10x/canon` at main (983152d) for the scenario `covers:` ids, `main.rs`, `lib.rs` and the Taskfile.

**Round-1 findings:**
- **Order line and protocol-imports:** fixed. The Order section now names the pair, and the edge exists.
- **`ess/system.yaml` with ess-command-surface:** fixed. The edge exists and the body names the pair.
- **`main.rs` omitted from scope:** fixed. `main.rs` and `lib.rs` are in scope, and canon-mutate, protocol-floor and protocol-imports all carry edges.
- **Scenario-registry readers:** fixed. capability-scope, revision-ordering, case-composition and protocol-imports all carry edges. The suite's count of 15 matches the `covers:` ids in `conformance/scenarios/` at main (the 16th id, CANON-REASONS-001, is a scenario id, not a covered requirement).

Waves puts the suite alone in wave 13 with every dependent after it, and `validate` reports valid.

**Surfaces placed (8 other not-yet-implemented stories):**
- **Cited:** 3 (canon-mutate, capability-scope, revision-ordering).
- **Inferred:** 4 (case-composition, ess-command-surface, protocol-floor, protocol-imports). All four have a mix of cited and inferred scope entries or inferred scope, and the collisions with the suite on them are marked inferred.
- **Unplaceable:** 0. invalidation-evaluation-cost has no scope, but its body cites `eval/claims.rs` and `eval/invalidation.rs`, which do not overlap the suite.
- **Acceptance reads traced to a producer in the set:** 1. The suite reads the real registry, whose producers are all implemented stories outside the set. The other reads (from the dependents) run toward the suite, and all 7 dependents have the `depends_on` edge.

**What I could not establish:**
- The four stories that add a scenario (capability-scope, revision-ordering, case-composition, protocol-imports) have no `conformance/requirements.yaml` in their scope. The suite's Order section says each will add its requirement there. Those four will collide with each other on the file, not with the suite. This is outside the suite-versus-others question, so it is not a finding. It is worth a line in each of those bodies.
- Out of my lane: the Taskfile has no conform entry today, so the "`task check` passes the flag" line is acceptance-critic territory.

```findings
- file: .engineering/planning/story/ess-command-surface.md
  line: 60
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Scope states that no other story adds a domain to ess/system.yaml, but story:conformance-suite adds canon.conform there and declares --requirements on conform run (conformance-suite.md:177-178); the body should name the suite as the shared-file pair now ordered by its depends_on edge"
- file: .engineering/planning/story/protocol-imports.md
  line: 107
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Scope says crates/canon/src/conform/ is shared with story:conformance-suite and that neither has an edge, but this story now depends_on story:conformance-suite (line 18), and the lib.rs, main.rs and website/ shares with the suite are not named; the body should record the suite pair as ordered by the edge"
```
