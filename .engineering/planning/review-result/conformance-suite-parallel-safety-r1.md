---
format: aep.planning-md/3
id: review-result:conformance-suite-parallel-safety-r1
kind: review-result
status: active
title: Conformance suite, parallel-safety critic, round 1
relations:
- reviews: story:conformance-suite
revision: 1
---
needs-revision

story:conformance-suite — its Order line says "No other ready unit shares a surface", but story:protocol-imports names `crates/canon/src/conform/` and `crates/canon-cli/src/lib.rs` as shared with it, and `waves` flags `website/docs/reference/` and `website/status.yaml` as colliding with it too (cited by protocol-imports, inferred on the suite's side), and no edge orders the pair; the body must name the pair and either add an ordering edge recording those files as its reason or split the surface so the two no longer share them — .engineering/planning/story/conformance-suite.md:153, .engineering/planning/story/protocol-imports.md:104

story:conformance-suite — it adds a domain to `ess/system.yaml` and declares the `--requirements` flag on `conform run`, while story:ess-command-surface (wave 14, no edge to this story) also edits `ess/system.yaml`, declares `conform run` in `ess/`, and states that no other story adds a domain, so the body must name that pair and either order it with an edge recording the shared file or split the surface (cited both bodies; `ess-command-surface` scope is inferred, and `cli.yaml` does not exist yet) — .engineering/planning/story/conformance-suite.md:113, .engineering/planning/story/ess-command-surface.md:59

story:conformance-suite — its scope omits `crates/canon-cli/src/main.rs`, though adding the flag means changing the `ConformCommand::Run { scenarios } => conform::run(&scenarios)` arm there. story:canon-mutate (same wave 13), story:protocol-imports and story:protocol-floor all list `main.rs`, so the body must add the file and either order the pair with an edge or split the surface (the suite's side is inferred from the tree, the others are cited) — ~/beyond10x/canon/crates/canon-cli/src/main.rs:38-40, .engineering/planning/story/canon-mutate.md:48

story:conformance-suite — its acceptance reads the scenario registry `conformance/scenarios/` and expects exactly `requirements: 15 implemented, 15 covered, 0 planned`. story:capability-scope (wave 13, the same wave), story:revision-ordering, story:case-composition and story:protocol-imports each create a scenario with a new id (`CANON-SCOPE-001`, `CANON-ORDER-002`, `CANON-COMPOSE-001`, `CANON-IMPORT-001`). None of them mentions `conformance/requirements.yaml`, and no `depends_on` edge runs from this story to any of them. If one lands first, the suite's `unlisted` check and its count of 15 fail. If the suite lands first, that story's scenario fails `unlisted` under `task check`. The missing edges, or merging, are what the finding calls for — .engineering/planning/story/conformance-suite.md:129, .engineering/planning/story/capability-scope.md:90

What I read: 9 stories, the 8 not-yet-implemented drafts plus story:conformance-suite, from `aep plan artifact list`, `aep plan artifact waves` and `aep plan artifact show` on each draft. I also checked the tree with `git grep` (`main.rs`, `lib.rs`, `conformance/scenarios`). The store shows conformance-suite at revision 10, not 9, and I judged revision 10.

Surfaces placed:
- **Cited:** 6 (canon-mutate, capability-scope, case-composition, revision-ordering, protocol-floor, protocol-imports). The last two are cited in their Surfaces sections but marked inferred in their scope lists.
- **Inferred:** 2 (ess-command-surface, invalidation-evaluation-cost).
- **Unplaceable:** 0. story:invalidation-evaluation-cost has no scope, but its body cites `eval/claims.rs` and `eval/invalidation.rs`, which do not overlap the suite.
- **Acceptance reads traced to a producer in the set:** 4 (the registry scenarios from capability-scope, revision-ordering, case-composition and protocol-imports). `depends_on` records 0 of them.

What I could not establish:
- Whether story:canon-mutate's calls into the conformance runner survive the suite's changes to `crates/canon/src/conform/`. The body says the checks are additive, but the library signature is not stated.
- Whether story:capability-scope's whole-directory `ess/` claim touches `ess/system.yaml` or `crates/canon/tests/ess_model_matches.rs`. I did not count it as a collision.
- `CHANGELOG.md` and `website/` are shared with most stories. I treated them as a norm and not a finding.
- Out of my lane: the Taskfile has no `conform` or `requirements` entry on main today, so the "`task check` passes the flag" line in the suite's scope is a change to a file I did not inspect further (acceptance critic).

```findings
- file: .engineering/planning/story/conformance-suite.md
  line: 153
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Order line says no other ready unit shares a surface, but story:protocol-imports names crates/canon/src/conform/ and crates/canon-cli/src/lib.rs as shared with it, and waves also flags website/docs/reference/ and website/status.yaml against it (inferred on the suite's side), with no edge; the body must name the pair and either add an ordering edge recording the shared files or split the surface"
- file: .engineering/planning/story/conformance-suite.md
  line: 113
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "this story adds a domain to ess/system.yaml and declares the --requirements flag on conform run, while story:ess-command-surface (no edge) also edits ess/system.yaml and declares conform run, and states no other story adds a domain (ess-command-surface.md:59); name the pair and either order it with an edge recording the shared file or split the surface (ess-command-surface scope is inferred and cli.yaml does not exist yet)"
- file: ~/beyond10x/canon/crates/canon-cli/src/main.rs
  line: 38
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the scope omits crates/canon-cli/src/main.rs, where the conform run arm must change to pass the flag; story:canon-mutate (same wave 13), story:protocol-imports and story:protocol-floor all list main.rs; add the file and either order the pair with an edge or split the surface (the suite's surface is inferred from the tree)"
- file: .engineering/planning/story/conformance-suite.md
  line: 129
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance reads the scenario registry conformance/scenarios/ and expects exactly 15 implemented and covered, but story:capability-scope (same wave), story:revision-ordering, story:case-composition and story:protocol-imports create scenarios with new ids (CANON-SCOPE-001, CANON-ORDER-002, CANON-COMPOSE-001, CANON-IMPORT-001) and no depends_on edge orders them; add the missing edges (or merge), since landing either way makes the unlisted check or the count of 15 fail"
```
