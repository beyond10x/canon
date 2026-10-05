//! The expression wire model uses the existing Canon handwritten-model convention.
//! Every field/type and enum/union variant must match the ESS declaration. Negative
//! controls demonstrate that additions and type substitutions are observed.
use serde_yaml_ng::Value;
use std::collections::BTreeMap;
use syn::{Fields, GenericArgument, Item, PathArguments, Type};

fn snake(name: &str) -> String {
    let mut s = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            s.push('_');
        }
        s.extend(c.to_lowercase());
    }
    s
}
fn rust_type(ty: &Type) -> String {
    let Type::Path(path) = ty else {
        panic!("unsupported model type")
    };
    let p = path.path.segments.last().unwrap();
    let name = p.ident.to_string();
    let args = if let PathArguments::AngleBracketed(a) = &p.arguments {
        a.args
            .iter()
            .filter_map(|a| {
                if let GenericArgument::Type(t) = a {
                    Some(rust_type(t))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    match name.as_str() {
        "bool" => "Boolean".into(),
        "u8" | "i64" => "Integer".into(),
        "String" => name,
        "Box" => args[0].clone(),
        "Vec" => format!("List<{}>", args[0]),
        "Option" => format!("Optional<{}>", args[0]),
        "BTreeMap" => format!("Map<{}, {}>", args[0], args[1]),
        _ => format!("canon.expr.{name}"),
    }
}
type Shape = (String, BTreeMap<String, String>);
fn rust(source: &str) -> BTreeMap<String, Shape> {
    let mut result = BTreeMap::new();
    for item in syn::parse_file(source).unwrap().items {
        match item {
            Item::Struct(s) if s.ident != "Diagnostic" => {
                let Fields::Named(fields) = s.fields else {
                    panic!("model struct needs named fields")
                };
                result.insert(
                    s.ident.to_string(),
                    (
                        "struct".into(),
                        fields
                            .named
                            .into_iter()
                            .map(|f| (f.ident.unwrap().to_string(), rust_type(&f.ty)))
                            .collect(),
                    ),
                );
            }
            Item::Enum(e) => {
                let unit = e.variants.iter().all(|v| matches!(v.fields, Fields::Unit));
                result.insert(
                    e.ident.to_string(),
                    (
                        if unit { "enum" } else { "union" }.into(),
                        e.variants
                            .into_iter()
                            .map(|v| {
                                let t = match v.fields {
                                    Fields::Unit => String::new(),
                                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                                        rust_type(&fields.unnamed.first().unwrap().ty)
                                    }
                                    _ => panic!("union requires one payload"),
                                };
                                (snake(&v.ident.to_string()), t)
                            })
                            .collect(),
                    ),
                );
            }
            _ => {}
        }
    }
    result
}
fn specification() -> BTreeMap<String, Shape> {
    let doc: Value = serde_yaml_ng::from_str(canon_expr::MODEL_SPEC).unwrap();
    doc["types"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|t| {
            let name = t["name"]
                .as_str()
                .unwrap()
                .strip_prefix("canon.expr.")
                .unwrap()
                .to_owned();
            let kind = t["kind"].as_str().unwrap();
            let fields = match kind {
                "struct" => t["fields"]
                    .as_sequence()
                    .unwrap()
                    .iter()
                    .map(|f| {
                        (
                            f["name"].as_str().unwrap().into(),
                            f["type"].as_str().unwrap().into(),
                        )
                    })
                    .collect(),
                "enum" => t["variants"]
                    .as_sequence()
                    .unwrap()
                    .iter()
                    .map(|v| (v.as_str().unwrap().into(), String::new()))
                    .collect(),
                "union" => t["variants"]
                    .as_mapping()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.as_str().unwrap().into(), v.as_str().unwrap().into()))
                    .collect(),
                _ => panic!("unsupported ESS kind"),
            };
            (name, (kind.into(), fields))
        })
        .collect()
}
#[test]
fn every_wire_declaration_matches_ess() {
    let spec = specification();
    assert_eq!(spec.len(), 19);
    assert_eq!(spec, rust(include_str!("../src/model.rs")));
}
#[test]
fn parity_detects_changed_field_type() {
    let source = include_str!("../src/model.rs");
    let altered = source.replacen(
        "pub context_identity: String",
        "pub context_identity: Option<String>",
        1,
    );
    assert_ne!(source, altered);
    assert_ne!(specification(), rust(&altered));
}
#[test]
fn parity_detects_added_variant() {
    let source = include_str!("../src/model.rs");
    let altered = source.replacen("pub enum Scalar {", "pub enum Scalar { Extra,", 1);
    assert_ne!(source, altered);
    assert_ne!(specification(), rust(&altered));
}
