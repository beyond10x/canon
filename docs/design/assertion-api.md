# canon-expr/1 interface

The library package is `b10x-canon-expr`, imported as `canon_expr`. All maps are ordered. The evaluator performs no IO.

```rust,ignore
pub fn parse(source: &str) -> Result<ParsedExpr, Diagnostic>;
pub fn check(parsed: ParsedExpr, catalog: &Catalog, context_types: &BTreeMap<String, Type>) -> Result<CheckedExpr, Diagnostic>;
pub fn plan(checked: CheckedExpr, bindings: &Bindings) -> Result<Plan, Diagnostic>;
pub fn compile(sources: &[String], catalog: &Catalog, bindings: &Bindings) -> Result<Plan, Diagnostic>;
pub fn evaluate(plan: &Plan, evidence: &[Observation], context: &EvalContext) -> Result<Report, Diagnostic>;
```

Wire types derive Serde, deny unknown fields, and use tagged enums `{kind: snake_case, value: ...}`. The complete typed schema is `ess/domains/expr.yaml`.

- `Type`: `Scalar(Scalar)`, `List(Box<Type>)`, `Map(Box<Type>)`, `Record(BTreeMap<String,Type>)`, `Optional(Box<Type>)`. Associated constants `Type::Bool`, `Type::String`, `Type::Integer`, `Type::Decimal` abbreviate Scalar values.
- `Value`: `Bool(bool)`, `String(String)`, `Integer(String)`, `Decimal(Exact)`, `List(Vec<Value>)`, `Map(BTreeMap<String,Value>)`, `Record(BTreeMap<String,Value>)`, `Optional(Option<Box<Value>>)`. Integer strings are canonical signed base10 i128. `Exact { coefficient: String, scale: u8 }` is a canonical i128 coefficient times 10^-scale; no trailing zero coefficient with nonzero scale. Helpers `Value::integer(i128)`, `Value::from_json(value, type)`, `Value::to_json()` preserve exact JSON numeric lexemes. Provider JSON uses ordinary values; retained evidence uses typed values.
- `Catalog { format: String, functions: BTreeMap<String,Function> }`, format `canon-catalog/1`.
- `Function { parameters: Vec<Parameter>, returns: Type, implementation: Implementation }`; `Parameter { name, value_type: Type, default: Option<Value> }`.
- `Implementation::Observation(Provider { provider, operation, digest })`, `Recipe(String)`, `Pure(Builtin)`. Provider strings identify an explicitly registered implementation. `Builtin` is one of `Length, Contains, StartsWith, EndsWith, Present`; implementations cannot install new operators or pure native code.
- `Bindings { values: BTreeMap<String,Value>, source_identity: String, context_identity: String }`.
- `Request { id, provider, operation, digest, arguments: BTreeMap<String,Value>, result_type: Type }`.
- `Observation { request_id, plan_id, observed_at: i64, valid_until: Option<i64>, outcome: Outcome }`; `Outcome::Known(Value)` or `Unavailable(String)`.
- `EvalContext { now: i64, source_identity: String, context_identity: String }`. Time is integer epoch seconds, injected by the caller.
- `Report { plan_id, truth: Truth, assertions: Vec<Evaluation> }`; `Evaluation { expression, truth, reasons: Vec<String> }`; `Truth::True | False | Unknown`.
- `Plan` exposes `id: String` and `requests: Vec<Request>`. Its serialization is `PlanDocument { id, sources, catalog, bindings, requests }`; deserialization recompiles and compares the derived identity and requests, rejecting tampering. Compile collects and deduplicates requests across all sources.
- `Diagnostic { code: String, message: String, start: usize, end: usize, expression: Option<usize> }` uses byte spans; the edge adds document path/gate identity.

All assertion roots must be Bool; an empty source list is refused. Variables are `$name`. Catalog facts omit parentheses only with zero parameters. Named arguments use `name: value`. Lists use `[]`, records `{field: value}` (quoted keys allowed). Comparisons are nonassociative; `not x == y` means `not (x == y)`. Precedence ascending: or, and, not, comparisons, unary minus, field/index/call. Decimal literals allow an optional exponent and reject overflow/scale outside u8. Numeric comparisons never convert to floating point.

All observation arguments must be bindable without observation results. Boolean operators do not condition acquisition. Recipes substitute only declared parameters, have no ambient variable capture, and cannot form cycles. Declared pure builtin signatures are checked against their semantics. Missing or stale evidence yields Unknown, a known false stays False, and optional absence is a known value. Invalid evidence is a hard error even in a false or true boolean branch. Evidence collected after the explicit evaluation instant is unavailable. Validity expires when now exceeds valid_until.

Source and context identities are supplied by the edge and must identify complete relevant input snapshots. Catalog definitions, provider digests, canonical bound values and language version enter the plan identity. Retained evidence for a different plan/source/context is unavailable. Evidence with duplicate or unrequested request IDs is refused. The engine is deterministic for identical normalized inputs.

Limits are part of language version 1: source 64 KiB, 16,384 tokens, expression nesting 32, 4,096 expression nodes after recipe expansion, 128 recipe depth, 1,024 assertions, and 4,096 unique requests. Collection limits and provider supervision belong to the edge; values and catalog types are checked recursively before use. No loops, assignments, shell interpolation, runtime operator registration or implicit truthiness exist.
