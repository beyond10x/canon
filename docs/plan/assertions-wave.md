# Assertion implementation

The operator approved implementation on 2026-10-05. This unit serves O2 by giving
assertions typed deterministic semantics. The coordinated engineering unit owns collection
and the CLI; Canon remains free of engineering vocabulary and I/O.

Managed tree: `$HOME/.local/state/worktree/trees/b10x/canon/canon-assertions`.
Branch: `feat/canon-assertions`, initial head `66c8d4b`.
Target: `$HOME/.cache/b10x-target/canon-assertions`.
Scratch: `$HOME/.cache/b10x-assertions/core`. Stage: verified, source publication blocked.
Spec commit `876e01e`; implementation `cf2ae40c2284c86d05b33c3003c45c7351f1150b`.
Final `task check` and site build pass; 21 expression tests and 598 workspace cases.
Two bounded adversary passes reproduced four defects, all corrected with retained cases.
See `verification-report:assertion-expressions` for exact counts and model policy.
`b10x-gates check` passed. Publication refused because Canon has no required App-only
branch-authority ruleset; see `blocker:assertion-publication`. No bypass was attempted.

Roles use `aep:implementing` 0.19.2 references (implementor, adversary), run through
Codex's generic agents because plugin agent types are not exposed. Coordinator owns
planning mutations. Approval covers spec/red-test commit, implementation, review fixes,
planning evidence and source-branch publication. No release or deployment.

First record the typed ESS home, then the story; first commit specification and failing
semantic tests. Check full workspace and ESS gate, docs drift, exact numeric boundaries,
bounded parser/recipe behavior, stale/unknown evidence and offline deterministic replay.
Catalog additions must not modify lexer, parser or operator semantics.

## Pull request submission resumed

On 2026-10-05 the operator requested submitted green pull requests. Canon's missing exact App-only
branch-authority rule was restored through the bot API (rule 24531196), and the publication blocker
was cleared in AEP. Current main is integrated for PR checks. Submission continues in managed tree
`canon-assertions-pr`, branch `feat/canon-assertions-pr`; earlier archives remain recovery snapshots.
