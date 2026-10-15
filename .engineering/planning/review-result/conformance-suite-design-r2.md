---
format: aep.planning-md/3
id: review-result:conformance-suite-design-r2
kind: review-result
status: active
title: Conformance suite, design critic, round 2
relations:
- reviews: story:conformance-suite
revision: 1
---
approve

I read the one-story set as a set, including its edges outside the set, and found nothing that would change what was drafted.

The round-1 finding is closed. The catalogue model now sits in `crates/canon/src/model/catalogue.rs`, which `crates/canon/tests/ess_model_matches.rs` reads. `ess/ess-inputs.yaml` and the `DOCUMENTS` entry are in scope, so the ESS domain and its Rust model can now be compared by the parity test.

**What I read:** 1 artifact (`story:conformance-suite`), plus `review-result:conformance-suite-design-r1`.
- Commands: `aep plan artifact show`, `relations`, `graph` and `validate` (store valid).
- Tree files: `crates/canon/src/conform/mod.rs`, `crates/canon/src/model/mod.rs`, `crates/canon/tests/ess_model_matches.rs`, `ess/ess-inputs.yaml`, `conformance/scenarios/*.yaml` and `crates/canon/src/eval/`.

**Cycles and chains:**
- I walked the whole graph output (402 edge lines), including edges outside the set.
- The story has ten `depends_on` edges, all to implemented stories, plus one `decomposes` and two `serves` edges.
- Seven draft stories depend on it (`canon-mutate`, `capability-scope`, `case-composition`, `ess-command-surface`, `protocol-floor`, `protocol-imports`, `revision-ordering`). None is reachable from the story's own dependencies, so there is no cycle.
- Its dependencies are all implemented, so nothing queues behind another unit.

**Acceptance reads:** I traced 5 reads, and none depends on a sibling in the set.
- The real registry and its 15 `covers` ids match the catalogue list exactly (14 scenario files, ids as in `conformance/scenarios/`).
- The catalogue, fixtures and test it reads are created by this story.
- Producers outside the set are all implemented stories, with edges recorded.

**Split abstraction:** The catalogue model, the coverage and determinism checks, and the CLI flag are one outcome in one story, with no seam between sibling stories. The ESS spec and the Rust model are in the same first commit.

**What I could not establish:**
- The store prints revision 12, not the 11 you named. I judged what it prints.
- I did not build or run the code. Whether reversing the `evidence`, `authority` and `decisions` lists really leaves every real registry decision byte-identical is unconfirmed. The `eval` code sorts by `observed_at` and by code point, so I saw no defect. That question belongs to the acceptance critic.
- The checks need each scenario's `covers` list and its decision bytes. `run_registry` returns only a `Report` and `run` returns only a `Verdict`. The story says the existing entry points are kept, so the new library entry points are an implementation matter. I left it out of the findings.

```findings
[]
```
