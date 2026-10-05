---
format: aep.planning-md/3
id: release-plan:canon-0-1-0
kind: release-plan
status: approved
title: Release Canon 0.1.0
relations:
- delivers: story:assertion-expressions
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-05T21:35:08Z", actor: "human:timo", revision: 2}
---
## Intent and authorization

The operator requested a new release of Canon and Engineering Protocols after merging their
assertion PRs. Publish Canon 0.1.0 first, from a green main commit. This is Canon's first source
release: protocol calculus, CLI and generic assertion expression library.

## Scope and procedure

Set workspace version and lockfile to 0.1.0; document the source-release contract in AGENTS.md;
record the first release in CHANGELOG.md; update installation/status prose and regenerate derived
documentation. No protocol or expression semantics change, so no new ESS/red specification is
required. ESS remains the newest release, verified as 0.53.0.

Run task check and the documentation build, land the bot-authored release preparation through a
green PR, verify the exact main commit's required source checks, then publish an annotated bot tag
0.1.0 and a bot-authored GitHub Release. This is a source release: no binary assets or crates.io
publication are specified. Verify tag peeling, Release identity and downloadable source archives.
Docs delivery runs asynchronously; no Atlas, Website or downstream consumer promotion is required.

The existing tests include 601 workspace tests and 42 repeated ESS-first cases on the integrated
feature head; the release candidate must pass its own gate. Earlier feature evidence is retained
in verification-report:assertion-expressions and GitHub PR #7. The exact released revision and
final CI/Release evidence will be retained in the delivery report.

## Candidate validation

The release version changes decision provenance. An initial `canon conform run` reproduced the
old-version mismatch in CANON-EXPLAIN-001. The expected Canon version in that scenario and the
CLI exact-output fixtures was updated from 0.0.0 to 0.1.0, preserving all other expected output.
Generated documentation records the rebuilt 0.1.0 executable. The release procedure now identifies
these versioned fixtures for the next release.
