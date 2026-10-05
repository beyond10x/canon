---
format: aep.planning-md/3
id: review-result:assertions-adversary-pass-2
kind: review-result
status: active
title: 'Expression adversary pass 2: formatter binding fixed, twenty tests green'
relations:
- reviews: story:assertion-expressions
revision: 1
---
The bounded second pass reproduced one regression from the formatter repair:
`(root).field` changed binding to `root.field` when both a record-valued fact and a dotted
catalog fact existed. The counterexample observed TRUE where the source was FALSE.

```findings
[{"file":"crates/canon-expr/src/parser.rs","line":1,"category":"correctness","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"Canonical formatting must preserve parentheses around a name receiver so field access does not become a dotted catalog name."}]
```

Implementor preserved name-receiver parentheses. Independent recheck: all four adversarial
cases pass; full crate suite 13 conformance + 4 adversary + 3 model parity = 20 passed, exit0.
No standing finding. Red and green logs retained under `$HOME/.cache/b10x-assertions/core-review/`.
