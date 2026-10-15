---
format: aep.planning-md/3
id: review-result:conformance-suite-design-r1
kind: review-result
status: active
title: Conformance suite, design critic, round 1
relations:
- reviews: story:conformance-suite
revision: 1
---
needs-revision

- story:conformance-suite — the story puts the Rust catalogue model (`Requirements`, `Requirement`, `RequirementStatus`) in `crates/canon/src/conform/`, but `ess_model_matches.rs` reads only `crates/canon/src/model/`, so the model the story says is "held equal" to `canon.conform` is never compared. The scope also omits the other two edits the test needs. `ess/ess-inputs.yaml` must list `domains/conform.yaml`, because `authored_map_keys` reads map keys only from the files that list names. `DOCUMENTS` must name `Requirements`, because the types the test compares are those an entity reaches plus those `DOCUMENTS` reaches, and no entity holds the catalogue. The body should place the model type in `crates/canon/src/model/` (for example a new file) or state a different test change, and add `ess/ess-inputs.yaml` and the `DOCUMENTS` entry to its scope. As written, the ESS half and the Rust half of one abstraction sit where the test cannot join them. — .engineering/planning/story/conformance-suite.md:113-116, .engineering/planning/story/conformance-suite.md:140-142, crates/canon/tests/ess_model_matches.rs:366, crates/canon/tests/ess_model_matches.rs:82-89, crates/canon/tests/ess_model_matches.rs:196-203, ess/ess-inputs.yaml:4-7

What I read: 1 artifact, `story:conformance-suite`, the only unit of its wave.
- Commands: `aep plan artifact show`, `relations`, `graph` and `validate`.
- Files read: `ess/system.yaml`, `ess/ess-inputs.yaml`, `ess/domains/check.yaml`, `crates/canon/tests/ess_model_matches.rs`, `crates/canon/src/conform/mod.rs`, `conformance/scenarios/*.yaml`.
- Validator: `aep plan artifact validate` reports the store valid. It also notes that two review results, `review-result:adversary-action-arguments-pass-1` and `review-result:adversary-evidence-independence-pass-1`, have no findings block. Neither concerns this story.

What I could not establish:
- The store prints revision 10, not the 9 named in the request; I judged revision 10.
- Walked the full `graph` output (108 artifacts, including edges outside the set). The story has ten `depends_on` edges, all to stories with status implemented, plus one `decomposes` and two `serves`. I found no cycle and no chain.
- Traced the acceptance reads to producers. The real registry and the 15 requirement ids match: every id the story lists is covered by a `conformance/scenarios/` file. The fixtures and `conformance/requirements.yaml` are created by this same story. So no read depends on a sibling, and none is missing an edge.
- Whether reversing `evidence` leaves real decisions unchanged was not run. Order predicates key on `observed_at` rather than list position (`story:evidence-order-predicate`), so I did not treat it as a defect. Whether the acceptance can be checked is `plan-critic-acceptance`'s lane, and parallel safety belongs to its own critic.
- The finding above could also be called a scope gap. I filed it as a design seam because it splits one abstraction between the ESS domain and its Rust model.

```findings
- file: .engineering/planning/story/conformance-suite.md
  line: 113
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the story puts the Rust catalogue model in crates/canon/src/conform/ but ess_model_matches.rs reads only crates/canon/src/model/, so canon.conform is never held equal to its model; the scope also omits ess/ess-inputs.yaml (must list domains/conform.yaml) and the DOCUMENTS entry for Requirements, so the model belongs in crates/canon/src/model/ and those two edits belong in scope"
```
