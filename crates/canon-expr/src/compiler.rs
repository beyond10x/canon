use crate::parser::{Ast, Node, Op};
use crate::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Literal(Value),
    Variable(String),
    List(Vec<Typed>),
    Record(BTreeMap<String, Typed>),
    Map(BTreeMap<String, Typed>),
    Field(Box<Typed>, String),
    Index(Box<Typed>, Box<Typed>),
    Not(Box<Typed>),
    Neg(Box<Typed>),
    Binary(Op, Box<Typed>, Box<Typed>),
    Pure(Builtin, Vec<Typed>),
    Observation(Provider, BTreeMap<String, Typed>),
    Request(String),
}
#[derive(Clone, Debug)]
pub(crate) struct Typed {
    pub expr: Expr,
    pub ty: Type,
}
impl Typed {
    fn size(&self) -> Result<usize, Diagnostic> {
        let mut todo = vec![(self, 0)];
        let mut count = 0;
        while let Some((n, d)) = todo.pop() {
            count += 1;
            if count > MAX_NODES || d > MAX_DEPTH {
                return Err(error("expansion-budget", "expanded tree exceeds bounds"));
            }
            match &n.expr {
                Expr::List(v) | Expr::Pure(_, v) => todo.extend(v.iter().map(|v| (v, d + 1))),
                Expr::Record(v) | Expr::Map(v) | Expr::Observation(_, v) => {
                    todo.extend(v.values().map(|v| (v, d + 1)))
                }
                Expr::Field(v, _) | Expr::Not(v) | Expr::Neg(v) => todo.push((v, d + 1)),
                Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                    todo.push((a, d + 1));
                    todo.push((b, d + 1));
                }
                _ => {}
            }
        }
        Ok(count)
    }
    fn observed(&self) -> bool {
        match &self.expr {
            Expr::Observation(..) | Expr::Request(_) => true,
            Expr::List(v) | Expr::Pure(_, v) => v.iter().any(Self::observed),
            Expr::Record(v) | Expr::Map(v) => v.values().any(Self::observed),
            Expr::Field(v, _) | Expr::Not(v) | Expr::Neg(v) => v.observed(),
            Expr::Index(a, b) | Expr::Binary(_, a, b) => a.observed() || b.observed(),
            _ => false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CheckedExpr {
    source: String,
    catalog: Catalog,
    root: Typed,
    context_types: BTreeMap<String, Type>,
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,
    pub requests: Vec<Request>,
    pub(crate) document: PlanDocument,
    pub(crate) roots: Vec<Typed>,
}
impl Serialize for Plan {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.document.serialize(s)
    }
}
impl<'de> Deserialize<'de> for Plan {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let doc = PlanDocument::deserialize(d)?;
        let plan = compile_with_context(
            &doc.sources,
            &doc.catalog,
            &doc.bindings,
            Some(&doc.context_types),
        )
        .map_err(serde::de::Error::custom)?;
        if plan.document != doc {
            return Err(serde::de::Error::custom(
                "plan identity or requests were modified",
            ));
        }
        Ok(plan)
    }
}

fn name_valid(name: &str, dotted: bool) -> bool {
    !name.is_empty()
        && name.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .next()
                    .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        })
        && (dotted || !name.contains('.'))
}
pub fn validate_catalog(c: &Catalog) -> Result<(), Diagnostic> {
    if c.format != "canon-catalog/1" {
        return Err(error("catalog-format", "expected canon-catalog/1"));
    }
    if c.functions.len() > MAX_NODES {
        return Err(error("catalog-budget", "too many functions"));
    }
    for (name, f) in &c.functions {
        if !name_valid(name, true) {
            return Err(error("catalog-name", format!("invalid function {name}")));
        }
        f.returns.validate()?;
        let mut params = BTreeSet::new();
        if f.parameters.len() > MAX_NODES {
            return Err(error("catalog-budget", "too many parameters"));
        }
        for p in &f.parameters {
            if !name_valid(&p.name, false) || !params.insert(&p.name) {
                return Err(error(
                    "parameter",
                    format!("invalid or duplicate parameter in {name}"),
                ));
            }
            p.value_type.validate()?;
            if let Some(v) = &p.default
                && !v.matches(&p.value_type)?
            {
                return Err(error(
                    "default-type",
                    format!("default for {name}.{} has wrong type", p.name),
                ));
            }
        }
        match &f.implementation {
            Implementation::Observation(p) => {
                if p.provider.is_empty() || p.operation.is_empty() || p.digest.is_empty() {
                    return Err(error(
                        "provider-identity",
                        "provider identity, operation and digest are required",
                    ));
                }
            }
            Implementation::Recipe(s) => {
                parse(s)?;
            }
            Implementation::Pure(b) => {
                let args = f
                    .parameters
                    .iter()
                    .map(|p| &p.value_type)
                    .collect::<Vec<_>>();
                let valid = match b {
                    Builtin::Length => {
                        args.len() == 1
                            && matches!(
                                args[0],
                                Type::Scalar(Scalar::String)
                                    | Type::List(_)
                                    | Type::Map(_)
                                    | Type::Record(_)
                            )
                            && f.returns == Type::Scalar(Scalar::Integer)
                    }
                    Builtin::Contains => {
                        args.len() == 2
                            && ((args[0] == &Type::Scalar(Scalar::String)
                                && args[1] == &Type::Scalar(Scalar::String))
                                || matches!(args[0],Type::List(t) if t.as_ref()==args[1]))
                            && f.returns == Type::Scalar(Scalar::Bool)
                    }
                    Builtin::StartsWith | Builtin::EndsWith => {
                        args == vec![&Type::Scalar(Scalar::String), &Type::Scalar(Scalar::String)]
                            && f.returns == Type::Scalar(Scalar::Bool)
                    }
                    Builtin::Present => {
                        args.len() == 1
                            && matches!(args[0], Type::Optional(_))
                            && f.returns == Type::Scalar(Scalar::Bool)
                    }
                };
                if !valid {
                    return Err(error(
                        "builtin-signature",
                        format!("invalid signature for {name}"),
                    ));
                }
            }
        }
    }
    if serde_json::to_vec(c)
        .map_err(|e| error("encoding", e.to_string()))?
        .len()
        > 1024 * 1024
    {
        return Err(error("catalog-budget", "catalog exceeds 1 MiB"));
    }
    let mut checker = Checker {
        catalog: c,
        nodes: 0,
        stack: Vec::new(),
        bindings: None,
        inferred_context: BTreeMap::new(),
    };
    for (name, f) in &c.functions {
        if let Implementation::Recipe(source) = &f.implementation {
            let env = f
                .parameters
                .iter()
                .map(|p| {
                    (
                        p.name.clone(),
                        Typed {
                            expr: Expr::Variable(p.name.clone()),
                            ty: p.value_type.clone(),
                        },
                    )
                })
                .collect();
            checker.stack.push(name.clone());
            let parsed = parse(source)?;
            let root = checker.node(&parsed.root, &env, Some(&f.returns), 0)?;
            checker.stack.pop();
            expect(&root.ty, &f.returns)?;
        }
    }
    Ok(())
}
pub fn check(
    parsed: ParsedExpr,
    catalog: &Catalog,
    context_types: &BTreeMap<String, Type>,
) -> Result<CheckedExpr, Diagnostic> {
    validate_catalog(catalog)?;
    for t in context_types.values() {
        t.validate()?;
    }
    let env = context_types
        .iter()
        .map(|(k, t)| {
            (
                k.clone(),
                Typed {
                    expr: Expr::Variable(k.clone()),
                    ty: t.clone(),
                },
            )
        })
        .collect();
    let mut checker = Checker {
        catalog,
        nodes: 0,
        stack: Vec::new(),
        bindings: None,
        inferred_context: BTreeMap::new(),
    };
    let root = checker.node(&parsed.root, &env, Some(&Type::Scalar(Scalar::Bool)), 0)?;
    expect(&root.ty, &Type::Scalar(Scalar::Bool))?;
    Ok(CheckedExpr {
        source: parsed.source,
        catalog: catalog.clone(),
        root,
        context_types: context_types.clone(),
    })
}
fn expect(actual: &Type, expected: &Type) -> Result<(), Diagnostic> {
    if actual == expected {
        Ok(())
    } else {
        Err(error(
            "type",
            format!("expected {expected:?}, got {actual:?}"),
        ))
    }
}
struct Checker<'a> {
    catalog: &'a Catalog,
    nodes: usize,
    stack: Vec<String>,
    bindings: Option<&'a BTreeMap<String, Value>>,
    inferred_context: BTreeMap<String, Type>,
}
impl Checker<'_> {
    fn node(
        &mut self,
        n: &Node,
        env: &BTreeMap<String, Typed>,
        expected: Option<&Type>,
        depth: usize,
    ) -> Result<Typed, Diagnostic> {
        self.nodes += 1;
        if self.nodes > MAX_NODES || depth > MAX_DEPTH {
            return Err(error(
                "expansion-budget",
                "expanded expression is too large or deep",
            ));
        }
        let result = self.node_inner(n, env, expected, depth);
        result.map_err(|mut e| {
            if e.start == 0 && e.end == 0 {
                e.start = n.start;
                e.end = n.end;
            }
            e
        })
    }
    fn node_inner(
        &mut self,
        n: &Node,
        env: &BTreeMap<String, Typed>,
        expected: Option<&Type>,
        d: usize,
    ) -> Result<Typed, Diagnostic> {
        let (expr, ty) = match &n.ast {
            Ast::Literal(v) => (Expr::Literal(v.clone()), v.inferred()?),
            Ast::Variable(v) => {
                if let Some(value) = env.get(v) {
                    self.nodes += value.size()?;
                    if self.nodes > MAX_NODES {
                        return Err(error(
                            "expansion-budget",
                            "recipe substitution exceeds node budget",
                        ));
                    }
                    return Ok(value.clone());
                }
                if self.stack.is_empty()
                    && let Some(bindings) = self.bindings
                    && let (Some(value), Some(t)) = (bindings.get(v), expected)
                    && value.matches(t)?
                {
                    if let Some(prior) = self.inferred_context.insert(v.clone(), t.clone()) {
                        expect(t, &prior)?;
                    }
                    return Ok(Typed {
                        expr: Expr::Variable(v.clone()),
                        ty: t.clone(),
                    });
                }
                return Err(error(
                    "variable",
                    format!("undeclared or ambiguously typed variable ${v}"),
                ));
            }
            Ast::Name(name) => {
                if self.catalog.functions.contains_key(name) {
                    return self.call(name, &[], env, d);
                }
                let parts = name.split('.').collect::<Vec<_>>();
                for end in (1..parts.len()).rev() {
                    let base = parts[..end].join(".");
                    if self.catalog.functions.contains_key(&base) {
                        let mut value = self.call(&base, &[], env, d)?;
                        for field in &parts[end..] {
                            let Type::Record(fields) = &value.ty else {
                                return Err(error("field", "field access needs closed record"));
                            };
                            let ty = fields
                                .get(*field)
                                .ok_or_else(|| error("field", format!("unknown field {field}")))?
                                .clone();
                            value = Typed {
                                expr: Expr::Field(Box::new(value), (*field).into()),
                                ty,
                            };
                        }
                        return Ok(value);
                    }
                }
                return Err(error("function", format!("unknown catalog name {name}")));
            }
            Ast::Call(name, args) => return self.call(name, args, env, d),
            Ast::List(items) => {
                let expected = match expected {
                    Some(Type::List(t)) => Some(t.as_ref()),
                    _ => None,
                };
                let mut values = Vec::new();
                let mut element = expected.cloned();
                for item in items {
                    let v = self.node(item, env, element.as_ref(), d + 1)?;
                    if let Some(t) = &element {
                        expect(&v.ty, t)?;
                    } else {
                        element = Some(v.ty.clone());
                    }
                    values.push(v);
                }
                let element = element
                    .ok_or_else(|| error("ambiguous-type", "empty list needs a context type"))?;
                (Expr::List(values), Type::List(Box::new(element)))
            }
            Ast::Record(fields) => {
                if let Some(Type::Map(t)) = expected {
                    let mut values = BTreeMap::new();
                    for (k, v) in fields {
                        let v = self.node(v, env, Some(t), d + 1)?;
                        expect(&v.ty, t)?;
                        values.insert(k.clone(), v);
                    }
                    (Expr::Map(values), Type::Map(t.clone()))
                } else {
                    let mut types = BTreeMap::new();
                    let mut values = BTreeMap::new();
                    for (k, v) in fields {
                        let expected = match expected {
                            Some(Type::Record(t)) => t.get(k),
                            _ => None,
                        };
                        let v = self.node(v, env, expected, d + 1)?;
                        types.insert(k.clone(), v.ty.clone());
                        values.insert(k.clone(), v);
                    }
                    (Expr::Record(values), Type::Record(types))
                }
            }
            Ast::Field(value, field) => {
                let v = self.node(value, env, None, d + 1)?;
                let Type::Record(fields) = &v.ty else {
                    return Err(error("field", "field access needs closed record"));
                };
                let ty = fields
                    .get(field)
                    .ok_or_else(|| error("field", format!("unknown field {field}")))?
                    .clone();
                (Expr::Field(Box::new(v), field.clone()), ty)
            }
            Ast::Index(value, index) => {
                let v = self.node(value, env, None, d + 1)?;
                let (itype, ty) = match &v.ty {
                    Type::List(t) => (Type::Scalar(Scalar::Integer), t.as_ref().clone()),
                    Type::Map(t) => (Type::Scalar(Scalar::String), t.as_ref().clone()),
                    _ => return Err(error("index", "indexing needs List or Map")),
                };
                let i = self.node(index, env, Some(&itype), d + 1)?;
                expect(&i.ty, &itype)?;
                (Expr::Index(Box::new(v), Box::new(i)), ty)
            }
            Ast::Not(v) => {
                let v = self.node(v, env, Some(&Type::Scalar(Scalar::Bool)), d + 1)?;
                expect(&v.ty, &Type::Scalar(Scalar::Bool))?;
                (Expr::Not(Box::new(v)), Type::Scalar(Scalar::Bool))
            }
            Ast::Neg(v) => {
                let v = self.node(v, env, None, d + 1)?;
                if !v.ty.numeric() {
                    return Err(error("type", "unary minus requires numeric value"));
                }
                let ty = v.ty.clone();
                (Expr::Neg(Box::new(v)), ty)
            }
            Ast::Binary(op, a, b) => {
                let boolop = matches!(op, Op::And | Op::Or);
                let a = self.node(
                    a,
                    env,
                    if boolop {
                        Some(&Type::Scalar(Scalar::Bool))
                    } else {
                        None
                    },
                    d + 1,
                )?;
                let b = self.node(
                    b,
                    env,
                    if boolop {
                        Some(&Type::Scalar(Scalar::Bool))
                    } else if *op != Op::In {
                        Some(&a.ty)
                    } else {
                        None
                    },
                    d + 1,
                )?;
                match op {
                    Op::And | Op::Or => {
                        expect(&a.ty, &Type::Scalar(Scalar::Bool))?;
                        expect(&b.ty, &Type::Scalar(Scalar::Bool))?;
                    }
                    Op::In => match &b.ty {
                        Type::List(t) => expect(&a.ty, t)?,
                        Type::Map(_) => expect(&a.ty, &Type::Scalar(Scalar::String))?,
                        Type::Scalar(Scalar::String) => {
                            expect(&a.ty, &Type::Scalar(Scalar::String))?
                        }
                        _ => return Err(error("type", "in requires a list, map or string")),
                    },
                    Op::Eq | Op::Ne => {
                        if !(a.ty.numeric() && b.ty.numeric()) {
                            expect(&a.ty, &b.ty)?;
                        }
                    }
                    _ => {
                        if !(a.ty.numeric() && b.ty.numeric()
                            || a.ty == Type::Scalar(Scalar::String)
                                && b.ty == Type::Scalar(Scalar::String))
                        {
                            return Err(error("type", "ordering needs two numbers or two strings"));
                        }
                    }
                };
                (
                    Expr::Binary(op.clone(), Box::new(a), Box::new(b)),
                    Type::Scalar(Scalar::Bool),
                )
            }
        };
        let result = Typed { expr, ty };
        result.size()?;
        Ok(result)
    }
    fn call(
        &mut self,
        name: &str,
        args: &[(Option<String>, Node)],
        env: &BTreeMap<String, Typed>,
        d: usize,
    ) -> Result<Typed, Diagnostic> {
        let f = self
            .catalog
            .functions
            .get(name)
            .ok_or_else(|| error("function", format!("unknown function {name}")))?
            .clone();
        let mut values = BTreeMap::new();
        let mut pos = 0;
        let mut named = false;
        for (key, arg) in args {
            let param = if let Some(key) = key {
                named = true;
                f.parameters
                    .iter()
                    .find(|p| &p.name == key)
                    .ok_or_else(|| error("argument", format!("unknown argument {key}")))?
            } else {
                if named {
                    return Err(error(
                        "argument",
                        "positional argument after named argument",
                    ));
                }
                let param = f
                    .parameters
                    .get(pos)
                    .ok_or_else(|| error("argument", "too many arguments"))?;
                pos += 1;
                param
            };
            if values.contains_key(&param.name) {
                return Err(error(
                    "argument",
                    format!("duplicate argument {}", param.name),
                ));
            }
            let value = self.node(arg, env, Some(&param.value_type), d + 1)?;
            expect(&value.ty, &param.value_type)?;
            values.insert(param.name.clone(), value);
        }
        for p in &f.parameters {
            if !values.contains_key(&p.name) {
                let v = p
                    .default
                    .clone()
                    .ok_or_else(|| error("argument", format!("missing argument {}", p.name)))?;
                values.insert(
                    p.name.clone(),
                    Typed {
                        expr: Expr::Literal(v),
                        ty: p.value_type.clone(),
                    },
                );
            }
        }
        let expr = match &f.implementation {
            Implementation::Observation(provider) => {
                if values.values().any(Typed::observed) {
                    return Err(error(
                        "acquisition-dependency",
                        "observation arguments cannot depend on observations",
                    ));
                }
                Expr::Observation(provider.clone(), values)
            }
            Implementation::Pure(b) => Expr::Pure(
                b.clone(),
                f.parameters
                    .iter()
                    .map(|p| values[&p.name].clone())
                    .collect(),
            ),
            Implementation::Recipe(source) => {
                if self.stack.iter().any(|s| s == name) {
                    return Err(error("recipe-cycle", format!("recursive recipe {name}")));
                }
                if self.stack.len() >= 128 {
                    return Err(error("recipe-budget", "recipe depth exceeded"));
                }
                self.stack.push(name.into());
                let parsed = parse(source)?;
                let result = self.node(&parsed.root, &values, Some(&f.returns), d + 1);
                self.stack.pop();
                let root = result?;
                expect(&root.ty, &f.returns)?;
                return Ok(root);
            }
        };
        Ok(Typed {
            expr,
            ty: f.returns,
        })
    }
}
pub fn compile(
    sources: &[String],
    catalog: &Catalog,
    bindings: &Bindings,
) -> Result<Plan, Diagnostic> {
    compile_with_context(sources, catalog, bindings, None)
}
fn compile_with_context(
    sources: &[String],
    catalog: &Catalog,
    bindings: &Bindings,
    context_types: Option<&BTreeMap<String, Type>>,
) -> Result<Plan, Diagnostic> {
    validate_catalog(catalog)?;
    if sources.is_empty() || sources.len() > MAX_ASSERTIONS {
        return Err(error("assertions-budget", "expected 1..1024 assertions"));
    }
    validate_bindings(bindings)?;
    if sources.iter().map(String::len).sum::<usize>() > MAX_SOURCE {
        return Err(error("source-budget", "combined sources exceed 64 KiB"));
    }
    let mut types = bindings
        .values
        .iter()
        .filter_map(|(k, v)| v.inferred().ok().map(|t| (k.clone(), t)))
        .collect::<BTreeMap<_, _>>();
    if let Some(explicit) = context_types {
        for t in explicit.values() {
            t.validate()?;
        }
        types = explicit.clone();
    }
    let env = types
        .iter()
        .map(|(k, t)| {
            (
                k.clone(),
                Typed {
                    expr: Expr::Variable(k.clone()),
                    ty: t.clone(),
                },
            )
        })
        .collect();
    let mut checked = Vec::new();
    let mut checker = Checker {
        catalog,
        nodes: 0,
        stack: Vec::new(),
        bindings: Some(&bindings.values),
        inferred_context: BTreeMap::new(),
    };
    for (i, source) in sources.iter().enumerate() {
        let parsed = parse(source).map_err(|mut e| {
            e.expression = Some(i);
            e
        })?;
        let root = checker
            .node(&parsed.root, &env, Some(&Type::Scalar(Scalar::Bool)), 0)
            .and_then(|root| {
                expect(&root.ty, &Type::Scalar(Scalar::Bool))?;
                Ok(root)
            })
            .map_err(|mut e| {
                e.expression = Some(i);
                e
            })?;
        checked.push(root);
    }
    types.extend(checker.inferred_context);
    build_plan(sources.to_vec(), catalog.clone(), checked, bindings, types)
}
fn validate_bindings(bindings: &Bindings) -> Result<(), Diagnostic> {
    if bindings.source_identity.is_empty() || bindings.context_identity.is_empty() {
        return Err(error(
            "identity",
            "source and context identities are required",
        ));
    }
    if bindings.values.len() > MAX_NODES {
        return Err(error("bindings-budget", "too many bindings"));
    }
    for v in bindings.values.values() {
        v.validate()?;
    }
    Ok(())
}
pub fn plan(checked: CheckedExpr, bindings: &Bindings) -> Result<Plan, Diagnostic> {
    validate_bindings(bindings)?;
    build_plan(
        vec![checked.source],
        checked.catalog,
        vec![checked.root],
        bindings,
        checked.context_types,
    )
}
fn build_plan(
    sources: Vec<String>,
    catalog: Catalog,
    roots: Vec<Typed>,
    bindings: &Bindings,
    context_types: BTreeMap<String, Type>,
) -> Result<Plan, Diagnostic> {
    let id = digest(&(LANGUAGE, &sources, &catalog, bindings, &context_types))?;
    let mut requests = BTreeMap::new();
    let mut payload = 0;
    let roots = roots
        .into_iter()
        .map(|root| bind(root, bindings, &mut requests, &mut payload))
        .collect::<Result<_, _>>()?;
    let requests = requests.into_values().collect::<Vec<_>>();
    let document = PlanDocument {
        id: id.clone(),
        sources,
        catalog,
        bindings: bindings.clone(),
        context_types,
        requests: requests.clone(),
    };
    Ok(Plan {
        id,
        requests,
        document,
        roots,
    })
}
fn bind(
    node: Typed,
    bindings: &Bindings,
    requests: &mut BTreeMap<String, Request>,
    payload: &mut usize,
) -> Result<Typed, Diagnostic> {
    let ty = node.ty;
    let expr = match node.expr {
        Expr::Variable(name) => {
            let value = bindings
                .values
                .get(&name)
                .ok_or_else(|| error("binding", format!("missing ${name}")))?;
            account(value, payload)?;
            if !value.matches(&ty)? {
                return Err(error(
                    "binding-type",
                    format!("${name} has wrong value type"),
                ));
            }
            Expr::Literal(value.clone())
        }
        Expr::Observation(provider, args) => {
            let args = args
                .into_iter()
                .map(|(k, v)| {
                    let v = bind(v, bindings, requests, payload)?;
                    let value = crate::eval::known(&v)?;
                    Ok((k, value))
                })
                .collect::<Result<BTreeMap<_, _>, Diagnostic>>()?;
            Value::Record(args.clone()).validate()?;
            let id = digest(&(
                LANGUAGE,
                &provider,
                &args,
                &ty,
                &bindings.source_identity,
                &bindings.context_identity,
            ))?;
            let request = Request {
                id: id.clone(),
                provider: provider.provider,
                operation: provider.operation,
                digest: provider.digest,
                arguments: args,
                result_type: ty.clone(),
            };
            requests.insert(id.clone(), request);
            if requests.len() > MAX_REQUESTS {
                return Err(error("request-budget", "too many requests"));
            }
            Expr::Request(id)
        }
        Expr::List(v) => Expr::List(
            v.into_iter()
                .map(|v| bind(v, bindings, requests, payload))
                .collect::<Result<_, _>>()?,
        ),
        Expr::Record(v) => Expr::Record(
            v.into_iter()
                .map(|(k, v)| Ok((k, bind(v, bindings, requests, payload)?)))
                .collect::<Result<_, Diagnostic>>()?,
        ),
        Expr::Map(v) => Expr::Map(
            v.into_iter()
                .map(|(k, v)| Ok((k, bind(v, bindings, requests, payload)?)))
                .collect::<Result<_, Diagnostic>>()?,
        ),
        Expr::Pure(b, v) => Expr::Pure(
            b,
            v.into_iter()
                .map(|v| bind(v, bindings, requests, payload))
                .collect::<Result<_, _>>()?,
        ),
        Expr::Field(v, k) => Expr::Field(Box::new(bind(*v, bindings, requests, payload)?), k),
        Expr::Index(a, b) => Expr::Index(
            Box::new(bind(*a, bindings, requests, payload)?),
            Box::new(bind(*b, bindings, requests, payload)?),
        ),
        Expr::Not(v) => Expr::Not(Box::new(bind(*v, bindings, requests, payload)?)),
        Expr::Neg(v) => Expr::Neg(Box::new(bind(*v, bindings, requests, payload)?)),
        Expr::Binary(op, a, b) => Expr::Binary(
            op,
            Box::new(bind(*a, bindings, requests, payload)?),
            Box::new(bind(*b, bindings, requests, payload)?),
        ),
        Expr::Literal(value) => {
            account(&value, payload)?;
            Expr::Literal(value)
        }
        other => other,
    };
    Ok(Typed { expr, ty })
}

fn account(value: &Value, total: &mut usize) -> Result<(), Diagnostic> {
    value.validate()?;
    *total += serde_json::to_vec(value)
        .map_err(|e| error("encoding", e.to_string()))?
        .len();
    if *total > MAX_BOUND_BYTES {
        return Err(error("bound-value-budget", "bound values exceed 1 MiB"));
    }
    Ok(())
}
