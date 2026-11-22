---
format: aep.planning-md/3
id: story:pages-deploy-uses-the-artifact-it-uploaded
kind: story
status: draft
title: The Documentation site workflow deploys the artifact it uploaded, so a push publishes without a manual dispatch
tags:
- from-finding
- release
relations:
- decomposes: epic:canon-kernel
scope:
- confidence: cited
  path: .github/workflows/b10x-docs-site.yml
revision: 2
---
## Outcome

The "Documentation site" workflow (`b10x-docs-site.yml`) publishes the pages of the commit that triggered it on the first run, without a manual `workflow_dispatch`.

## Why

- Release 0.11.1 (2026-11-22): the workflow, triggered by `workflow_run` after "Documentation validation", failed at "Deploy Pages artifact" with "Found 0 artifacts". `gh run rerun --failed` then failed with "Multiple artifacts". A `workflow_dispatch` of `b10x-docs-site.yml --ref main` succeeded. Release 0.11.0 needed the same.
- The failure is intermittent: the deploy step looks the artifact up by name after the upload step of the same workflow, and finds none (an upload-then-fetch race), and a rerun uploads a second artifact of the same name.

## Scope

- **In:** read the workflow's upload and deploy steps; make the deploy use the artifact the upload step produced (the action's output or id, not a lookup by name); a rerun of the failed job must not leave two artifacts of one name; show a run on a commit of a branch that passes without a dispatch.
- **Out:** the content of the site, the validation workflow.
- **Surfaces:** `.github/workflows/b10x-docs-site.yml`.

## Acceptance

Two consecutive pushes to `main` publish their pages with no manual dispatch (run links recorded as `test_result`), and `gh run rerun --failed` on a failed deploy finishes green.

## Source

- **Request:** the release of 0.11.1 and of 0.11.0, in this session's reports.
- **Not run:** nothing of the workflow was changed or read beyond its failure logs.
