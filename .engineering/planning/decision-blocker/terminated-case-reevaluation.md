---
format: aep.planning-md/3
id: decision-blocker:terminated-case-reevaluation
kind: decision-blocker
status: cleared
title: A terminated case becomes unevaluable when later evidence blocks its outcome
relations:
- blocks: story:decision-outcomes
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-04T06:15:01Z", actor: "human:timo", revision: 3}
---
## Question

Should evaluating a case that terminated through an outcome which later evidence leaves blocked
refuse the whole decision (`illegitimate-termination`, wave 2026-10-04-w7 decision F5), or return
the decision and record the termination as no longer supported?

## Context

story:outcomes refuses a termination through a declared outcome that is blocked (design § 4.6: an
outcome is a declared legitimate terminal interpretation). Adversary pass 2 showed the consequence:
a case that ended legitimately becomes impossible to evaluate once a later record makes its claim
UNKNOWN, and the refusal drops the reasons.

## Options

- A: keep the refusal; re-evaluation of a closed case is a separate operation.
- B: return the decision and mark the termination as unsupported in the outcomes section.

## Decision (coordinator, wave 2026-10-04-w8)

Option A. Evaluating a case snapshot that records a termination through an outcome the evidence
leaves blocked stays refused as `illegitimate-termination`; this is the shipped behaviour of
story:outcomes. Re-evaluating a closed case against later evidence is a separate operation, to be
planned when a caller needs it. story:decision-outcomes builds on the refusal as it is.
