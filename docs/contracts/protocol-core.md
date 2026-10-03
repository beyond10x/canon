# Contract Sketch — Canon Protocol Core

This is a sketch, not yet a final schema.

```text
Protocol
Case

Artifact
Revision

Fact
Claim
Predicate
Truth = TRUE | FALSE | UNKNOWN

EvidenceKind
Evidence

Obligation

Action
EffectClass
AuthorityRequirement

Outcome
InvalidationRule
RecoveryRule
```

## Deterministic evaluation

```text
Evaluation =
  f(
    protocol_ir,
    case_snapshot,
    evidence_set,
    authority_decisions,
    evaluation_instant
  )
```

No hidden IO is allowed in this function.

## Minimum evaluation output

```yaml
case_revision: 17

claims:
  tests.pass:
    value: unknown
    because:
      - current implementation is R2
      - applicable test evidence only exists for R1

obligations:
  - id: verify.tests
    status: open

actions:
  - id: tests.run
    status: admissible

  - id: repository.merge
    status: blocked
    because:
      - tests.pass is UNKNOWN

outcomes:
  accepted:
    status: blocked
```
