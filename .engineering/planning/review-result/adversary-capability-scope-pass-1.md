---
format: aep.planning-md/3
id: review-result:adversary-capability-scope-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:capability-scope (wave 2026-11-20-w1): green, 2 notes, both introduced'
relations:
- reviews: story:capability-scope
revision: 2
---
Adversary pass 1 on story:capability-scope, wave 2026-11-20-w1, `aep:adversary` on Opus. The
report below is as the agent returned it.

unit: story:capability-scope, impl/capability-scope at b2b81e1 (worktree clean)
verdict: green
cases: executed n/a→n/a (no case added, so the suite was not run), red 0
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: 1 path (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w1/capability-scope/adversary-p1/probe/`)
needs-coordinator: no

**1. Diff stat:** `git --no-pager diff --stat` is empty and `git status --short` is empty. No file in the worktree changed and nothing was committed.

**2. Cases added:** none. I found no behaviour that the story's rules contradict, and no mutant the unit's suite misses. Each mutant I checked against the unit's tests is caught by a literal assertion:
- coverage widened inwards;
- a deny applied inwards;
- precedence swapped;
- `require_approval` treated as granted;
- the capability filter dropped from `decide`;
- the `scopes` filter dropped;
- the `requirements()` chain dropped;
- reason scope omitted or always written;
- explanation order;
- duplicate check by capability only;
- scope identifier checks removed.

**3. Suite run:** not run, because I added no case. `slot task docs-check` passed: `canon-docs: 18 generated files are current`.

**4. Findings** (they cover b2b81e1)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/eval/authority.rs:285 | judgement | note | CONFIRMED | introduced | `canon evaluate --authority`, the documented workflow |
| website/docs/reference/documents.md:112 | judgement | note | CONFIRMED | introduced | `slot task check` run on commits bf1b6f4 or c6bf1c4 (bisecting) |

- **Narrower deny does not restrict a wider requirement** (authority.rs:285, `covers`). I ran the probe `a2.yaml` through the CLI: `allow m`, `deny m{cluster: prod-eu, namespace: secret}`, and `require_approval m{cluster: prod-us}`. The actions requiring `m{cluster: prod-eu}` and unscoped `m` both came out `admissible`, although they reach the denied namespace. The code follows the story's rule literally ("among the grants covering a requirement"; coverage never widens inwards). This is a question about the story's rule, not a defect in the code. The person who owns the story decides whether "deny wins unconditionally" should also cover a deny that only overlaps the requirement.
- **Two intermediate commits fail `docs-check`.** bf1b6f4 adds `rename_all = "snake_case"`, but canon-docs at that commit refuses anything other than `"lowercase"` (`source.rs:243`). The c6bf1c4 copy of `documents.md:112` reads "decided twice at an equal scope", while `pages.rs:324` at that commit still generates "decided twice is refused". So those generated files did not come from the generator as committed; the generator change only landed in 365c0df. The final tree is current.

**5. Attacked, not broken**
- Coverage:
  - a `{cluster: prod-eu, namespace: foo}` grant does not cover a `{cluster: prod-eu}` requirement;
  - a grant with an extra key covers nothing;
  - `scope: {}` on either side counts as no scope;
  - one capability required at two scopes, one granted: `approval-required`, naming only the uncovered one.
- Precedence: every pair and triple of decisions, plus a non-covering deny beside a covering allow. Results do not depend on decision order (BTreeMap; conform reverses the lists).
- IR side map: a hand-edited IR is refused by the canonical byte check, and `requires`/`scopes` cannot disagree in the text. Only a caller-built `Ir` can disagree, nothing outside `compile` builds one, and `requirements()` takes the union. A scoped and an unscoped requirement on one capability across actions, and the read-back of a scoped IR through `evaluate --ir`, both work. The IR of a protocol without a scope writes the same `{"capability": c}`.
- Duplicates:
  - keys reordered within an equal scope: refused;
  - `scope: {}` beside no scope: refused;
  - a key repeated in an authority entry or in JSON input: refused as malformed;
  - a key repeated in a protocol requirement: refused as `duplicate-scope-key`.
- `canon check` and `canon generate` on a scoped protocol: 0 findings and 5 scenarios, and `conform run` passes all 5. The only check findings that depend on authority are about properties; unscoped "grant all" already exposes every status difference, so the findings stay sound.
- `canon diff` on a changed scope, and the explanation's authority record for `require_approval` and for a scoped deny.
- Documentation: the regenerated pages and schemas match the behaviour, every `# read from:` line I checked points at the right lines, and AGENTS.md § ESS is consistent with `protocol.yaml`.
- `UNKNOWN`: no scope path touches a claim value.
- Known difference, not counted as a finding: the protocol reader accepts an unquoted `scope: {cluster: 1}` while the authority reader refuses it. Capability ids already behave the same way on the base.

**6. Paths written outside the worktree:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w1/capability-scope/adversary-p1/probe/` (probe protocols, authority documents, shell scripts, compiled IR, and the `gen/` output of `canon generate`).

The store refused the block above this note's place as unreadable YAML (a value holding `: `). The refusal went back to the author, who returned the block below; it replaces the first one and holds the same two findings.

```findings
[{"file": "crates/canon/src/eval/authority.rs", "line": 285, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a deny narrower than a requirement never covers it, so allow m beside deny m{cluster: prod-eu, namespace: secret} leaves an action requiring m{cluster: prod-eu} admissible, faithful to the story's rule but worth a decision by the story owner"}, {"file": "website/docs/reference/documents.md", "line": 112, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "commits bf1b6f4 and c6bf1c4 fail docs-check because canon-docs learned snake_case only in 365c0df, so the generated files committed in c6bf1c4 are not what that commit's generator produces"}]
```
