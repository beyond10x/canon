---
format: aep.planning-md/3
id: review-result:adversary-conformance-suite-pass-1
kind: review-result
status: active
title: Wave conformance-suite adversary, canon story:conformance-suite, pass 1
relations:
- reviews: story:conformance-suite
revision: 1
---
The adversary pass on story:conformance-suite found one warning and one note at e432ce8; the probes are kept as regression tests (fa12df0) and the fixes are 21a147c. Coverage, the planned-but-covered check, a failing scenario covering nothing, the determinism check on real inputs, exit codes 1 and 2 and the report without the flag all held.

```
unit: story:conformance-suite, tree canon-adv-conf, at e432ce8
verdict: NEEDS-CHANGE
cases: executed 1→20 (crates/canon/tests/adversary_conformance_suite.rs, crates/canon-cli/tests/adversary_conformance_suite_cli.rs), red 2
origin: introduced 2 / pre-existing 0 / undecided 0
```

## Findings

1. `crates/canon/src/conform/requirements.rs` (`parse_catalogue`), warning, introduced: a refused catalogue quoted its `format` value and any key that is not a `CANON-` id, although the module docs promise a catalogue that does not parse is reported by kind and position only. A file passed to `--requirements` by mistake had its values echoed. Red: `a_refused_catalogue_does_not_echo_its_values`.
2. `crates/canon/src/conform/requirements.rs` (`output`), note, infeasible: a step whose list cannot be written back is never evaluated, yet its compared output carried the entry's index, which moves when the list is reversed, so the step was also reported `nondeterministic: … permuted differs`. Red: `an_unevaluated_step_is_not_reported_nondeterministic`.

## Decisions on the findings

- 1: the refusals say `format is not `canon-requirements/1`` and `requirement <n> is not a CANON-* requirement`, naming the position only.
- 2: an unserializable step compares as one constant output; the runner already fails that step.

## What held

Unknown keys inside a requirement, malformed ids (empty suffix, zero-width, lowercase, leading space), one id written quoted and unquoted, null and missing fields, a whitespace-only statement, null `requirements` and an empty file are refused. A scenario failing at its second step covers nothing; a planned requirement covered only by a failing scenario is not reported; an unlisted id in a failing scenario is. Check lines come in catalogue order, then registry order. With the flag the output is the plain report plus the `requirements:` line, byte for byte; without it the unit's nondeterministic fixture registry gives only the runner's report. A missing catalogue or a directory exits 2 with empty standard output; a file that is not a catalogue exits 1. The real catalogue lists exactly the 15 requirements the story names, and the registry's passing scenarios cover them all.
