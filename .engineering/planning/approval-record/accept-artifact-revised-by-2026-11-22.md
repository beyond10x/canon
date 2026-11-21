---
format: aep.planning-md/3
id: approval-record:accept-artifact-revised-by-2026-11-22
kind: approval-record
status: approved
title: story:artifact-revised-by accepted and one wave for the nine open stories, on the operator's word
tags:
- operator-instruction
relations:
- decides: story:artifact-revised-by
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T23:23:19Z", actor: "human:timo", revision: 2}
---
## Decision

story:artifact-revised-by is accepted, and every open story is put into one wave, on the
operator's word typed in the coordinating session on 2026-11-22:

> Alright, accept the main story, and then accept all of them, put all of them into a single way.
> They all now belong to a single way. Then use Sonnet to implement that stuff. Integrate
> everything into one integration branch, including the stuff which is currently running.
> Dispatch one agent. Dispatch up to 6 more subagents. Sonnet medium to implement that stuff.
> Review only once in integration, all at once. Then 1 PR, 1 cut release. Ship this as fast as
> possible, but with slower review at the end.

and, a few minutes later:

> accept the draft. I demand to be accepted and included into the current procedure.

> Not a single wave. A wave. /wave

"Way" is read as "wave"; "the main story" and "the draft" as story:artifact-revised-by, the one
story that was a draft. The other eight open stories were accepted before.

## What it covers

- The acceptance of story:artifact-revised-by, with its two open choices taken as the story
  recommends (the story's § Decided).
- One wave for the nine open stories, against the collisions `aep plan artifact waves` names
  among them; one review on the integration branch instead of one per unit.
- The commits of that wave on the unit branches and on `wave/2026-11-21`, the pushes to draft
  pull request 28, and one release after the review.
