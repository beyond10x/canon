//! Canon's ESS specification and its hand-written Rust model stay equal.
//!
//! The model under `crates/canon/src/model/` — the `protocol/1` source model and the
//! `canon-case/1`, `canon-evidence/1`, `canon-authority/1`, `canon-decisions/1`,
//! `canon-decision/1` and `canon-properties/1` documents — is
//! hand-written beside its specification under `ess/`. This test compiles the specification (`ess
//! specify compile --format json`) and reads the model's Rust source, then compares, for every
//! entity and value type the specification declares, its fields and their types and every enum or
//! union variant. A model change that leaves `ess/` behind fails here, naming what differs.
//!
//! How the Rust model maps onto the specification:
//!
//! * A specification type `canon.protocol.X` or `canon.check.X` (the [`DOMAINS`]) is the Rust type
//!   `X`; two domains declaring one local name fail the test.
//! * A type made by a `macro_rules!` macro that defines `struct $name(T)` — `identifier!(X)` — is a
//!   `newtype` of `T`, read from the macro body; a struct with named fields is a `struct`; an enum
//!   of unit variants is an `enum`; an enum of one-payload variants is a `union`. A variant's
//!   specification name is its Rust name in snake case.
//! * `String` is `String`; `i64` is `Integer`; `u64` is an `Integer` field whose owner states the
//!   invariant `<field> >= 0`, or one [`NON_NEGATIVE`] lists; `Option<T>` is `Optional<T>`,
//!   `Vec<T>` is `List<T>`, `Box<T>` is `T`, `Declarations<K, V>` is `Map<K, V>` and the model's
//!   `Json` is `Json`. Any other Rust type matches nothing.
//! * Each entity is the Rust struct [`ENTITIES`] names. The entity `Protocol` is the Rust struct
//!   `Protocol` with its `protocol: ProtocolHeader` field written in place: the header's `id` is
//!   the entity's identity `protocol_id`, and its other fields are entity fields. The entity `Case`
//!   is the Rust struct `Case`: its `id` field is the identity `id`, its other fields are entity
//!   fields. An entity the specification declares and [`ENTITIES`] does not map fails the test.
//! * The value types compared are those the entities reach, and those [`DOCUMENTS`] reach: the
//!   evidence record, the authority decision, the explicit decision, the decision and the
//!   properties, which no entity holds.
//! * Only module-level items count. Items inside functions, inline modules (`mod tests { … }`) and
//!   anything under `#[cfg(test)]` are not the model. Two module-level types with one name, in any
//!   model files, fail the test rather than one shadowing the other.
//!
//! ess 0.52.0 compiles a map key to its primitive (`string`), so the key type is read from the
//! authored `ess/domains/*.yaml`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_yaml_ng::Value;

/// Every domain the specification declares whose types are the Rust model. A type's local name is
/// its name without the domain, and is unique across them.
const DOMAINS: &[&str] = &["canon.protocol", "canon.check"];

/// How one entity of the specification is written in the Rust model.
struct EntityMapping {
    /// The entity's local name, which is also the Rust struct's name.
    name: &'static str,
    /// The Rust field whose struct is written in place, if any.
    header: Option<&'static str>,
    /// The Rust field that is the entity's identity (in the header, when there is one), and the
    /// identity's specification name.
    identity: (&'static str, &'static str),
}

/// Every entity the specification declares.
const ENTITIES: &[EntityMapping] = &[
    EntityMapping {
        name: "Protocol",
        header: Some("protocol"),
        identity: ("id", "protocol_id"),
    },
    EntityMapping {
        name: "Case",
        header: None,
        identity: ("id", "id"),
    },
];

/// Value types no entity holds that are still part of the model: the evidence record, the
/// authority decision (`canon-authority/1`) and the explicit decision (`canon-decisions/1`) an
/// evaluation reads, the decision it writes, and the properties (`canon-properties/1`) `canon
/// check` reads. They, and what they reach, are compared too.
const DOCUMENTS: &[&str] = &[
    "EvidenceRecord",
    "AuthorityDecision",
    "ExplicitDecision",
    "Decision",
    "Properties",
];

/// `(owner, field)` Integer fields that are never negative although `ess/` cannot say so: ess
/// 0.52.0 refuses an invariant on a struct type no view publishes (ESS-SYNTH-013), and this
/// specification has no views. Each is read as if its owner stated `<field> >= 0`, so its Rust
/// type must be `u64`. Remove an entry once `ess/` can state the bound itself.
const NON_NEGATIVE: &[(&str, &str)] = &[("Decision", "protocol_revision")];

/// The tree under test, read at run time: a test binary reused from a shared target directory
/// would otherwise check the tree it was built from.
fn repo_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect(
        "CARGO_MANIFEST_DIR is unset: run this test through cargo, which sets it to the \
         crates/canon directory of the tree under test",
    );
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

// ---------------------------------------------------------------------------------------------
// One type language for both sides.

#[derive(Debug, Clone, PartialEq, Eq)]
enum Ty {
    Primitive(String),
    Declared(String),
    Optional(Box<Ty>),
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Primitive(name) | Ty::Declared(name) => f.write_str(name),
            Ty::Optional(of) => write!(f, "Optional<{of}>"),
            Ty::List(of) => write!(f, "List<{of}>"),
            Ty::Map(key, value) => write!(f, "Map<{key}, {value}>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Shape {
    Newtype(Ty),
    Struct(Vec<(String, Ty)>),
    Enum(Vec<String>),
    Union(Vec<(String, Ty)>),
}

impl Shape {
    fn kind(&self) -> &'static str {
        match self {
            Shape::Newtype(_) => "newtype",
            Shape::Struct(_) => "struct",
            Shape::Enum(_) => "enum",
            Shape::Union(_) => "union",
        }
    }
}

/// One entity: its identity and its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entity {
    identity: (String, Ty),
    fields: Vec<(String, Ty)>,
}

/// One side of the comparison: every entity by name, and every value type.
#[derive(Debug, Clone)]
struct Model {
    entities: BTreeMap<String, Entity>,
    types: BTreeMap<String, Shape>,
}

// ---------------------------------------------------------------------------------------------
// The specification side: compiled IR, with map keys from the authored YAML.

fn compiled_ir(root: &Path) -> Value {
    let args = ["specify", "compile", "--path", "ess", "--format", "json"];
    let output = match Command::new("ess").args(args).current_dir(root).output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => panic!(
            "`ess` is not on PATH: comparing the specification with the model needs ess 0.52.0, \
             the release ess/ess-inputs.yaml pins"
        ),
        Err(error) => panic!("cannot run `ess {}`: {error}", args.join(" ")),
    };
    assert!(
        output.status.success(),
        "`ess {}` exited {}:\n{}",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_yaml_ng::from_slice(&output.stdout).expect("compiled IR is JSON")
}

/// Map key types as authored: `(owner, field) -> key`, for every `Map<K, V>` field.
fn authored_map_keys(root: &Path) -> BTreeMap<(String, String), String> {
    let mut keys = BTreeMap::new();
    let spec = root.join("ess");
    let inputs =
        fs::read_to_string(spec.join("ess-inputs.yaml")).expect("ess/ess-inputs.yaml reads");
    let inputs: Value = serde_yaml_ng::from_str(&inputs).expect("ess/ess-inputs.yaml is YAML");
    let listed: Vec<&str> = inputs["specification"]
        .as_sequence()
        .expect("ess/ess-inputs.yaml lists its `specification:` files")
        .iter()
        .map(|file| file.as_str().expect("a listed file is a path"))
        .collect();
    // Only the files ess compiles: an unlisted file under ess/ is not the specification.
    for file in listed {
        let path = spec.join(file);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("listed file {}: {error}", path.display()));
        let doc: Value = serde_yaml_ng::from_str(&text).expect("domain file is YAML");
        for section in ["types", "entities"] {
            for declaration in doc[section].as_sequence().into_iter().flatten() {
                let owner = local(declaration["name"].as_str().unwrap_or_default());
                for field in declaration["fields"].as_sequence().into_iter().flatten() {
                    let ty = field["type"].as_str().unwrap_or_default().trim();
                    if let Some(args) = ty.strip_prefix("Map<").and_then(|t| t.strip_suffix('>')) {
                        let key = split_top(args)[0].trim().to_owned();
                        let name = field["name"].as_str().unwrap_or_default().to_owned();
                        keys.insert((owner.clone(), name), key);
                    }
                }
            }
        }
    }
    keys
}

/// `X` for a name `<domain>.X` of one of [`DOMAINS`], and `None` for any other name.
fn in_model(name: &str) -> Option<&str> {
    DOMAINS
        .iter()
        .find_map(|domain| name.strip_prefix(&format!("{domain}.")))
}

/// `canon.protocol.X` and `canon.check.X` are `X`; a primitive is its lower-case name.
fn local(name: &str) -> String {
    in_model(name).unwrap_or(name).to_owned()
}

fn ir_ty(value: &Value, map_key: Option<&String>) -> Ty {
    let name = || local(value["name"].as_str().expect("type ref has a name"));
    match value["kind"].as_str().expect("type ref has a kind") {
        "primitive" => Ty::Primitive(name()),
        "declared" => Ty::Declared(name()),
        "optional" => Ty::Optional(Box::new(ir_ty(&value["of"], None))),
        "list" => Ty::List(Box::new(ir_ty(&value["of"], None))),
        "map" => {
            let compiled = value["key"].as_str().expect("map key").to_owned();
            let key = match map_key.and_then(|authored| in_model(authored)) {
                Some(declared) => Ty::Declared(declared.to_owned()),
                None => Ty::Primitive(compiled),
            };
            Ty::Map(Box::new(key), Box::new(ir_ty(&value["value"], None)))
        }
        other => panic!("type ref kind `{other}` has no Rust mapping in this test"),
    }
}

/// The fields of a struct or entity whose compiled declaration is `owner_ir`. An `Integer` field
/// the owner bounds with the invariant `<field> >= 0`, or that [`NON_NEGATIVE`] lists, is a
/// non-negative integer.
fn ir_fields(
    owner: &str,
    owner_ir: &Value,
    keys: &BTreeMap<(String, String), String>,
) -> Vec<(String, Ty)> {
    let invariants: BTreeSet<String> = owner_ir["invariants"]
        .as_sequence()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|invariant| invariant.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    owner_ir["fields"]
        .as_sequence()
        .expect("fields are a list")
        .iter()
        .map(|field| {
            let name = field["name"].as_str().expect("field name").to_owned();
            let key = keys.get(&(owner.to_owned(), name.clone()));
            let mut ty = ir_ty(&field["type_ref"], key);
            let listed = NON_NEGATIVE.contains(&(owner, name.as_str()));
            if ty == integer() && (listed || invariants.contains(&format!("{name} >= 0"))) {
                ty = non_negative_integer();
            }
            (name, ty)
        })
        .collect()
}

fn integer() -> Ty {
    Ty::Primitive("integer".to_owned())
}

fn non_negative_integer() -> Ty {
    Ty::Primitive("integer >= 0".to_owned())
}

fn specification() -> Model {
    specification_at(&repo_root())
}

/// The specification under `root`/ess.
fn specification_at(root: &Path) -> Model {
    let ir = compiled_ir(root);
    let keys = authored_map_keys(root);
    let mut entities = BTreeMap::new();
    let mut state_types = BTreeSet::new();
    for (name, entity) in ir["entities"].as_mapping().expect("entities map") {
        let name = local(name.as_str().expect("entity name"));
        let identity = (
            entity["identity"]["name"]
                .as_str()
                .expect("identity name")
                .to_owned(),
            ir_ty(&entity["identity"]["type_ref"], None),
        );
        let fields = ir_fields(&name, entity, &keys);
        if let Some(state_type) = entity["state_type"].as_str() {
            state_types.insert(state_type.to_owned());
        }
        entities.insert(name, Entity { identity, fields });
    }

    let mut types = BTreeMap::new();
    for (name, declaration) in ir["types"].as_mapping().expect("types map") {
        let name = name.as_str().expect("type name");
        if state_types.contains(name) || in_model(name).is_none() {
            continue;
        }
        let qualified = name;
        let name = local(name);
        assert!(
            !types.contains_key(&name),
            "two domains declare a type `{name}` (one is `{qualified}`): the Rust model has one \
             namespace, so local names must be unique across {DOMAINS:?}"
        );
        let body = &declaration["body"];
        let shape = match body["kind"].as_str().expect("type kind") {
            "newtype" => Shape::Newtype(ir_ty(&body["of"], None)),
            "struct" => Shape::Struct(ir_fields(&name, body, &keys)),
            "enum" => Shape::Enum(
                body["variants"]
                    .as_sequence()
                    .expect("enum variants")
                    .iter()
                    .map(|v| v.as_str().expect("variant").to_owned())
                    .collect(),
            ),
            "union" => Shape::Union(
                body["variants"]
                    .as_mapping()
                    .expect("union variants")
                    .iter()
                    .map(|(v, ty)| (v.as_str().expect("variant").to_owned(), ir_ty(ty, None)))
                    .collect(),
            ),
            other => panic!("type kind `{other}` of {name} has no Rust mapping in this test"),
        };
        types.insert(name, shape);
    }
    Model { entities, types }
}

// ---------------------------------------------------------------------------------------------
// The Rust side: the model's source text.

fn model_sources() -> Vec<(String, String)> {
    let dir = repo_root().join("crates/canon/src/model");
    let mut sources: Vec<(String, String)> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .map(|path| {
            let text = fs::read_to_string(&path).expect("model source reads");
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, text)
        })
        .collect();
    sources.sort();
    sources
}

/// The source with every comment, and the contents of every string and character literal, blanked
/// to spaces, so a brace, keyword or `#[` inside them is not read as code. Every blanked byte
/// becomes one space, so the result is ASCII wherever it was blanked and offsets are unchanged.
fn mask(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let blank = |out: &mut Vec<u8>, from: usize, to: usize| {
        for byte in &mut out[from..to.min(bytes.len())] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    };
    let mut i = 0;
    while i < bytes.len() {
        let rest = &bytes[i..];
        if rest.starts_with(b"//") {
            let end = rest
                .iter()
                .position(|&b| b == b'\n')
                .map_or(bytes.len(), |p| i + p);
            blank(&mut out, i, end);
            i = end;
        } else if rest.starts_with(b"/*") {
            let mut depth = 0usize;
            let mut j = i;
            while j < bytes.len() {
                if bytes[j..].starts_with(b"/*") {
                    depth += 1;
                    j += 2;
                } else if bytes[j..].starts_with(b"*/") {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            blank(&mut out, i, j);
            i = j;
        } else if rest[0] == b'r'
            && (i == 0 || !is_ident_char(bytes[i - 1] as char))
            && rest[1..]
                .iter()
                .position(|&b| b != b'#')
                .map(|p| rest[1 + p])
                == Some(b'"')
        {
            let hashes = rest[1..].iter().take_while(|&&b| b == b'#').count();
            let open = i + 2 + hashes;
            let mut close = vec![b'"'];
            close.extend(std::iter::repeat_n(b'#', hashes));
            let end = bytes[open..]
                .windows(close.len())
                .position(|w| w == close.as_slice())
                .map_or(bytes.len(), |p| open + p);
            blank(&mut out, open, end);
            i = end + close.len();
        } else if rest[0] == b'"' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'"' {
                j += if bytes[j] == b'\\' { 2 } else { 1 };
            }
            blank(&mut out, i + 1, j);
            i = j + 1;
        } else if rest[0] == b'\'' {
            // A character literal, or a lifetime (which has no closing quote).
            let end = if rest.get(1) == Some(&b'\\') {
                // Skip the escaped character itself, which may be a quote: '\''.
                rest.get(3..)
                    .and_then(|tail| tail.iter().position(|&b| b == b'\''))
                    .map(|p| i + 3 + p)
            } else {
                let width = source[i + 1..].chars().next().map_or(1, char::len_utf8);
                (rest.get(1 + width) == Some(&b'\'')).then_some(i + 1 + width)
            };
            match end {
                Some(end) => {
                    blank(&mut out, i + 1, end);
                    i = end + 1;
                }
                None => i += 1,
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("blanking keeps UTF-8")
}

/// The brace depth at byte `at` of masked source: 0 is module level.
fn depth_at(masked: &str, at: usize) -> usize {
    let opened = masked[..at].bytes().filter(|&b| b == b'{').count();
    let closed = masked[..at].bytes().filter(|&b| b == b'}').count();
    opened.saturating_sub(closed)
}

/// Masked source with every module-level item under `#[cfg(test)]` blanked.
fn without_test_items(masked: &str) -> String {
    let mut out = masked.as_bytes().to_vec();
    let mut from = 0;
    while let Some(at) = masked[from..].find("#[cfg(test)]").map(|p| from + p) {
        let after = at + "#[cfg(test)]".len();
        let end = match masked[after..].find(['{', ';']).map(|p| after + p) {
            Some(open) if masked.as_bytes()[open] == b'{' => {
                matching(masked, open, '{', '}').expect("test item closes") + 1
            }
            Some(semicolon) => semicolon + 1,
            None => masked.len(),
        };
        for byte in &mut out[at..end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
        from = end;
    }
    String::from_utf8(out).expect("blanking whole characters keeps UTF-8")
}

/// Removes `#[...]` attributes.
fn strip_attributes(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("#[") {
        out.push_str(&rest[..start]);
        let end = matching(rest, start + 1, '[', ']').expect("attribute closes");
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

/// The index of the bracket closing the one at `open`.
fn matching(text: &str, open: usize, left: char, right: char) -> Option<usize> {
    let mut depth = 0usize;
    for (index, c) in text[open..].char_indices() {
        if c == left {
            depth += 1;
        } else if c == right {
            depth -= 1;
            if depth == 0 {
                return Some(open + index);
            }
        }
    }
    None
}

/// Splits on commas outside `<>`, `()` and `[]`, dropping empty items.
fn split_top(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for c in text.chars() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                items.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    items.push(current);
    items
        .into_iter()
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Every module-level `<keyword> Name<generics>? { body }` in masked `source`, as `(Name, body)`.
fn items<'a>(source: &'a str, keyword: &str) -> Vec<(String, &'a str)> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = source[from..].find(keyword) {
        let start = from + at;
        from = start + keyword.len();
        let before_ok = source[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !is_ident_char(c));
        let after = &source[from..];
        if !before_ok || !after.starts_with(char::is_whitespace) || depth_at(source, start) != 0 {
            continue;
        }
        let after = after.trim_start();
        let name: String = after.chars().take_while(|c| is_ident_char(*c)).collect();
        if name.is_empty() {
            continue;
        }
        let mut rest_at = source.len() - after.len() + name.len();
        let rest = source[rest_at..].trim_start();
        rest_at = source.len() - rest.len();
        let rest_at = if rest.starts_with('<') {
            let close = matching(source, rest_at, '<', '>').expect("generics close");
            let tail = source[close + 1..].trim_start();
            source.len() - tail.len()
        } else {
            rest_at
        };
        if !source[rest_at..].starts_with('{') {
            continue;
        }
        let close = matching(source, rest_at, '{', '}').expect("body closes");
        found.push((name, &source[rest_at + 1..close]));
        from = close;
    }
    found
}

/// Every module-level `macro_rules!` in masked `source` whose body defines `struct $name(T)`, as
/// `(macro name, T)`.
fn newtype_macros(source: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for (name, body) in items(source, "macro_rules!") {
        let Some(at) = body.find("struct $name") else {
            continue;
        };
        let rest = &body[at + "struct $name".len()..];
        let open = rest
            .find(|c: char| !c.is_whitespace())
            .filter(|&p| rest[p..].starts_with('('))
            .unwrap_or_else(|| panic!("macro {name}! defines `struct $name` but not as `($type)`"));
        let close = matching(rest, open, '(', ')').expect("wrapped type closes");
        found.push((name, rest[open + 1..close].trim().to_owned()));
    }
    found
}

/// Every module-level `<macro>!(... Name)` invocation in masked `source`.
fn invocations(source: &str, macro_name: &str) -> Vec<String> {
    let needle = format!("{macro_name}!(");
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = source[from..].find(&needle) {
        let start = from + at;
        let open = start + needle.len() - 1;
        let close = matching(source, open, '(', ')').expect("invocation closes");
        let before_ok = source[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !is_ident_char(c));
        if before_ok && depth_at(source, start) == 0 {
            let inside = strip_attributes(&source[open + 1..close]);
            found.push(inside.trim().to_owned());
        }
        from = close;
    }
    found
}

/// How one model file names types: which file it is, the model's other files, and what its
/// module-level `use` declarations import.
struct Scope {
    /// The file name under `crates/canon/src/model/`; `mod.rs` is the `model` module itself.
    file: String,
    /// The stems of the model's other files: `model`'s child modules.
    children: BTreeSet<String>,
    /// Imported name to the path it was imported from, as written.
    imports: BTreeMap<String, Vec<String>>,
}

impl Scope {
    fn new(file: &str, files: &[&str], masked: &str) -> Scope {
        let children = files
            .iter()
            .filter(|f| **f != "mod.rs")
            .filter_map(|f| f.strip_suffix(".rs"))
            .map(str::to_owned)
            .collect();
        let mut scope = Scope {
            file: file.to_owned(),
            children,
            imports: BTreeMap::new(),
        };
        for tree in use_trees(masked) {
            for (name, path) in expand_use(&tree, &[]) {
                if name == "*" {
                    assert!(
                        scope.inside_model(&path),
                        "{file} glob-imports `{}::*` from outside the model; a bare type name there \
                         could name a type outside crates/canon/src/model/",
                        path.join("::")
                    );
                    continue;
                }
                scope.imports.insert(name, path);
            }
        }
        scope
    }

    /// Whether `path`, written in this file, names an item of the `model` module or its children.
    fn inside_model(&self, path: &[String]) -> bool {
        // This file's module path below the crate root.
        let mut at: Vec<String> = vec!["model".to_owned()];
        if self.file != "mod.rs" {
            at.push(self.file.trim_end_matches(".rs").to_owned());
        }
        let mut segments = path.iter().map(String::as_str).peekable();
        match segments.peek() {
            Some(&"crate") => {
                segments.next();
                at.clear();
            }
            Some(&"self") => {
                segments.next();
            }
            Some(&"super") => {
                while segments.peek() == Some(&"super") {
                    segments.next();
                    if at.pop().is_none() {
                        return false;
                    }
                }
            }
            // A relative path: a child module of this file's module, or an external crate.
            Some(first) if self.file == "mod.rs" && self.children.contains(*first) => {}
            _ => return false,
        }
        let rest: Vec<&str> = segments.collect();
        let mut full: Vec<&str> = at.iter().map(String::as_str).collect();
        full.extend(rest.iter().copied());
        full.len() >= 2 && full[0] == "model"
    }

    /// The model-local name of the type `path` names, or `None` when it names a type outside the
    /// model.
    fn model_name(&self, path: &str) -> Option<String> {
        let segments: Vec<String> = path.split("::").map(|s| s.trim().to_owned()).collect();
        let last = segments.last()?.clone();
        if segments.len() == 1 {
            return match self.imports.get(&last) {
                Some(from) => self.inside_model(from).then_some(from.last()?.clone()),
                None => Some(last),
            };
        }
        self.inside_model(&segments).then_some(last)
    }

    /// Whether `name` is the prelude or primitive type of that name, not an import of it.
    fn is_prelude(&self, path: &str, name: &str) -> bool {
        path.trim() == name && !self.imports.contains_key(name)
    }
}

/// The trees of every module-level `use` declaration in masked `source`, without `use` and `;`.
fn use_trees(source: &str) -> Vec<String> {
    let mut trees = Vec::new();
    let mut from = 0;
    while let Some(at) = source[from..].find("use ").map(|p| from + p) {
        from = at + 4;
        let before_ok = source[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !is_ident_char(c));
        if !before_ok || depth_at(source, at) != 0 {
            continue;
        }
        let end = source[from..].find(';').map_or(source.len(), |p| from + p);
        trees.push(
            source[from..end]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        );
        from = end;
    }
    trees
}

/// The `(name, path)` pairs a use tree imports; a glob is named `*`.
fn expand_use(tree: &str, prefix: &[String]) -> Vec<(String, Vec<String>)> {
    let tree = tree.trim().trim_start_matches("::");
    if let Some(open) = tree.find('{') {
        let mut base = prefix.to_vec();
        base.extend(
            tree[..open]
                .trim()
                .trim_end_matches("::")
                .split("::")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
        );
        let close = matching(tree, open, '{', '}').expect("use group closes");
        return split_top_braces(&tree[open + 1..close])
            .iter()
            .flat_map(|item| expand_use(item, &base))
            .collect();
    }
    let (path, alias) = match tree.split_once(" as ") {
        Some((path, alias)) => (path, Some(alias.trim().to_owned())),
        None => (tree, None),
    };
    let mut full = prefix.to_vec();
    full.extend(
        path.split("::")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned),
    );
    if full.last().map(String::as_str) == Some("self") {
        full.pop();
    }
    if full.last().map(String::as_str) == Some("*") {
        full.pop();
        return vec![("*".to_owned(), full)];
    }
    let name = alias.unwrap_or_else(|| full.last().cloned().unwrap_or_default());
    vec![(name, full)]
}

/// Splits on commas outside `{}`, dropping empty items.
fn split_top_braces(text: &str) -> Vec<String> {
    let mut items = vec![String::new()];
    let mut depth = 0i32;
    for c in text.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                items.push(String::new());
                continue;
            }
            _ => {}
        }
        items.last_mut().expect("one item").push(c);
    }
    items.retain(|item| !item.trim().is_empty());
    items
}

/// A Rust type written in the model file `scope`, in the shared type language. A type is matched
/// by the path it is written with: a path into a module outside the model matches nothing, even
/// when its last segment is a model type's name.
fn rust_ty(text: &str, scope: &Scope) -> Ty {
    let text = text.trim();
    if let Some(open) = text.find('<') {
        let head = &text[..open];
        let args = split_top(&text[open + 1..text.len() - 1]);
        let arg = |index: usize| Box::new(rust_ty(&args[index], scope));
        return match args.len() {
            1 if scope.is_prelude(head, "Option") => Ty::Optional(arg(0)),
            1 if scope.is_prelude(head, "Vec") => Ty::List(arg(0)),
            1 if scope.is_prelude(head, "Box") => *arg(0),
            2 if scope.model_name(head).as_deref() == Some("Declarations") => {
                Ty::Map(arg(0), arg(1))
            }
            _ => unmapped(text),
        };
    }
    if scope.is_prelude(text, "String") {
        return Ty::Primitive("string".to_owned());
    }
    if scope.is_prelude(text, "i64") {
        return integer();
    }
    if scope.is_prelude(text, "u64") {
        return non_negative_integer();
    }
    let is_path = text
        .split("::")
        .all(|s| !s.trim().is_empty() && s.trim().chars().all(is_ident_char));
    match scope.model_name(text) {
        // The model's untyped section payload, `model::Json`, is the specification's `Json`.
        Some(name) if is_path && name == "Json" => Ty::Primitive("json".to_owned()),
        Some(name) if is_path && name.starts_with(char::is_uppercase) => Ty::Declared(name),
        _ => unmapped(text),
    }
}

/// A Rust type with no specification counterpart. Private helper types the entity does not reach
/// use such types; one the entity does reach differs from every specification type, so the
/// comparison names it.
fn unmapped(text: &str) -> Ty {
    Ty::Primitive(format!("rust `{text}`"))
}

/// `CamelCase` as `snake_case`.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for (index, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Whether an attribute list (whitespace removed) makes what it is on conditional.
fn is_conditional(text: &str) -> bool {
    let compact: String = text.split_whitespace().collect();
    compact.contains("#[cfg(") || compact.contains("#[cfg_attr(")
}

/// The fields or variants of `owner`, attributes removed. One under `#[cfg(...)]` or
/// `#[cfg_attr(...)]` may not exist in the model that is built, so it fails loudly rather than
/// being counted either way.
fn members(file: &str, owner: &str, body: &str) -> Vec<String> {
    split_top(body)
        .into_iter()
        .map(|member| {
            assert!(
                !is_conditional(&member),
                "{file}: {owner} has a conditionally compiled member, which this test cannot \
                 compare with ess/: {}",
                member.split_whitespace().collect::<Vec<_>>().join(" ")
            );
            strip_attributes(&member).trim().to_owned()
        })
        .filter(|member| !member.is_empty())
        .collect()
}

/// Fails loudly on a module-level `#[cfg(...)]` other than `#[cfg(test)]` (already blanked): the
/// item under it may not exist in the model that is built.
fn refuse_conditional_items(file: &str, masked: &str) {
    let mut from = 0;
    while let Some(at) = masked[from..].find("#[").map(|p| from + p) {
        let close = matching(masked, at + 1, '[', ']').expect("attribute closes");
        let attribute = &masked[at..=close];
        assert!(
            depth_at(masked, at) != 0 || !is_conditional(attribute),
            "{file}: module-level `{attribute}` makes a model item conditional, which this test \
             cannot compare with ess/"
        );
        from = close + 1;
    }
}

/// Module-level type definitions by name, each with the file that defines it.
#[derive(Default)]
struct Definitions {
    shapes: BTreeMap<String, Shape>,
    files: BTreeMap<String, String>,
}

impl Definitions {
    /// Adds a definition; a second module-level definition of one name fails loudly.
    fn insert(&mut self, name: String, shape: Shape, file: &str) {
        if let Some(first) = self.files.get(&name) {
            panic!(
                "the Rust model defines `{name}` twice at module level, in {first} and {file}; \
                 one would shadow the other"
            );
        }
        self.files.insert(name.clone(), file.to_owned());
        self.shapes.insert(name, shape);
    }

    fn get(&self, name: &str) -> Option<&Shape> {
        self.shapes.get(name)
    }
}

/// The Rust model as read from `sources`, reduced to what the entity reaches.
fn rust_model(sources: &[(String, String)]) -> Model {
    let masked: Vec<(&str, String)> = sources
        .iter()
        .map(|(file, text)| (file.as_str(), without_test_items(&mask(text))))
        .collect();

    let files: Vec<&str> = masked.iter().map(|(file, _)| *file).collect();
    for (file, source) in &masked {
        refuse_conditional_items(file, source);
    }

    let mut all = Definitions::default();
    let macros: Vec<(String, String)> = masked
        .iter()
        .flat_map(|(_, source)| newtype_macros(source))
        .collect();
    for (file, source) in &masked {
        let scope = Scope::new(file, &files, source);
        for (macro_name, wrapped) in &macros {
            for name in invocations(source, macro_name) {
                all.insert(name, Shape::Newtype(rust_ty(wrapped, &scope)), file);
            }
        }
    }
    for (file, source) in &masked {
        let scope = Scope::new(file, &files, source);
        let source = source.as_str();
        for (name, body) in items(source, "struct") {
            let fields = members(file, &name, body)
                .into_iter()
                .map(|field| {
                    let field = field.strip_prefix("pub(crate)").unwrap_or(&field);
                    let field = field.trim().strip_prefix("pub ").unwrap_or(field).trim();
                    let (name, ty) = field
                        .split_once(':')
                        .unwrap_or_else(|| panic!("field `{field}` has a type"));
                    (name.trim().to_owned(), rust_ty(ty, &scope))
                })
                .collect();
            all.insert(name, Shape::Struct(fields), file);
        }
        for (name, body) in items(source, "enum") {
            let variants = members(file, &name, body);
            let shape = if variants.iter().all(|v| v.chars().all(is_ident_char)) {
                Shape::Enum(variants.iter().map(|v| snake(v)).collect())
            } else {
                Shape::Union(
                    variants
                        .iter()
                        .map(|variant| {
                            let open = variant
                                .find('(')
                                .unwrap_or_else(|| panic!("{name}::{variant} holds one value"));
                            let payload = &variant[open + 1..variant.len() - 1];
                            (snake(variant[..open].trim()), rust_ty(payload, &scope))
                        })
                        .collect(),
                )
            };
            all.insert(name, shape, file);
        }
    }

    let mut entities = BTreeMap::new();
    for mapping in ENTITIES {
        entities.insert(mapping.name.to_owned(), rust_entity(mapping, &all));
    }

    // Keep only the types the entities and the documents reach.
    let mut types = BTreeMap::new();
    let mut pending: Vec<Ty> = entities
        .values()
        .flat_map(|entity: &Entity| {
            let mut reached: Vec<Ty> = entity.fields.iter().map(|(_, ty)| ty.clone()).collect();
            reached.push(entity.identity.1.clone());
            reached
        })
        .collect();
    pending.extend(
        DOCUMENTS
            .iter()
            .map(|name| Ty::Declared((*name).to_owned())),
    );
    while let Some(ty) = pending.pop() {
        match ty {
            Ty::Primitive(_) => {}
            Ty::Optional(of) | Ty::List(of) => pending.push(*of),
            Ty::Map(key, value) => {
                pending.push(*key);
                pending.push(*value);
            }
            Ty::Declared(name) => {
                if types.contains_key(&name) {
                    continue;
                }
                let shape = all
                    .get(&name)
                    .unwrap_or_else(|| {
                        panic!("the Rust model uses `{name}` but defines no such type")
                    })
                    .clone();
                match &shape {
                    Shape::Newtype(of) => pending.push(of.clone()),
                    Shape::Struct(fields) | Shape::Union(fields) => {
                        pending.extend(fields.iter().map(|(_, ty)| ty.clone()));
                    }
                    Shape::Enum(_) => {}
                }
                types.insert(name, shape);
            }
        }
    }
    Model { entities, types }
}

/// One entity as the Rust model writes it: the struct `mapping` names, its header (if any)
/// written in place, its identity field taken out of the fields.
fn rust_entity(mapping: &EntityMapping, all: &Definitions) -> Entity {
    let name = mapping.name;
    let Some(Shape::Struct(entity_fields)) = all.get(name).cloned() else {
        panic!("the Rust model has no struct {name}");
    };
    let (identity_field, identity_name) = mapping.identity;
    let mut written = Vec::new();
    for (field, ty) in entity_fields {
        if Some(field.as_str()) != mapping.header {
            written.push((field, ty));
            continue;
        }
        let Ty::Declared(header) = &ty else {
            panic!("{name}.{field} is a struct");
        };
        let Some(Shape::Struct(header_fields)) = all.get(header) else {
            panic!("{name}.{field} is a struct");
        };
        written.extend(header_fields.iter().cloned());
    }
    let mut identity = None;
    let mut fields = Vec::new();
    for (field, ty) in written {
        if field == identity_field && identity.is_none() {
            identity = Some((identity_name.to_owned(), ty));
        } else {
            fields.push((field, ty));
        }
    }
    let identity =
        identity.unwrap_or_else(|| panic!("{name} has no `{identity_field}` identity field"));
    Entity { identity, fields }
}

// ---------------------------------------------------------------------------------------------
// The comparison.

fn compare_members(
    owner: &str,
    what: &str,
    spec: &[(String, Ty)],
    rust: &[(String, Ty)],
    out: &mut Vec<String>,
) {
    let spec_map: BTreeMap<&str, &Ty> = spec.iter().map(|(n, t)| (n.as_str(), t)).collect();
    let rust_map: BTreeMap<&str, &Ty> = rust.iter().map(|(n, t)| (n.as_str(), t)).collect();
    for (name, ty) in rust {
        match spec_map.get(name.as_str()) {
            None => out.push(format!(
                "{owner}: Rust {what} `{name}: {ty}` is not in ess/"
            )),
            Some(spec_ty) if *spec_ty != ty => out.push(format!(
                "{owner}.{name}: ess/ says {spec_ty}, Rust says {ty}"
            )),
            Some(_) => {}
        }
    }
    for (name, ty) in spec {
        if !rust_map.contains_key(name.as_str()) {
            out.push(format!(
                "{owner}: ess/ {what} `{name}: {ty}` is not in the Rust model"
            ));
        }
    }
}

/// Every difference between the two models, in a stable order.
fn differences(spec: &Model, rust: &Model) -> Vec<String> {
    let mut out = Vec::new();
    for (entity, rust_entity) in &rust.entities {
        let Some(spec_entity) = spec.entities.get(entity) else {
            out.push(format!(
                "Rust entity `{entity}` is not an entity ess/ declares"
            ));
            continue;
        };
        if spec_entity.identity != rust_entity.identity {
            out.push(format!(
                "{entity} identity: ess/ says {}: {}, Rust says {}: {}",
                spec_entity.identity.0,
                spec_entity.identity.1,
                rust_entity.identity.0,
                rust_entity.identity.1
            ));
        }
        compare_members(
            entity,
            "field",
            &spec_entity.fields,
            &rust_entity.fields,
            &mut out,
        );
    }
    for entity in spec.entities.keys() {
        if !rust.entities.contains_key(entity) {
            out.push(format!(
                "ess/ entity `{entity}` has no Rust struct this test maps (ENTITIES)"
            ));
        }
    }
    for (name, rust_shape) in &rust.types {
        let Some(spec_shape) = spec.types.get(name) else {
            out.push(format!(
                "Rust {} `{name}` is reached from the Rust model but ess/ does not declare it",
                rust_shape.kind()
            ));
            continue;
        };
        match (spec_shape, rust_shape) {
            (Shape::Newtype(s), Shape::Newtype(r)) if s != r => {
                out.push(format!("{name}: ess/ wraps {s}, Rust wraps {r}"));
            }
            (Shape::Struct(s), Shape::Struct(r)) => compare_members(name, "field", s, r, &mut out),
            (Shape::Union(s), Shape::Union(r)) => compare_members(name, "variant", s, r, &mut out),
            (Shape::Enum(s), Shape::Enum(r)) if s != r => out.push(format!(
                "{name}: ess/ variants [{}], Rust variants [{}]",
                s.join(", "),
                r.join(", ")
            )),
            (s, r) if s.kind() != r.kind() => out.push(format!(
                "{name}: ess/ declares a {}, Rust defines a {}",
                s.kind(),
                r.kind()
            )),
            _ => {}
        }
    }
    for (name, shape) in &spec.types {
        if !rust.types.contains_key(name) {
            out.push(format!(
                "ess/ {} `{name}` is not reached from the Rust model",
                shape.kind()
            ));
        }
    }
    out
}

fn assert_equal(spec: &Model, rust: &Model) {
    let found = differences(spec, rust);
    assert!(
        found.is_empty(),
        "ess/ and crates/canon/src/model/ differ; the first difference is:\n  {}\nall {}:\n  {}",
        found[0],
        found.len(),
        found.join("\n  ")
    );
}

/// The real model with one edit; the edit must apply.
fn edited(file: &str, from: &str, to: &str) -> Model {
    rust_model(&edited_sources(&[(file, from, to)]))
}

/// The real model sources with every `(file, from, to)` edit applied; each edit must apply.
fn edited_sources(edits: &[(&str, &str, &str)]) -> Vec<(String, String)> {
    let mut sources = model_sources();
    for (file, from, to) in edits {
        let (_, text) = sources
            .iter_mut()
            .find(|(name, _)| name == file)
            .unwrap_or_else(|| panic!("no model file {file}"));
        assert!(
            text.contains(from),
            "the control's edit `{from}` no longer applies to {file}"
        );
        *text = text.replacen(from, to, 1);
    }
    sources
}

fn assert_named(spec: &Model, rust: &Model, needles: &[&str]) {
    let found = differences(spec, rust);
    assert!(
        found
            .first()
            .is_some_and(|first| needles.iter().all(|needle| first.contains(needle))),
        "expected the first difference to name {needles:?}, found {found:?}"
    );
}

#[test]
fn specification_and_model_are_equal() {
    let spec = specification();
    let rust = rust_model(&model_sources());
    let entities_with_fields = |model: &Model| {
        model
            .entities
            .values()
            .filter(|entity| !entity.fields.is_empty())
            .count()
    };
    assert!(
        entities_with_fields(&spec) == ENTITIES.len()
            && entities_with_fields(&rust) == ENTITIES.len()
            && DOCUMENTS.iter().all(|name| spec.types.contains_key(*name))
            && DOCUMENTS.iter().all(|name| rust.types.contains_key(*name)),
        "nothing or too little was compared: ess/ declares {} entities with fields and {} type(s), \
         the Rust model has {} entities with fields and reaches {} type(s); every one of the {} \
         entities and the documents {DOCUMENTS:?} must be on both sides",
        entities_with_fields(&spec),
        spec.types.len(),
        entities_with_fields(&rust),
        rust.types.len(),
        ENTITIES.len()
    );
    assert_equal(&spec, &rust);
}

/// Expectation 7: a field added to a model struct and not to `ess/` fails, naming the field.
#[test]
fn a_field_added_only_to_the_model_is_named() {
    let spec = specification();
    let rust = edited(
        "mod.rs",
        "pub struct Artifact {",
        "pub struct Artifact {\n    pub probe_field: String,",
    );
    assert_named(&spec, &rust, &["Artifact", "probe_field"]);
}

/// The same for a field added to the header the entity is written from.
#[test]
fn a_field_added_to_the_header_is_named() {
    let spec = specification();
    let rust = edited(
        "mod.rs",
        "pub struct ProtocolHeader {",
        "pub struct ProtocolHeader {\n    pub probe_field: u64,",
    );
    assert_named(&spec, &rust, &["Protocol", "probe_field"]);
}

#[test]
fn a_field_removed_from_the_model_is_named() {
    let spec = specification();
    let rust = edited("mod.rs", "pub effect: Option<EffectClass>,", "");
    assert_named(&spec, &rust, &["Action", "effect", "not in the Rust model"]);
}

#[test]
fn a_changed_field_type_is_named() {
    let spec = specification();
    let rust = edited("mod.rs", "pub revision: u64,", "pub revision: String,");
    assert_named(&spec, &rust, &["Protocol.revision", "integer", "string"]);
}

#[test]
fn a_changed_map_key_is_named() {
    let spec = specification();
    let rust = edited(
        "mod.rs",
        "Declarations<ClaimId, Claim>",
        "Declarations<OutcomeId, Claim>",
    );
    assert_named(&spec, &rust, &["Protocol.claims", "ClaimId", "OutcomeId"]);
}

#[test]
fn a_union_variant_added_only_to_the_model_is_named() {
    let spec = specification();
    let rust = edited(
        "predicate.rs",
        "    Claim(ClaimTest),\n}",
        "    Claim(ClaimTest),\n    ProbeVariant(ClaimTest),\n}",
    );
    assert_named(&spec, &rust, &["Predicate", "probe_variant"]);
}

#[test]
fn an_enum_variant_added_only_to_the_model_is_named() {
    let spec = specification();
    let rust = edited(
        "predicate.rs",
        "    Unknown,\n}",
        "    Unknown,\n    Probe,\n}",
    );
    assert_named(&spec, &rust, &["Truth", "probe"]);
}

#[test]
fn a_type_reached_only_in_the_model_is_named() {
    let spec = specification();
    let rust = edited(
        "mod.rs",
        "pub capability: CapabilityId,\n}",
        "pub capability: CapabilityId,\n    pub grant: ProbeGrant,\n}\n\npub struct ProbeGrant {\n    pub scope: String,\n}",
    );
    let found = differences(&spec, &rust);
    assert!(
        found
            .iter()
            .any(|d| d.contains("CapabilityRequirement") && d.contains("grant"))
            && found
                .iter()
                .any(|d| d.contains("ProbeGrant") && d.contains("does not declare")),
        "expected the new field and the new type to be named, found {found:?}"
    );
}

/// `u64` admits no negative value; a signed revision differs from `ess/`'s `revision >= 0`.
#[test]
fn a_signed_revision_is_named() {
    let spec = specification();
    let rust = edited("mod.rs", "pub revision: u64,", "pub revision: i64,");
    assert_named(
        &spec,
        &rust,
        &["Protocol.revision", "integer >= 0", "Rust says integer"],
    );
}

/// What `identifier!` wraps is read from the macro, not assumed.
#[test]
fn a_changed_identifier_representation_is_named() {
    let spec = specification();
    let rust = edited(
        "ids.rs",
        "pub struct $name(String);",
        "pub struct $name(u64);",
    );
    assert_named(
        &spec,
        &rust,
        &["ActionId", "wraps string", "wraps integer >= 0"],
    );
}

/// Same-named items under `#[cfg(test)]`, in an inline module or inside a function are not the
/// model, and do not hide a change to the model's own type.
#[test]
fn a_same_named_fixture_does_not_hide_a_model_change() {
    let spec = specification();
    let fixture = "#[cfg(test)]\nmod tests {\n    struct Artifact {\n        note: &'static str,\n    }\n}\n\n\
                   mod fixtures {\n    pub struct Artifact {\n        pub label: char,\n    }\n}\n\n\
                   fn local() {\n    struct Artifact {\n        brace: char,\n    }\n    let _ = '}';\n}\n\n\
                   impl Predicate {";
    let rust = rust_model(&edited_sources(&[
        (
            "mod.rs",
            "pub struct Artifact {",
            "pub struct Artifact {\n    pub owner: String,",
        ),
        ("predicate.rs", "impl Predicate {", fixture),
    ]));
    assert_named(&spec, &rust, &["Artifact", "owner"]);
}

/// Two module-level definitions of one name fail loudly instead of one shadowing the other.
#[test]
fn a_duplicate_module_level_name_fails_loudly() {
    let sources = edited_sources(&[(
        "predicate.rs",
        "impl Predicate {",
        "pub struct Artifact {\n    pub description: Option<String>,\n}\n\nimpl Predicate {",
    )]);
    let panic = std::panic::catch_unwind(|| rust_model(&sources))
        .expect_err("a duplicate module-level name is refused");
    let message = panic.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(
        message.contains("`Artifact` twice")
            && message.contains("mod.rs")
            && message.contains("predicate.rs"),
        "{message}"
    );
}

/// A path into a module outside the model matches nothing, whatever its last segment.
#[test]
fn a_type_outside_the_model_with_a_model_name_is_named() {
    let spec = specification();
    let rust = edited(
        "mod.rs",
        "Declarations<ClaimId, Claim>",
        "Declarations<ClaimId, crate::ir::Claim>",
    );
    assert_named(&spec, &rust, &["Protocol.claims", "crate::ir::Claim"]);
}

/// The same for a bare name imported from outside the model, renamed or not.
#[test]
fn a_bare_name_imported_from_outside_the_model_is_named() {
    let spec = specification();
    let rust = rust_model(&edited_sources(&[
        (
            "mod.rs",
            "use std::marker::PhantomData;",
            "use std::marker::PhantomData;\nuse crate::ir::Outcome as Ending;",
        ),
        (
            "mod.rs",
            "Declarations<OutcomeId, Outcome>",
            "Declarations<OutcomeId, Ending>",
        ),
    ]));
    assert_named(&spec, &rust, &["Protocol.outcomes", "Ending"]);
}

/// Paths that do lead into the model are the model's types.
#[test]
fn paths_into_the_model_are_the_model_types() {
    let spec = specification();
    let rust = rust_model(&edited_sources(&[
        (
            "mod.rs",
            "Declarations<ClaimId, Claim>",
            "self::Declarations<crate::model::ClaimId, crate::model::Claim>",
        ),
        (
            "predicate.rs",
            "pub claim: ClaimId,",
            "pub claim: super::ids::ClaimId,",
        ),
    ]));
    assert_equal(&spec, &rust);
}

fn panic_message(run: impl FnOnce() -> Model + std::panic::UnwindSafe) -> String {
    let panic = std::panic::catch_unwind(run).expect_err("the model is refused");
    panic.downcast_ref::<String>().cloned().unwrap_or_default()
}

/// A field under any `#[cfg(...)]` may not be in the model that is built; it fails loudly.
#[test]
fn a_conditionally_compiled_field_fails_loudly() {
    let sources = edited_sources(&[(
        "mod.rs",
        "    pub effect: Option<EffectClass>,",
        "    #[cfg(feature = \"effects\")]\n    pub effect: Option<EffectClass>,",
    )]);
    let message = panic_message(|| rust_model(&sources));
    assert!(
        message.contains("Action has a conditionally compiled member")
            && message.contains("effect"),
        "{message}"
    );
}

/// The same for a variant, and for `cfg_attr`.
#[test]
fn a_conditionally_compiled_variant_fails_loudly() {
    let sources = edited_sources(&[(
        "predicate.rs",
        "    Claim(ClaimTest),\n}",
        "    #[cfg_attr(test, allow(dead_code))]\n    Claim(ClaimTest),\n}",
    )]);
    let message = panic_message(|| rust_model(&sources));
    assert!(
        message.contains("Predicate has a conditionally compiled member"),
        "{message}"
    );
}

/// A module-level item under a `cfg` other than `#[cfg(test)]` fails loudly.
#[test]
fn a_conditionally_compiled_item_fails_loudly() {
    let sources = edited_sources(&[(
        "mod.rs",
        "pub struct Obligation {",
        "#[cfg(feature = \"obligations\")]\npub struct Obligation {",
    )]);
    let message = panic_message(|| rust_model(&sources));
    assert!(
        message.contains("makes a model item conditional"),
        "{message}"
    );
}

/// Map keys come only from the files `ess/ess-inputs.yaml` lists: an unlisted stale copy beside
/// the domain file does not supply them.
#[test]
fn an_unlisted_file_under_ess_does_not_supply_map_keys() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("canon-ess-model-matches")
        .join(format!("{:016x}", {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            repo_root().hash(&mut hasher);
            hasher.finish()
        }))
        .join("unlisted-stale-copy");
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove the previous copy");
    }
    let domains = root.join("ess/domains");
    fs::create_dir_all(&domains).expect("create the copy");
    let spec = repo_root().join("ess");
    for file in [
        "system.yaml",
        "ess-inputs.yaml",
        "domains/protocol.yaml",
        "domains/check.yaml",
    ] {
        fs::copy(spec.join(file), root.join("ess").join(file)).expect("copy spec file");
    }
    let domain = domains.join("protocol.yaml");
    let original = fs::read_to_string(&domain).expect("domain reads");
    let from = "type: Map<canon.protocol.ClaimId, canon.protocol.Claim>";
    assert!(
        original.contains(from),
        "the control's edit no longer applies"
    );
    fs::write(
        &domain,
        original.replacen(
            from,
            "type: Map<canon.protocol.OutcomeId, canon.protocol.Claim>",
            1,
        ),
    )
    .expect("edited domain written");
    for stale in ["protocol.yaml.orig", "protocol.yaml~", "zz-protocol.yaml"] {
        fs::write(domains.join(stale), &original).expect("stale copy written");
    }

    let spec = specification_at(&root);
    let rust = rust_model(&model_sources());
    assert_named(&spec, &rust, &["Protocol.claims", "OutcomeId", "ClaimId"]);
}

/// The Rust model has one namespace, so a local name declared in two domains fails loudly rather
/// than one declaration hiding the other: here `canon.check` declares its own `ClaimId`.
#[test]
fn a_local_name_declared_in_two_domains_fails_loudly() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("canon-ess-model-matches")
        .join(format!("{:016x}", {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            repo_root().hash(&mut hasher);
            hasher.finish()
        }))
        .join("one-name-two-domains");
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove the previous copy");
    }
    fs::create_dir_all(root.join("ess/domains")).expect("create the copy");
    let spec = repo_root().join("ess");
    for file in [
        "system.yaml",
        "ess-inputs.yaml",
        "domains/protocol.yaml",
        "domains/check.yaml",
    ] {
        fs::copy(spec.join(file), root.join("ess").join(file)).expect("copy spec file");
    }
    let domain = root.join("ess/domains/check.yaml");
    let original = fs::read_to_string(&domain).expect("domain reads");
    assert!(
        original.contains("\ntypes:\n"),
        "the control's edit no longer applies"
    );
    fs::write(
        &domain,
        original.replacen(
            "\ntypes:\n",
            "\ntypes:\n  - name: canon.check.ClaimId\n    kind: newtype\n    of: String\n\n",
            1,
        ),
    )
    .expect("edited domain written");

    let panic = std::panic::catch_unwind(|| specification_at(&root))
        .expect_err("a local name in two domains is refused");
    let message = panic.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(
        message.contains("two domains declare a type `ClaimId`"),
        "{message}"
    );
}

/// The second entity is compared like the first: a field added to `Case` is named.
#[test]
fn a_field_added_to_the_case_is_named() {
    let spec = specification();
    let rust = edited(
        "case.rs",
        "pub struct Case {",
        "pub struct Case {\n    pub probe_field: String,",
    );
    assert_named(&spec, &rust, &["Case", "probe_field", "not in ess/"]);
}

/// The case's identity is compared: a case id that is plain text is named.
#[test]
fn a_changed_case_identity_is_named() {
    let spec = specification();
    let rust = edited("case.rs", "pub id: CaseId,", "pub id: String,");
    assert_named(&spec, &rust, &["Case identity", "CaseId", "string"]);
}

/// A type only the case reaches is compared.
#[test]
fn a_changed_case_artifact_field_is_named() {
    let spec = specification();
    let rust = edited(
        "case.rs",
        "pub revision: Revision,",
        "pub revision: String,",
    );
    assert_named(
        &spec,
        &rust,
        &["CaseArtifact.revision", "Revision", "string"],
    );
}

/// The documents no entity holds are compared: a field removed from the evidence record is named.
#[test]
fn a_field_removed_from_the_evidence_record_is_named() {
    let spec = specification();
    let rust = edited("evidence.rs", "pub subject: ArtifactId,", "");
    assert_named(
        &spec,
        &rust,
        &["EvidenceRecord", "subject", "not in the Rust model"],
    );
}

/// The same for the decision, through the claim entry only the decision reaches.
#[test]
fn a_changed_claim_decision_value_is_named() {
    let spec = specification();
    let rust = edited("decision.rs", "pub value: Truth,", "pub value: String,");
    assert_named(&spec, &rust, &["ClaimDecision.value", "Truth", "string"]);
}

/// `NON_NEGATIVE` reads the decision's protocol revision as `>= 0`: a signed one is named.
#[test]
fn a_signed_decision_revision_is_named() {
    let spec = specification();
    let rust = edited(
        "decision.rs",
        "pub protocol_revision: u64,",
        "pub protocol_revision: i64,",
    );
    assert_named(
        &spec,
        &rust,
        &[
            "Decision.protocol_revision",
            "integer >= 0",
            "Rust says integer",
        ],
    );
}

/// An entity the specification declares that this test does not map is a difference, not skipped.
#[test]
fn an_entity_without_a_rust_mapping_is_named() {
    let mut spec = specification();
    let case = spec.entities["Case"].clone();
    spec.entities.insert("Probe".to_owned(), case);
    let rust = rust_model(&model_sources());
    let found = differences(&spec, &rust);
    assert!(
        found
            .iter()
            .any(|d| d.contains("entity `Probe`") && d.contains("no Rust struct")),
        "{found:?}"
    );
}
