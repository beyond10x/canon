use crate::compiler::{Expr, Typed};
use crate::parser::Op;
use crate::*;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
struct State {
    value: Option<Value>,
    reasons: BTreeSet<String>,
}
impl State {
    fn known(v: Value) -> Self {
        Self {
            value: Some(v),
            reasons: BTreeSet::new(),
        }
    }
    fn unknown(reason: impl Into<String>) -> Self {
        Self {
            value: None,
            reasons: BTreeSet::from([reason.into()]),
        }
    }
    fn truth(&self) -> Result<Truth, Diagnostic> {
        match &self.value {
            Some(Value::Bool(true)) => Ok(Truth::True),
            Some(Value::Bool(false)) => Ok(Truth::False),
            None => Ok(Truth::Unknown),
            _ => Err(error("type", "assertion value is not boolean")),
        }
    }
}
pub fn evaluate(
    plan: &Plan,
    evidence: &[Observation],
    context: &EvalContext,
) -> Result<Report, Diagnostic> {
    if plan.id != plan.document.id || plan.requests != plan.document.requests {
        return Err(error("plan-integrity", "public plan fields were modified"));
    }
    if evidence.len() > MAX_REQUESTS {
        return Err(error("evidence-budget", "too many observations"));
    }
    let requests = plan
        .requests
        .iter()
        .map(|r| (&r.id, r))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut states = BTreeMap::new();
    let mut payload = 0;
    for obs in evidence {
        if !seen.insert(&obs.request_id) {
            return Err(error(
                "duplicate-evidence",
                format!("duplicate {}", obs.request_id),
            ));
        }
        let request = requests.get(&obs.request_id).ok_or_else(|| {
            error(
                "unrequested-evidence",
                format!("unknown request {}", obs.request_id),
            )
        })?;
        if obs.valid_until.is_some_and(|t| t < obs.observed_at) {
            return Err(error("evidence-time", "valid_until precedes observed_at"));
        }
        match &obs.outcome {
            Outcome::Known(v) => {
                if !v.matches(&request.result_type)? {
                    return Err(error(
                        "evidence-type",
                        format!("{} does not match provider result contract", obs.request_id),
                    ));
                }
                payload += serde_json::to_vec(v)
                    .map_err(|e| error("encoding", e.to_string()))?
                    .len();
            }
            Outcome::Unavailable(reason) => {
                if reason.len() > MAX_SOURCE {
                    return Err(error("evidence-budget", "unavailable reason too large"));
                }
                payload += reason.len();
            }
        }
        if payload > MAX_BOUND_BYTES {
            return Err(error("evidence-budget", "evidence payload exceeds 1 MiB"));
        }
        let state = if obs.plan_id != plan.id {
            State::unknown(format!("{}: plan mismatch", obs.request_id))
        } else if obs.observed_at > context.now {
            State::unknown(format!("{}: observed in future", obs.request_id))
        } else if obs.valid_until.is_some_and(|t| context.now > t) {
            State::unknown(format!("{}: expired", obs.request_id))
        } else {
            match &obs.outcome {
                Outcome::Known(v) => State::known(v.clone()),
                Outcome::Unavailable(reason) => {
                    State::unknown(format!("{}: {reason}", obs.request_id))
                }
            }
        };
        states.insert(obs.request_id.clone(), state);
    }
    let identity_matches = context.source_identity == plan.document.bindings.source_identity
        && context.context_identity == plan.document.bindings.context_identity;
    let mut assertions = Vec::new();
    let mut truth = Truth::True;
    for (source, root) in plan.document.sources.iter().zip(&plan.roots) {
        let state = if identity_matches {
            eval(root, &states)?
        } else {
            State::unknown("source or context identity mismatch")
        };
        let result = state.truth()?;
        truth = and(truth, result);
        assertions.push(Evaluation {
            expression: source.clone(),
            truth: result,
            reasons: state.reasons.into_iter().collect(),
        });
    }
    Ok(Report {
        plan_id: plan.id.clone(),
        truth,
        assertions,
    })
}
pub(crate) fn known(node: &Typed) -> Result<Value, Diagnostic> {
    eval(node, &BTreeMap::new())?.value.ok_or_else(|| {
        error(
            "acquisition-dependency",
            "observation arguments are not known",
        )
    })
}
fn and(a: Truth, b: Truth) -> Truth {
    match (a, b) {
        (Truth::False, _) | (_, Truth::False) => Truth::False,
        (Truth::True, Truth::True) => Truth::True,
        _ => Truth::Unknown,
    }
}
fn or(a: Truth, b: Truth) -> Truth {
    match (a, b) {
        (Truth::True, _) | (_, Truth::True) => Truth::True,
        (Truth::False, Truth::False) => Truth::False,
        _ => Truth::Unknown,
    }
}
fn boolean(t: Truth, reasons: BTreeSet<String>) -> State {
    State {
        value: match t {
            Truth::True => Some(Value::Bool(true)),
            Truth::False => Some(Value::Bool(false)),
            Truth::Unknown => None,
        },
        reasons,
    }
}
fn gather(
    values: impl IntoIterator<Item = Result<State, Diagnostic>>,
) -> Result<(Option<Vec<Value>>, BTreeSet<String>), Diagnostic> {
    let mut collected = Vec::new();
    let mut reasons = BTreeSet::new();
    let mut missing = false;
    for state in values {
        let state = state?;
        reasons.extend(state.reasons);
        if let Some(v) = state.value {
            collected.push(v);
        } else {
            missing = true;
        }
    }
    Ok((if missing { None } else { Some(collected) }, reasons))
}
fn eval(node: &Typed, states: &BTreeMap<String, State>) -> Result<State, Diagnostic> {
    let state = eval_inner(node, states)?;
    if let Some(value) = &state.value {
        value.validate()?;
    }
    Ok(state)
}
fn eval_inner(node: &Typed, states: &BTreeMap<String, State>) -> Result<State, Diagnostic> {
    match &node.expr {
        Expr::Literal(v) => Ok(State::known(v.clone())),
        Expr::Request(id) => Ok(states
            .get(id)
            .cloned()
            .unwrap_or_else(|| State::unknown(format!("{id}: missing")))),
        Expr::Variable(_) | Expr::Observation(..) => {
            Err(error("plan-integrity", "unbound plan node"))
        }
        Expr::List(values) => {
            let (v, reasons) = gather(values.iter().map(|v| eval(v, states)))?;
            Ok(State {
                value: v.map(Value::List),
                reasons,
            })
        }
        Expr::Record(values) | Expr::Map(values) => {
            let (v, reasons) = gather(values.values().map(|v| eval(v, states)))?;
            Ok(State {
                value: v.map(|v| {
                    let fields = values.keys().cloned().zip(v).collect();
                    if matches!(node.expr, Expr::Map(_)) {
                        Value::Map(fields)
                    } else {
                        Value::Record(fields)
                    }
                }),
                reasons,
            })
        }
        Expr::Field(value, field) => {
            let mut s = eval(value, states)?;
            s.value = match s.value {
                Some(Value::Record(v)) => Some(
                    v.get(field)
                        .cloned()
                        .ok_or_else(|| error("field", "record field absent"))?,
                ),
                None => None,
                _ => return Err(error("type", "field access on nonrecord")),
            };
            Ok(s)
        }
        Expr::Index(value, index) => {
            let (v, reasons) = gather([eval(value, states), eval(index, states)])?;
            let value = if let Some(v) = v {
                match (&v[0], &v[1]) {
                    (Value::List(list), Value::Integer(index)) => {
                        let i = crate::number::canonical_integer(index)?;
                        let i = usize::try_from(i).map_err(|_| {
                            error("index", "negative or unrepresentable list index")
                        })?;
                        Some(
                            list.get(i)
                                .cloned()
                                .ok_or_else(|| error("index", "list index out of bounds"))?,
                        )
                    }
                    (Value::Map(map), Value::String(key)) => {
                        Some(map.get(key).cloned().ok_or_else(|| {
                            error("index", "map key absent; model optional entries explicitly")
                        })?)
                    }
                    _ => return Err(error("type", "invalid index operands")),
                }
            } else {
                None
            };
            Ok(State { value, reasons })
        }
        Expr::Not(v) => {
            let s = eval(v, states)?;
            Ok(boolean(
                match s.truth()? {
                    Truth::True => Truth::False,
                    Truth::False => Truth::True,
                    Truth::Unknown => Truth::Unknown,
                },
                s.reasons,
            ))
        }
        Expr::Neg(v) => {
            let mut s = eval(v, states)?;
            s.value = s.value.map(crate::number::negate).transpose()?;
            Ok(s)
        }
        Expr::Binary(op, a, b) => {
            let a = eval(a, states)?;
            let b = eval(b, states)?;
            if matches!(op, Op::And | Op::Or) {
                let truth = if *op == Op::And {
                    and(a.truth()?, b.truth()?)
                } else {
                    or(a.truth()?, b.truth()?)
                };
                let mut reasons = a.reasons;
                reasons.extend(b.reasons);
                return Ok(boolean(truth, reasons));
            }
            let (v, reasons) = gather([Ok(a), Ok(b)])?;
            Ok(State {
                value: v
                    .map(|v| compare(op, &v[0], &v[1]).map(Value::Bool))
                    .transpose()?,
                reasons,
            })
        }
        Expr::Pure(b, args) => {
            let (v, reasons) = gather(args.iter().map(|v| eval(v, states)))?;
            Ok(State {
                value: v.map(|v| pure(b, &v)).transpose()?,
                reasons,
            })
        }
    }
}
fn compare(op: &Op, a: &Value, b: &Value) -> Result<bool, Diagnostic> {
    let ordering = crate::number::compare(a, b).or_else(|| match (a, b) {
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        _ => None,
    });
    Ok(match op {
        Op::Eq => ordering.map_or_else(|| a == b, |o| o == Ordering::Equal),
        Op::Ne => ordering.map_or_else(|| a != b, |o| o != Ordering::Equal),
        Op::Lt => {
            ordering.ok_or_else(|| error("type", "values cannot be ordered"))? == Ordering::Less
        }
        Op::Le => {
            ordering.ok_or_else(|| error("type", "values cannot be ordered"))? != Ordering::Greater
        }
        Op::Gt => {
            ordering.ok_or_else(|| error("type", "values cannot be ordered"))? == Ordering::Greater
        }
        Op::Ge => {
            ordering.ok_or_else(|| error("type", "values cannot be ordered"))? != Ordering::Less
        }
        Op::In => match (a, b) {
            (Value::String(a), Value::String(b)) => b.contains(a),
            (_, Value::List(b)) => b.iter().any(|b| {
                crate::number::compare(a, b).map_or_else(|| a == b, |o| o == Ordering::Equal)
            }),
            (Value::String(a), Value::Map(b)) => b.contains_key(a),
            _ => return Err(error("type", "invalid membership operands")),
        },
        _ => return Err(error("type", "boolean operator reached value comparator")),
    })
}
fn pure(b: &Builtin, args: &[Value]) -> Result<Value, Diagnostic> {
    match (b, args) {
        (Builtin::Length, [Value::String(s)]) => Ok(Value::integer(s.chars().count() as i128)),
        (Builtin::Length, [Value::List(v)]) => Ok(Value::integer(v.len() as i128)),
        (Builtin::Length, [Value::Map(v) | Value::Record(v)]) => {
            Ok(Value::integer(v.len() as i128))
        }
        (Builtin::Contains, [Value::String(s), Value::String(v)]) => Ok(Value::Bool(s.contains(v))),
        (Builtin::Contains, [Value::List(s), v]) => Ok(Value::Bool(s.contains(v))),
        (Builtin::StartsWith, [Value::String(s), Value::String(v)]) => {
            Ok(Value::Bool(s.starts_with(v)))
        }
        (Builtin::EndsWith, [Value::String(s), Value::String(v)]) => {
            Ok(Value::Bool(s.ends_with(v)))
        }
        (Builtin::Present, [Value::Optional(v)]) => Ok(Value::Bool(v.is_some())),
        _ => Err(error("builtin-signature", "invalid builtin arguments")),
    }
}
