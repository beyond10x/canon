---
format: aep.planning-md/3
id: decision-blocker:terminated-case-reevaluation
kind: decision-blocker
status: open
title: A terminated case becomes unevaluable when later evidence blocks its outcome
relations:
- blocks: story:decision-outcomes
revision: 1
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
