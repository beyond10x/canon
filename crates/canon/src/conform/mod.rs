//! Conformance scenarios and their runner. The runner is not built yet; this is the format it reads.
//!
//! # The `canon-conformance/1` scenario format
//!
//! A scenario file is a YAML document holding exactly one named scenario:
//!
//! ```yaml
//! format: canon-conformance/1
//! id: CANON-CLAIM-001                       # the scenario id, unique within the registry
//! covers: [CANON-CLAIM-001, CANON-CLAIM-002] # the CANON-* requirements it covers; may be empty
//! fixture: fixtures/investigation/protocol.yaml
//! steps:
//!   - id: compile                           # the step id, unique within the scenario
//!     compile: {}
//!     expect:
//!       canon-ir: |
//!         { ... the exact canon-ir/1 bytes `canon compile` prints ... }
//!   - id: no-evidence
//!     evaluate:
//!       case: { ... a canon-case/1 document ... }
//!       evidence: [ ... canon-evidence/1 records ... ]
//!       authority: [ ... canon-authority/1 decisions ... ]  # optional
//!       at: 2026-10-04T00:00:00Z                           # optional evaluation instant
//!     expect:
//!       decision: |
//!         { ... the exact canon-decision/1 bytes ... }
//!       # or, instead of `decision`:
//!       # refusal: <the identifier the refusal names>
//! ```
//!
//! - `fixture` is the `protocol/1` document the scenario compiles, as a path relative to the
//!   working directory `canon conform run` runs in (the repository root, by convention).
//! - `steps` is an ordered, non-empty list. Each step is exactly one of `compile` or `evaluate`,
//!   with exactly one expectation of the matching kind: a compile step expects `canon-ir`, an
//!   evaluate step expects `decision` or `refusal`.
//! - An expectation is compared byte for byte with the step's output. A YAML literal block (`|`)
//!   keeps one trailing newline, which is the one `canon compile` prints.
//! - Unknown keys are refused: a file with one does not parse as a scenario.
//!
//! # The registry
//!
//! The registry is the directory `conformance/scenarios/`: one file per story, named
//! `<story-name>.yaml`. There is no index file. `canon conform run` reads every `*.yaml` file of
//! the directory (`--scenarios <dir>`, default `conformance/scenarios`) in sorted file-name order
//! and runs each scenario's steps in order, stopping a scenario at its first failing step.
//!
//! # The report
//!
//! `canon conform run` prints one line per file, in the order read, then one summary line, all on
//! standard output:
//!
//! ```text
//! passed: scenario `<id>`
//! failed: scenario `<id>` step `<step>`: <what differed>
//! unreadable: <path>: <why>
//! conform: <n> passed, <n> failed, <n> unreadable
//! ```
//!
//! For a compile step whose output differs, `<what differed>` is `canon-ir/1 differs from the
//! expectation at line <n>`, `<n>` being the first differing line, counted from 1. `<path>` is the
//! scenario directory as given, joined with the file name. It exits 0 when every
//! scenario passed and 1 when any scenario failed or any file was unreadable. The same registry
//! produces byte-identical output on every run.
//!
//! Evaluate steps call the library entry point story:three-valued-claims provides. Until it lands,
//! an evaluate step fails as `failed: scenario `<id>` step `<step>`: evaluate steps are not
//! supported yet`.
