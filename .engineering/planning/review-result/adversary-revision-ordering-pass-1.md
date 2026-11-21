---
format: aep.planning-md/3
id: review-result:adversary-revision-ordering-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on the descended form: red by two cases'
relations:
- reviews: story:revision-ordering
revision: 1
---
Adversary pass 1 on the unit of story:revision-ordering, branch impl/revision-ordering at e9b99e2e, run on Opus on 2026-11-21. The report follows verbatim.

unit: story:revision-ordering (findings cover 39adcf2c9 = e9b99e2e plus one test file)
verdict: red
cases: executed 1245→1249, red 2
origin: introduced 7, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w9/revision-ordering/adversary-p1 (scratch only)
needs-coordinator: no

**1. Diff since e9b99e2e:** `crates/canon/tests/adversary_descended.rs | 203 +++` (1 file, test only). It is committed as the bot in `39adcf2c9 test: adversary pass 1 on the descended form`. The tree is clean and the lease is released.

**Charter note.** Mutants were swapped into `eval/order.rs`, `eval/case.rs` and `eval/evidence.rs` and restored with `git checkout --` while I held the lease. The brief asked for this. The aep adversary charter forbids it. Kept copies: `…/adversary-p1/mutate/keep/`.

**2. Cases added** (`crates/canon/tests/adversary_descended.rs`). The first run had these alone, before the suite.
- `:37 an_ancestor_that_is_not_an_identifier_is_refused_naming_the_case_and_the_artifact`. **Red.**
  `{revision: r2, descends_from: ['r 1']}: the refusal names the case and the artifact, as the ess/ comment of descends_from says; it reads: artifact ancestor revision `r 1` is empty or contains whitespace or a control character`
- `:101 the_published_schema_of_item_holds_the_identifier_rule_the_evaluator_applies`. **Red.**
  `the schema of `item` accepts what the evaluator refuses as invalid-identifier: {…"type":"string"}  left: (Null, Null) right: (String("^\\S+$"), Number(1))`
- `:158 descended_pairs_only_records_of_the_results_it_names`. Green now; red under M11, M12 and M13.
- `:179 descended_reads_no_observed_at`. Green: a pass observed before the fail, at equal instants, or with no instant gives TRUE.

**3. Suite** (`cargo test --workspace --locked --no-fail-fast`, after the cases existed):
- Exit 101.
- 163 summaries: 1247 passed, 2 failed, 3 ignored.
- The only failures are the two red cases above (`error: 1 target failed: -p b10x-canon --test adversary_descended`).
- The before count of 1245 is the unit's own report.
- `rustfmt --check` and `cargo clippy --test adversary_descended -D warnings` both exit 0.

**4. Findings**

| file:line | category | sev | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/eval/case.rs:96 | contract-drift | warning | NEEDS-CHANGE | introduced | Any `canon evaluate` whose case lists an ancestor that is not an identifier. ESS `protocol.yaml:1036-1040` and the docs of `case.rs`/`mod.rs` say all three refusals name the case, the artifact and the revision. `invalid-identifier` names only the revision, exit 1. |
| website/static/schemas/evidence-1.schema.json:71 | contract-drift | warning | NEEDS-CHANGE | introduced | ESS `protocol.yaml:1132` types `item` as `Optional<String>`, so the schema and `documents.md` say plain text. `item: ""` and `item: "t 1"` validate (jsonschema 4.26) and `canon evaluate` refuses them with `invalid-identifier`. Every other identifier-checked field is `Identifier`. |
| crates/canon/src/eval/order.rs:296 | mutant | note | CONFIRMED | introduced | Dropping the result test of `first`, of `then` (M12, M13) or of both (M11) leaves every `eval::`/`check::tests` lib test and all 29 conform scenarios green. Every unit row puts R1 on the ancestor and R2 on the current revision. Now caught by `adversary_descended.rs:158`. |
| crates/canon/src/explain/mod.rs:349 | judgement | warning | NEEDS-CHANGE | introduced | Every evaluation with `descends_from`. `computed_from.case.artifacts` writes only `revision`. Rows t04, x04 and t09 (TRUE/FALSE/UNKNOWN, same evidence) have byte-identical `computed_from`, so the decision cannot be rebuilt from it. `independence` and `inputs` are recorded there; `descends_from` is not. No red case: no story or ESS sentence covers it. |
| crates/canon/src/eval/order.rs:313 | judgement | note | CONFIRMED | introduced | The time per later record grows with the history: O(later × history). Fixture, N tests each passed on R1 and R2 (FALSE), debug build: 500→0.17 s, 2000→1.75 s, 4000→6.82 s. The same 8000 records under `after` take 0.59 s. |
| fixtures/investigation/revision-ordering.yaml:13 | judgement | note | CONFIRMED | introduced | `test_result`, `merge`, `red.then.green`. AGENTS.md § Boundary says Canon has no engineering vocabulary and makes no exception for fixtures. The existing order fixtures already use `test_result` (5 files) and `merge` (5). The model, ESS, schemas and refusal texts use neutral names (`descended`, `descends_from`, `item`). |
| ess/domains/protocol.yaml:821 | judgement | note | CONFIRMED | introduced | Row x11: T failed on an ancestor and failed now gives UNKNOWN (table row 2), while a pass now with no failure gives FALSE. A user may expect FALSE for "still failing". It does not contradict § Acceptance. |

**Rows evaluated** (`canon evaluate`, own protocol: subject `a` at R2 descending from [R1, R0], `b` at R2):

| rows | binary | ESS comment | equal |
|---|---|---|---|
| table 1–10 (none; R1 anc only; R2 now only; pair; R1 unlisted; both current; T/U; T pair + U; no descends_from; no items) | U U F T F F F T U U | the same | yes, all 10 |
| fail anc + pass now + fail now; two later of T; R1 on two ancestors | T T T | "true when a later record has an earlier record" | yes |
| `descends_from: []` with pair / with no record | F / U | CaseArtifact: "`[]` says it descends from none" | yes |
| first = then = (k, passed): pass anc + pass now / one pass now | T / F | "may name one kind, and one result" | yes |
| no subject: a (listed) later without pair, b (unlisted) later | U (d.a=F) | "unknown … when the case gives no descends_from for the subject of one" | yes |
| no subject: a pair + b unlisted later / cross-artifact pair | T / F | "about the same subject" | yes |
| fail anc + fail now; pass now without item; later without result | U U U | not R2, or no item → no later record | yes |
| observed_at contradicts / equal / one side only (o01–o03) | T T T | "never observed_at" | yes |
| item case differs (t vs T) | F | "compared for equality only" | yes |
| record without subject_revision / about undeclared z / item without subject | malformed / undeclared-artifact / malformed (exit 1) | not decided (refused before the form) | n/a |

**5. Attacked and not broken**
- **Claim 1:** all 5 CANON-ORDER-005 steps pass.
- **Claim 4:** checked against the unit's `canon-base` (0.10.0, built 18:47, between 738f895a at 18:43 and the spec commit at 18:54, no `descends_from` string). Over 146 fixtures, validate/compile/check are 0 of 146 different each. Of 198 scenario evaluate steps, 0 differ. The 0.10.0-written IR read by the new binary gives the base bytes in 198 of 198. The rank shift keeps the relative order.
- **Composition:** `not`, `all` with an UNKNOWN member, `any`, precondition, outcome requirement direct and through two claims all follow Kleene logic. In reasons and explanation, `subject` appears only when named, after the other order forms.
- **Claim 5 refusals (exit 1):** duplicate (also `R1`/`'R1'`/`"R1"`), self-descent (also before a duplicate), empty string, null, integer and float entries, misspelled key. Validate refuses undeclared first/then kinds (first before then, each once) and undeclared subject. `evidence-in-discharge` and `evidence-in-applicability` refuse with exit 1. 10,000 ancestors take 0.05 s.
- **Redelivery:** equal items accepted. A different item, an item missing or an item added is refused `duplicate-identifier`, the same as a producer change at base.
- **IR reader:** refuses an extra key, a missing result or then, and a non-string subject.
- **`canon generate`:** 2 scenarios, both pass `conform run`, and both carry `item` and `descends_from: ["r0"]`. `canon check` covers the fixture with 72 states.
- **`canon diff`:** a result change is BREAKING, as for `since`.
- **Schemas:** 0.10.0 case and record validate, new fields validate, misspelled fields are refused, 147 protocol fixtures are valid.
- **Re-pinned tests:** only the counts 29→30, the catalogue entry and the rank mirror changed.
- **Mutants caught by the unit's suite (10):**
  - M1 descends_from ignored: order tests and conform `failed-on-a-revision-not-descended-from`.
  - M2 current-revision test dropped: `descended_pairs_an_item_across_a_descended_revision`.
  - M3 item equality dropped: same order test plus conform `failed-and-passed-under-different-items`.
  - M4 itemless record takes part: check and order tests.
  - M5 missing descends_from gives FALSE: order tests plus conform `descent-not-given`.
  - M6 subject filter dropped: order test.
  - M7 earlier record on the current revision accepted: check and order tests.
  - M8 self-descent not refused: `eval::case::tests::descends_from_is_refused…`.
  - M9 redelivery item not compared: `eval::evidence::tests::a_copy_that_differs_in_any_field_is_refused`.
  - M10 same-subject test dropped: `descended_without_a_subject_pairs_records_about_one_artifact`.
- **Mutants that survived the unit's suite (3):** M11, M12 and M13 stayed green; only my case catches them (finding 3).

**6. Paths written outside the worktree:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w9/revision-ordering/adversary-p1/` (82 MB):
- `probe/` with `rows`, `refusals`, `validate`, `claim4`, `perf`, `gen`, `diff` and `irread`
- `mutate/` (logs, `keep/`)
- `build.log`, `red-alone.log`, `suite.log`, `clippy.log`, `msg-1.txt`

```findings
[{"file": "crates/canon/src/eval/case.rs", "line": 96, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The invalid-identifier refusal of a descends_from entry names only the revision, though the ESS comment of descends_from says each of the three refusals names the case, the artifact and the revision; red in adversary_descended.rs:37."},
{"file": "website/static/schemas/evidence-1.schema.json", "line": 71, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "item is published as a bare string (ESS type Optional<String>), so an empty item or one with whitespace validates against the schema and canon evaluate refuses it as invalid-identifier; red in adversary_descended.rs:101."},
{"file": "crates/canon/src/eval/order.rs", "line": 296, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Ignoring the result of first, of then, or both left every unit test, check test and conformance scenario green; adversary_descended.rs:158 now catches all three."},
{"file": "crates/canon/src/explain/mod.rs", "line": 349, "category": "judgement", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "computed_from records each case artifact without descends_from, so TRUE, FALSE and UNKNOWN decisions over the same evidence carry byte-identical computed_from and cannot be reconstructed from it."},
{"file": "crates/canon/src/eval/order.rs", "line": 313, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "descended is quadratic in the history: 4000 items on two revisions take 6.82 s in a debug build against 0.59 s for an after form over the same records."},
{"file": "fixtures/investigation/revision-ordering.yaml", "line": 13, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The fixture names test_result, merge and red.then.green, engineering vocabulary AGENTS.md Boundary excludes without a fixture exception, following the existing order fixtures."},
{"file": "ess/domains/protocol.yaml", "line": 821, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "An item that failed on an ancestor and still fails now is UNKNOWN while a pass with no earlier failure is FALSE; a user may expect FALSE for a test that is still failing."}]
```
