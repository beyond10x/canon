use crate::{MAX_DEPTH, MAX_NODES, error};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub start: usize,
    pub end: usize,
    pub expression: Option<usize>,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}..{}: {}",
            self.code, self.start, self.end, self.message
        )
    }
}
impl std::error::Error for Diagnostic {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scalar {
    Bool,
    String,
    Integer,
    Decimal,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Type {
    Scalar(Scalar),
    List(Box<Type>),
    Map(Box<Type>),
    Record(BTreeMap<String, Type>),
    Optional(Box<Type>),
}
#[allow(non_upper_case_globals)]
impl Type {
    pub const Bool: Self = Self::Scalar(Scalar::Bool);
    pub const String: Self = Self::Scalar(Scalar::String);
    pub const Integer: Self = Self::Scalar(Scalar::Integer);
    pub const Decimal: Self = Self::Scalar(Scalar::Decimal);
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let mut todo = vec![(self, 0)];
        let mut count = 0;
        while let Some((t, d)) = todo.pop() {
            count += 1;
            if d > MAX_DEPTH || count > MAX_NODES {
                return Err(error("type-budget", "type is too large or deep"));
            }
            match t {
                Self::List(t) | Self::Map(t) | Self::Optional(t) => todo.push((t, d + 1)),
                Self::Record(fields) => todo.extend(fields.values().map(|v| (v, d + 1))),
                _ => {}
            }
        }
        Ok(())
    }
    pub(crate) fn numeric(&self) -> bool {
        matches!(self, Self::Scalar(Scalar::Integer | Scalar::Decimal))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exact {
    pub coefficient: String,
    pub scale: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Bool(bool),
    String(String),
    Integer(String),
    Decimal(Exact),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
    Record(BTreeMap<String, Value>),
    Optional(Option<Box<Value>>),
}
impl Value {
    pub fn integer(value: i128) -> Self {
        Self::Integer(value.to_string())
    }
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let mut todo = vec![(self, 0)];
        let mut count = 0;
        let mut bytes = 0;
        while let Some((v, d)) = todo.pop() {
            count += 1;
            if count > MAX_NODES || d > MAX_DEPTH {
                return Err(error("value-budget", "value is too large or deep"));
            }
            match v {
                Self::Integer(n) => {
                    crate::number::canonical_integer(n)?;
                }
                Self::Decimal(n) => {
                    n.validate()?;
                }
                Self::String(s) => {
                    bytes += s.len();
                }
                Self::List(v) => todo.extend(v.iter().map(|v| (v, d + 1))),
                Self::Record(v) | Self::Map(v) => {
                    bytes += v.keys().map(String::len).sum::<usize>();
                    todo.extend(v.values().map(|v| (v, d + 1)));
                }
                Self::Optional(Some(v)) => todo.push((v, d + 1)),
                _ => {}
            }
            if bytes > crate::MAX_SOURCE {
                return Err(error("value-budget", "string payload budget exceeded"));
            }
        }
        Ok(())
    }
    pub fn matches(&self, ty: &Type) -> Result<bool, Diagnostic> {
        self.validate()?;
        ty.validate()?;
        Ok(self.matches_inner(ty))
    }
    fn matches_inner(&self, ty: &Type) -> bool {
        match (self, ty) {
            (Self::Bool(_), Type::Scalar(Scalar::Bool))
            | (Self::String(_), Type::Scalar(Scalar::String))
            | (Self::Integer(_), Type::Scalar(Scalar::Integer))
            | (Self::Decimal(_), Type::Scalar(Scalar::Decimal)) => true,
            (Self::List(v), Type::List(t)) => v.iter().all(|v| v.matches_inner(t)),
            (Self::Map(v), Type::Map(t)) => v.values().all(|v| v.matches_inner(t)),
            (Self::Record(v), Type::Record(t)) => {
                v.len() == t.len()
                    && t.iter()
                        .all(|(k, t)| v.get(k).is_some_and(|v| v.matches_inner(t)))
            }
            (Self::Optional(None), Type::Optional(_)) => true,
            (Self::Optional(Some(v)), Type::Optional(t)) => v.matches_inner(t),
            _ => false,
        }
    }
    pub(crate) fn inferred(&self) -> Result<Type, Diagnostic> {
        self.validate()?;
        match self {
            Self::Bool(_) => Ok(Type::Scalar(Scalar::Bool)),
            Self::String(_) => Ok(Type::Scalar(Scalar::String)),
            Self::Integer(_) => Ok(Type::Scalar(Scalar::Integer)),
            Self::Decimal(_) => Ok(Type::Scalar(Scalar::Decimal)),
            Self::Record(v) => Ok(Type::Record(
                v.iter()
                    .map(|(k, v)| Ok((k.clone(), v.inferred()?)))
                    .collect::<Result<_, Diagnostic>>()?,
            )),
            Self::List(v) => {
                let t = v
                    .first()
                    .ok_or_else(|| {
                        error(
                            "ambiguous-type",
                            "empty list needs an explicit context type",
                        )
                    })?
                    .inferred()?;
                if !v.iter().all(|v| v.matches_inner(&t)) {
                    return Err(error("type", "heterogeneous list"));
                }
                Ok(Type::List(Box::new(t)))
            }
            Self::Map(v) => {
                let t = v
                    .values()
                    .next()
                    .ok_or_else(|| {
                        error("ambiguous-type", "empty map needs an explicit context type")
                    })?
                    .inferred()?;
                if !v.values().all(|v| v.matches_inner(&t)) {
                    return Err(error("type", "heterogeneous map"));
                }
                Ok(Type::Map(Box::new(t)))
            }
            Self::Optional(Some(v)) => Ok(Type::Optional(Box::new(v.inferred()?))),
            Self::Optional(None) => Err(error(
                "ambiguous-type",
                "optional absence needs explicit context type",
            )),
        }
    }
    pub fn from_json(value: &serde_json::Value, ty: &Type) -> Result<Self, Diagnostic> {
        ty.validate()?;
        let mut todo = vec![(value, 0)];
        let mut n = 0;
        while let Some((v, d)) = todo.pop() {
            n += 1;
            if n > MAX_NODES || d > MAX_DEPTH {
                return Err(error("value-budget", "JSON value exceeds bounds"));
            }
            match v {
                serde_json::Value::Array(a) => todo.extend(a.iter().map(|x| (x, d + 1))),
                serde_json::Value::Object(a) => todo.extend(a.values().map(|x| (x, d + 1))),
                _ => {}
            }
        }
        let result = Self::from_json_inner(value, ty)?;
        result.validate()?;
        Ok(result)
    }
    fn from_json_inner(v: &serde_json::Value, t: &Type) -> Result<Self, Diagnostic> {
        let bad = || error("value-type", format!("JSON does not match {t:?}"));
        match t {
            Type::Scalar(Scalar::Bool) => v.as_bool().map(Self::Bool).ok_or_else(bad),
            Type::Scalar(Scalar::String) => {
                v.as_str().map(|s| Self::String(s.into())).ok_or_else(bad)
            }
            Type::Scalar(Scalar::Integer) => {
                let s = v.as_number().ok_or_else(bad)?.to_string();
                let n = crate::number::parse_number(&s)?;
                match n {
                    Self::Integer(_) => Ok(n),
                    Self::Decimal(n) if n.scale == 0 => Ok(Self::Integer(n.coefficient)),
                    _ => Err(bad()),
                }
            }
            Type::Scalar(Scalar::Decimal) => {
                let s = v.as_number().ok_or_else(bad)?.to_string();
                match crate::number::parse_number(&s)? {
                    Self::Integer(s) => Ok(Self::Decimal(Exact {
                        coefficient: s,
                        scale: 0,
                    })),
                    d => Ok(d),
                }
            }
            Type::List(t) => Ok(Self::List(
                v.as_array()
                    .ok_or_else(bad)?
                    .iter()
                    .map(|v| Self::from_json_inner(v, t))
                    .collect::<Result<_, _>>()?,
            )),
            Type::Map(t) => Ok(Self::Map(
                v.as_object()
                    .ok_or_else(bad)?
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), Self::from_json_inner(v, t)?)))
                    .collect::<Result<_, Diagnostic>>()?,
            )),
            Type::Record(fields) => {
                let obj = v.as_object().ok_or_else(bad)?;
                if obj.len() != fields.len() {
                    return Err(bad());
                }
                Ok(Self::Record(
                    fields
                        .iter()
                        .map(|(k, t)| {
                            Ok((
                                k.clone(),
                                Self::from_json_inner(obj.get(k).ok_or_else(bad)?, t)?,
                            ))
                        })
                        .collect::<Result<_, Diagnostic>>()?,
                ))
            }
            Type::Optional(t) => {
                if v.is_null() {
                    Ok(Self::Optional(None))
                } else {
                    Ok(Self::Optional(Some(Box::new(Self::from_json_inner(v, t)?))))
                }
            }
        }
    }
    pub fn to_json(&self) -> Result<serde_json::Value, Diagnostic> {
        self.validate()?;
        match self {
            Self::Bool(v) => Ok((*v).into()),
            Self::String(v) => Ok(v.clone().into()),
            Self::Integer(v) => serde_json::from_str(v).map_err(|e| error("number", e.to_string())),
            Self::Decimal(v) => {
                serde_json::from_str(&v.lexeme()).map_err(|e| error("number", e.to_string()))
            }
            Self::List(v) => Ok(serde_json::Value::Array(
                v.iter().map(Self::to_json).collect::<Result<_, _>>()?,
            )),
            Self::Record(v) | Self::Map(v) => Ok(serde_json::Value::Object(
                v.iter()
                    .map(|(k, v)| Ok((k.clone(), v.to_json()?)))
                    .collect::<Result<_, Diagnostic>>()?,
            )),
            Self::Optional(None) => Ok(serde_json::Value::Null),
            Self::Optional(Some(v)) => v.to_json(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Builtin {
    Length,
    Contains,
    StartsWith,
    EndsWith,
    Present,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub provider: String,
    pub operation: String,
    pub digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Implementation {
    Observation(Provider),
    Recipe(String),
    Pure(Builtin),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub value_type: Type,
    pub default: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub parameters: Vec<Parameter>,
    pub returns: Type,
    pub implementation: Implementation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub format: String,
    pub functions: BTreeMap<String, Function>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bindings {
    pub values: BTreeMap<String, Value>,
    pub source_identity: String,
    pub context_identity: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub provider: String,
    pub operation: String,
    pub digest: String,
    pub arguments: BTreeMap<String, Value>,
    pub result_type: Type,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Outcome {
    Known(Value),
    Unavailable(String),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub request_id: String,
    pub plan_id: String,
    pub observed_at: i64,
    pub valid_until: Option<i64>,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalContext {
    pub now: i64,
    pub source_identity: String,
    pub context_identity: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truth {
    True,
    False,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub expression: String,
    pub truth: Truth,
    pub reasons: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub plan_id: String,
    pub truth: Truth,
    pub assertions: Vec<Evaluation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDocument {
    pub id: String,
    pub sources: Vec<String>,
    pub catalog: Catalog,
    pub bindings: Bindings,
    pub context_types: BTreeMap<String, Type>,
    pub requests: Vec<Request>,
}
