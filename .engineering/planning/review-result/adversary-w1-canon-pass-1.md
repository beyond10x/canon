---
format: aep.planning-md/3
id: review-result:adversary-w1-canon-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w1 adversary, canon unit, pass 1
relations:
- reviews: story:protocol-source-model
revision: 1
---
unit: canon
verdict: red
cases: executed 16→18, red 1
origin: introduced 5, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ga-wave-2026-10-04-w1/canon/scratch/{mutant/, probe-empty-is.yaml, probe-ids.yaml, probe-forge.yaml, probe-chain.yaml, probe-chain-20000.yaml, probe-cycles.yaml}
needs-coordinator: no

Findings cover the uncommitted working tree on base `7840f412` in `~/.local/state/worktree/trees/b10x/canon/canon-w1-protocol-source-model`.

**1. What I touched.** `git diff --stat 7840f412` is unchanged: `Cargo.lock`, `crates/canon/Cargo.toml` and `crates/canon/src/lib.rs` are all the implementor's. I added three untracked files, all of them tests or test data, and changed no implementation file:
- `crates/canon/tests/adversary_source_model.rs`
- `crates/canon-cli/tests/adversary_validate.rs`
- `crates/canon-cli/tests/adversary-unparseable.yaml`

**2. Cases added (each run alone before the suite)**

| case | asserts | now |
|---|---|---|
| `adversary_source_model::a_claim_test_whose_is_has_no_value_is_a_parse_error` | `claim: a` with `is:` / `is: ~` / `is: null` fails to parse | **red** |
| `adversary_validate::validate_rejects_a_document_that_does_not_parse` | an unparseable document makes `canon validate` exit 1 with `error[parse]: …owner…` | green; it catches a mutant |

Red output:
```
`is:` was accepted as Ok([Claim(ClaimTest { claim: ClaimId("a"), is: True })])
test result: FAILED. 0 passed; 1 failed
```
Mutant proof: in a scratch copy I changed `main.rs:57` from `return ExitCode::from(REJECTED)` to `return ExitCode::SUCCESS`. The unit's own `tests/validate.rs` still passed 4 of 4. The new case failed with `left: Some(0) right: Some(1)`. So a parse error exiting 0 would get past the current suite, because every fixture under `invalid/` parses.

**3. Suite (after the cases existed)**
- `cargo fmt -p b10x-canon -p canon-cli --check`: exit 0.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --workspace --locked --no-fail-fast`: exit 101. 18 tests ran and 1 failed (the `is:` case). The 16 in `source_model` (12) and `validate` (4) pass.
- The "before" count of 16 comes from this same run with my two test binaries left out.

**4. Judgement findings (verified with the CLI, no test case)**
- `crates/canon/src/validate/mod.rs:98`: identifiers have no grammar. An empty protocol id and an empty artifact id are both accepted. A claim id containing a line break printed one problem as two lines, one of them a fake `error[duplicate-identifier]: action \`inspect\`…`. Nothing parses canon's stderr yet, so this is a note. CONFIRMED, introduced.
- `crates/canon/src/validate/mod.rs:244`: the doc comment says the check reports every cycle, but the search only reports some of them. With `a→{b,c}, b→c, c→a`, only `a -> b -> c -> a` is printed; `a -> c -> a` is not. The document is still rejected, so the fix is to reword the comment. CONFIRMED, introduced.
- `crates/canon/src/validate/mod.rs:255`: the cycle check is recursive and overflows the stack on a chain of claims. In a debug build with an 8 MiB stack, 15,000 claims pass and 20,000 crash (exit 134). No real protocol is that large. INFEASIBLE, introduced.
- `fixtures/investigation/protocol.yaml`: the story describes the fixture as the § 12 example plus an artifact and minus `inconclusive`. The fixture also drops `case: inputs:` and adds `evidence_kinds:`. That matches the story's own Outcome list, so the story text is what's loose. CONFIRMED, introduced.
- `crates/canon/Cargo.toml:6`: `repository.workspace = true` matches the brief's literal "no `repository` in crates/" rule. It is a Cargo setting, not engineering vocabulary, and it was already in the base. CONFIRMED, pre-existing.

Fix for the red case (I did not apply it): at `predicate.rs:124`, keep `#[serde(default)]` so a missing `is` still means `true`, but reject an explicit null, for example with a `deserialize_with` that parses `Truth` and wraps it in `Some`. A bare `precondition:` and `result:` behave the same way (an empty value counts as absent). I didn't write cases for those.

**5. Attacked and held**
- The acceptance check: all three variants name the offending identifier, exit 1, and give identical output over 5 runs.
- Duplicate keys in all six sections are kept and reported. `1` and `"1"` count as the same id.
- References are resolved inside `all`, `any` and `not`, and in preconditions, `may_produce` and outcomes.
- The library does no file, environment, clock or network access (grep of `crates/canon/src` found none).
- The order problems are reported in is stable.

**6. Paths written outside the worktree:** the `mutant/` copy (121M, including its own `target/`) and the six `probe-*.yaml` files, all under `~/.cache/ga-wave-2026-10-04-w1/canon/scratch/`. The `*.log` files in that directory are not mine. Free space on `/` is 11G.

```findings
[
  {"file": "crates/canon/src/model/predicate.rs", "line": 124, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A claim test written with an empty `is:` (or `~`/`null`) parses as `is: true` instead of being rejected, although the Truth deserializer admits only true, false or unknown."},
  {"file": "crates/canon-cli/src/main.rs", "line": 57, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "No existing case drives canon validate through its parse-error branch, so a mutant exiting 0 there keeps the suite green; adversary_validate.rs now catches it."},
  {"file": "crates/canon/src/validate/mod.rs", "line": 98, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Identifiers have no grammar: empty ids are accepted and an id containing a line break makes one problem render as two error lines with a forged code."},
  {"file": "crates/canon/src/validate/mod.rs", "line": 244, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The doc says every cycle is reported, but the search reports only some: for a->{b,c}, b->c, c->a it reports a->b->c->a and not a->c->a."},
  {"file": "crates/canon/src/validate/mod.rs", "line": 255, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "The recursive cycle check overflows the stack between 15k and 20k chained claims (debug build, 8 MiB stack), but no realistic protocol reaches that size."},
  {"file": "fixtures/investigation/protocol.yaml", "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The base fixture also drops the design section 12 `case: inputs:` block and adds `evidence_kinds:`, beyond the two deviations the story names."},
  {"file": "crates/canon/Cargo.toml", "line": 6, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "`repository.workspace = true` literally matches the brief's ban on `repository` in crates/, though it is a Cargo setting and not engineering vocabulary."}
]
```
