---
format: aep.planning-md/3
id: review-result:assertions-adversary-pass-1
kind: review-result
status: active
title: 'Expression adversary pass 1: three reproduced defects'
relations:
- reviews: story:assertion-expressions
revision: 1
---
Independent adversary added three Rust counterexamples in `crates/canon-expr/tests/adversary.rs`.
First execution: 0 passed, 3 failed. Log: `$HOME/.cache/b10x-assertions/core-review/pass1-red.log`.

```findings
[
 {"file":"crates/canon-expr/src/compiler.rs","line":579,"category":"correctness","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"check/plan loses explicit optional context schema; an absent optional plan cannot deserialize its own serialization."},
 {"file":"crates/canon-expr/src/parser.rs","line":1,"category":"boundary","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"Canonical formatting adds parentheses so an accepted 20-prefix-not expression exceeds parse depth."},
 {"file":"crates/canon-expr/src/compiler.rs","line":579,"category":"boundary","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"Two valid 40KiB bindings construct an 80KiB observation argument rejected by Value validation but admitted by planning."}
]
```

Routed to implementor. The optional replay fix preserves context_types in the typed PlanDocument;
that evidence-driven specification correction is required rather than guessing a type for None.
