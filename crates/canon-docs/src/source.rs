//! Reading Canon's own Rust source for what the documentation derives from it.
//!
//! The `protocol/1` reference and its JSON Schema are derived from the model types in
//! `crates/canon/src/model/`: their doc comments, fields, field types and serde attributes. A serde
//! attribute or a type shape this reader does not understand is an error, never a guess, so a model
//! change that would make the reference wrong stops generation instead.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use syn::parse::{ParseStream, Parser};
use syn::{
    AttrStyle, Attribute, Expr, Fields, GenericArgument, Ident, ImplItem, Item, Lit, Meta, Pat,
    PathArguments, Stmt, Type,
};

/// The directory the `protocol/1` model lives in, relative to the repository root.
pub const MODEL_DIR: &str = "crates/canon/src/model";

/// A field or payload type, as the documentation describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// `String`.
    Text,
    /// `u64`.
    Count,
    /// `Json`: any JSON value, a slot whose shape a later story settles.
    Any,
    /// One of the identifier newtypes in `ids.rs`.
    Identifier(String),
    /// A struct or enum of the model.
    Named(String),
    Option(Box<Ty>),
    List(Box<Ty>),
    Boxed(Box<Ty>),
    /// `Declarations<K, V>`: a map keyed by identifier.
    Map(Box<Ty>, Box<Ty>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub doc: String,
    pub ty: Ty,
    /// The field carries `#[serde(default)]`: leaving the key out takes the default.
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub doc: String,
    /// The single payload of a tuple variant, if any.
    pub payload: Option<Ty>,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Struct(Vec<Field>),
    Enum(Vec<Variant>),
}

/// How a type is read from a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// `#[derive(Deserialize)]` reads the fields as declared.
    Derived,
    /// `#[serde(try_from = "…")]`: the document is written as the named type.
    TryFrom(String),
    /// A hand-written `Deserialize` impl.
    Custom,
    /// Not read from a document at all.
    NotRead,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub doc: String,
    pub shape: Shape,
    pub reading: Reading,
}

/// Everything the documentation derives from the model.
#[derive(Debug, Clone)]
pub struct Model {
    pub types: BTreeMap<String, TypeDef>,
    /// The identifier newtypes and their doc comments, in declaration order.
    pub identifiers: Vec<(String, String)>,
    /// Every `*FORMAT` constant of the model and its value: `FORMAT` is `protocol/1`.
    pub formats: BTreeMap<String, String>,
    /// The doc comment of `is_identifier`: what makes an identifier well-formed.
    pub identifier_rule: String,
    /// The inner doc comment of each model file, keyed by file name.
    pub module_docs: BTreeMap<String, String>,
}

impl Model {
    pub fn get(&self, name: &str) -> Result<&TypeDef, String> {
        self.types
            .get(name)
            .ok_or_else(|| format!("the model has no type `{name}`"))
    }

    /// The value of one `*FORMAT` constant.
    pub fn format(&self, name: &str) -> Result<&str, String> {
        self.formats
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| format!("the model declares no `{name}`"))
    }

    /// The types a document rooted at `root` is written in, in the order they are first reached.
    pub fn reachable(&self, root: &str) -> Result<Vec<String>, String> {
        let mut order = Vec::new();
        let mut seen = BTreeSet::new();
        let mut queue = vec![root.to_owned()];
        while let Some(name) = queue.first().cloned() {
            queue.remove(0);
            if !seen.insert(name.clone()) {
                continue;
            }
            let def = self.get(&name)?;
            order.push(name.clone());
            let mut next = Vec::new();
            if let Reading::TryFrom(written) = &def.reading {
                next.push(written.clone());
            }
            match &def.shape {
                Shape::Struct(fields) => {
                    for field in fields {
                        named(&field.ty, &mut next);
                    }
                }
                Shape::Enum(variants) => {
                    for variant in variants {
                        if let Some(payload) = &variant.payload {
                            named(payload, &mut next);
                        }
                    }
                }
            }
            queue.extend(next);
        }
        Ok(order)
    }
}

/// Every model type a type refers to.
pub fn named(ty: &Ty, out: &mut Vec<String>) {
    match ty {
        Ty::Named(name) => out.push(name.clone()),
        Ty::Option(inner) | Ty::List(inner) | Ty::Boxed(inner) => named(inner, out),
        Ty::Map(key, value) => {
            named(key, out);
            named(value, out);
        }
        Ty::Text | Ty::Count | Ty::Any | Ty::Identifier(_) => {}
    }
}

/// The outer doc comment of an item, one source line per line, the leading space removed.
pub fn docs(attrs: &[Attribute]) -> String {
    doc_lines(attrs, |style| matches!(style, AttrStyle::Outer))
}

/// The inner (`//!`) doc comment of a file.
pub fn inner_docs(file: &syn::File) -> String {
    doc_lines(&file.attrs, |style| matches!(style, AttrStyle::Inner(_)))
}

fn doc_lines(attrs: &[Attribute], style: impl Fn(&AttrStyle) -> bool) -> String {
    let mut lines = Vec::new();
    for attr in attrs.iter().filter(|attr| style(&attr.style)) {
        if let Meta::NameValue(pair) = &attr.meta
            && pair.path.is_ident("doc")
            && let Expr::Lit(literal) = &pair.value
            && let Lit::Str(text) = &literal.lit
        {
            let text = text.value();
            lines.push(text.strip_prefix(' ').unwrap_or(&text).to_owned());
        }
    }
    lines.join("\n")
}

/// Reads and parses one Rust source file of the repository.
pub fn parse_file(root: &Path, relative: &str) -> Result<syn::File, String> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("reading {}: {error}", path.display()))?;
    syn::parse_file(&text).map_err(|error| format!("parsing {relative}: {error}"))
}

/// Whether the attributes derive `Deserialize`.
fn derives_deserialize(attrs: &[Attribute]) -> Result<bool, String> {
    let mut found = false;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("derive")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("Deserialize") {
                found = true;
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    }
    Ok(found)
}

/// The serde attributes this reader understands, collected; any other is an error.
#[derive(Default)]
struct Serde {
    default: bool,
    try_from: Option<String>,
}

fn serde(attrs: &[Attribute], owner: &str) -> Result<Serde, String> {
    let mut found = Serde::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                found.default = true;
            } else if meta.path.is_ident("deny_unknown_fields") {
            } else if meta.path.is_ident("deserialize_with") {
                let _: syn::LitStr = meta.value()?.parse()?;
            } else if meta.path.is_ident("try_from") {
                let written: syn::LitStr = meta.value()?.parse()?;
                found.try_from = Some(written.value());
            } else {
                return Err(meta.error("canon-docs does not understand this serde attribute"));
            }
            Ok(())
        })
        .map_err(|error| format!("{owner}: {error}"))?;
    }
    Ok(found)
}

fn ty(ty: &Type, identifiers: &BTreeSet<String>, owner: &str) -> Result<Ty, String> {
    let Type::Path(path) = ty else {
        return Err(format!(
            "{owner}: canon-docs does not understand this type shape"
        ));
    };
    let segment = path
        .path
        .segments
        .last()
        .ok_or_else(|| format!("{owner}: empty type path"))?;
    let name = segment.ident.to_string();
    let arguments: Vec<&Type> = match &segment.arguments {
        PathArguments::None => Vec::new(),
        PathArguments::AngleBracketed(angle) => angle
            .args
            .iter()
            .filter_map(|argument| match argument {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
            .collect(),
        PathArguments::Parenthesized(_) => {
            return Err(format!(
                "{owner}: canon-docs does not understand `{name}(…)`"
            ));
        }
    };
    let one = |wrap: fn(Box<Ty>) -> Ty| -> Result<Ty, String> {
        match arguments.as_slice() {
            [inner] => Ok(wrap(Box::new(self::ty(inner, identifiers, owner)?))),
            _ => Err(format!("{owner}: `{name}` takes one type argument")),
        }
    };
    match name.as_str() {
        "String" => Ok(Ty::Text),
        "u64" => Ok(Ty::Count),
        "Json" => Ok(Ty::Any),
        "Option" => one(Ty::Option),
        "Vec" => one(Ty::List),
        "Box" => one(Ty::Boxed),
        "Declarations" => match arguments.as_slice() {
            [key, value] => Ok(Ty::Map(
                Box::new(self::ty(key, identifiers, owner)?),
                Box::new(self::ty(value, identifiers, owner)?),
            )),
            _ => Err(format!("{owner}: `Declarations` takes two type arguments")),
        },
        _ if identifiers.contains(&name) => Ok(Ty::Identifier(name)),
        _ if arguments.is_empty() => Ok(Ty::Named(name)),
        _ => Err(format!(
            "{owner}: canon-docs does not understand the generic type `{name}`"
        )),
    }
}

/// The identifier newtypes declared with `identifier!( /// doc Name );` in `ids.rs`.
fn identifier_macros(file: &syn::File) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for item in &file.items {
        if let Item::Macro(invocation) = item
            && invocation.mac.path.is_ident("identifier")
        {
            let parser = |input: ParseStream| {
                let attrs = input.call(Attribute::parse_outer)?;
                let name: Ident = input.parse()?;
                Ok((attrs, name))
            };
            let (attrs, name) = parser
                .parse2(invocation.mac.tokens.clone())
                .map_err(|error| format!("ids.rs: identifier!: {error}"))?;
            found.push((name.to_string(), docs(&attrs)));
        }
    }
    Ok(found)
}

/// Reads the `protocol/1` model from `crates/canon/src/model/`.
pub fn model(root: &Path) -> Result<Model, String> {
    let dir = root.join(MODEL_DIR);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|error| format!("reading {}: {error}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".rs"))
        .collect();
    names.sort();
    let files: Vec<(String, syn::File)> = names
        .iter()
        .map(|name| {
            Ok((
                name.clone(),
                parse_file(root, &format!("{MODEL_DIR}/{name}"))?,
            ))
        })
        .collect::<Result<_, String>>()?;

    let mut identifiers = Vec::new();
    for (_, file) in &files {
        identifiers.extend(identifier_macros(file)?);
    }
    let identifier_names: BTreeSet<String> =
        identifiers.iter().map(|(name, _)| name.clone()).collect();

    let mut types = BTreeMap::new();
    let mut formats = BTreeMap::new();
    let mut identifier_rule = None;
    let mut module_docs = BTreeMap::new();
    let mut custom = BTreeSet::new();
    for (file_name, file) in &files {
        module_docs.insert(file_name.clone(), inner_docs(file));
        for item in &file.items {
            match item {
                Item::Fn(function) if function.sig.ident == "is_identifier" => {
                    identifier_rule = Some(docs(&function.attrs));
                }
                Item::Const(constant) if constant.ident.to_string().ends_with("FORMAT") => {
                    if let Expr::Lit(literal) = &*constant.expr
                        && let Lit::Str(text) = &literal.lit
                    {
                        formats.insert(constant.ident.to_string(), text.value());
                    }
                }
                Item::Impl(implementation) => {
                    let is_deserialize = implementation
                        .trait_
                        .as_ref()
                        .and_then(|(path, _)| path.segments.last())
                        .is_some_and(|segment| segment.ident == "Deserialize");
                    if is_deserialize
                        && let Type::Path(path) = &*implementation.self_ty
                        && let Some(segment) = path.path.segments.last()
                    {
                        custom.insert(segment.ident.to_string());
                    }
                }
                // `Declarations<K, V>` is read as `Ty::Map`; `Required<T>` is a helper. Neither is
                // a type a document is written in by name.
                Item::Struct(item) if !item.generics.params.is_empty() => {}
                Item::Struct(item) => {
                    let name = item.ident.to_string();
                    let attrs = serde(&item.attrs, &name)?;
                    let fields = match &item.fields {
                        Fields::Named(named) => named
                            .named
                            .iter()
                            .map(|field| {
                                let field_name = field
                                    .ident
                                    .as_ref()
                                    .map(ToString::to_string)
                                    .unwrap_or_default();
                                let owner = format!("{name}.{field_name}");
                                Ok(Field {
                                    doc: docs(&field.attrs),
                                    ty: ty(&field.ty, &identifier_names, &owner)?,
                                    optional: serde(&field.attrs, &owner)?.default,
                                    name: field_name,
                                })
                            })
                            .collect::<Result<Vec<_>, String>>()?,
                        // The identifier newtypes come from the macro; no other tuple struct is
                        // part of the model.
                        _ => continue,
                    };
                    let reading = reading(&item.attrs, attrs.try_from)?;
                    types.insert(
                        name.clone(),
                        TypeDef {
                            name,
                            doc: docs(&item.attrs),
                            shape: Shape::Struct(fields),
                            reading,
                        },
                    );
                }
                Item::Enum(item) => {
                    let name = item.ident.to_string();
                    let attrs = serde(&item.attrs, &name)?;
                    let variants = item
                        .variants
                        .iter()
                        .map(|variant| {
                            let owner = format!("{name}::{}", variant.ident);
                            let payload = match &variant.fields {
                                Fields::Unit => None,
                                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                                    Some(ty(&fields.unnamed[0].ty, &identifier_names, &owner)?)
                                }
                                _ => {
                                    return Err(format!(
                                        "{owner}: canon-docs reads only unit and one-field variants"
                                    ));
                                }
                            };
                            Ok(Variant {
                                name: variant.ident.to_string(),
                                doc: docs(&variant.attrs),
                                payload,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let reading = reading(&item.attrs, attrs.try_from)?;
                    types.insert(
                        name.clone(),
                        TypeDef {
                            name,
                            doc: docs(&item.attrs),
                            shape: Shape::Enum(variants),
                            reading,
                        },
                    );
                }
                _ => {}
            }
        }
    }
    for name in custom {
        if let Some(def) = types.get_mut(&name) {
            def.reading = Reading::Custom;
        }
    }
    Ok(Model {
        types,
        identifiers,
        formats,
        identifier_rule: identifier_rule.ok_or("the model declares no `is_identifier`")?,
        module_docs,
    })
}

fn reading(attrs: &[Attribute], try_from: Option<String>) -> Result<Reading, String> {
    Ok(match try_from {
        Some(written) => Reading::TryFrom(written),
        None if derives_deserialize(attrs)? => Reading::Derived,
        None => Reading::NotRead,
    })
}

/// One validation problem: its stable code and the doc comment of its variant.
#[derive(Debug, Clone)]
pub struct ProblemCode {
    pub code: String,
    pub doc: String,
}

/// The problem codes `validate` reports, read from `Problem` and `Problem::code` in
/// `crates/canon/src/validate/mod.rs`, in variant order.
pub fn problem_codes(root: &Path) -> Result<Vec<ProblemCode>, String> {
    let file = parse_file(root, "crates/canon/src/validate/mod.rs")?;
    let mut docs_by_variant = Vec::new();
    let mut codes = BTreeMap::new();
    for item in &file.items {
        match item {
            Item::Enum(problem) if problem.ident == "Problem" => {
                for variant in &problem.variants {
                    docs_by_variant.push((variant.ident.to_string(), docs(&variant.attrs)));
                }
            }
            Item::Impl(implementation) if implementation.trait_.is_none() => {
                for item in &implementation.items {
                    if let ImplItem::Fn(function) = item
                        && function.sig.ident == "code"
                    {
                        for stmt in &function.block.stmts {
                            if let Stmt::Expr(Expr::Match(matching), _) = stmt {
                                for arm in &matching.arms {
                                    let variant = match &arm.pat {
                                        Pat::Struct(pattern) => pattern.path.segments.last(),
                                        Pat::Path(pattern) => pattern.path.segments.last(),
                                        Pat::TupleStruct(pattern) => pattern.path.segments.last(),
                                        _ => None,
                                    };
                                    if let (Some(variant), Expr::Lit(literal)) =
                                        (variant, &*arm.body)
                                        && let Lit::Str(code) = &literal.lit
                                    {
                                        codes.insert(variant.ident.to_string(), code.value());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    docs_by_variant
        .into_iter()
        .map(|(variant, doc)| {
            let code = codes
                .remove(&variant)
                .ok_or_else(|| format!("Problem::{variant} has no code in Problem::code"))?;
            Ok(ProblemCode { code, doc })
        })
        .collect()
}

/// The inner doc comment of one source file of the repository.
pub fn module_doc(root: &Path, relative: &str) -> Result<String, String> {
    Ok(inner_docs(&parse_file(root, relative)?))
}
