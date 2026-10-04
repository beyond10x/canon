//! The generated pages, each derived from the repository: the CLI from its clap definition, the
//! `protocol/1` reference and JSON Schema from the model types, the validation, compiled-form and
//! conformance references from their module documentation, and the worked example from the
//! investigation fixture run through Canon itself.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use b10x_canon::model::{Predicate, Protocol};
use b10x_canon::{eval, ir, model, validate};
use clap::CommandFactory;

use crate::json::{self, Json, obj};
use crate::md::{block, cell, code, front_matter, plain, table};
use crate::source::{self, Model, Reading, Shape, Ty, TypeDef};

/// Where generated Markdown goes, relative to the repository root.
pub const REFERENCE: &str = "website/docs/reference";
/// Where the generated JSON Schema goes; Docusaurus serves `static/` at the site root.
pub const SCHEMA_DIR: &str = "website/static/schemas";
const FIXTURE: &str = "fixtures/investigation/protocol.yaml";
const INVALID: &str = "fixtures/investigation/invalid";
const SOURCE: &str = "https://github.com/beyond10x/canon/blob/main";
const SITE: &str = "https://beyond10x.github.io/canon";

/// Types whose `Deserialize` is written by hand, and how a document writes them. Nothing else
/// about the model is written here; a hand-written `Deserialize` without an entry stops generation.
const CUSTOM: [(&str, &str); 1] = [(
    "Truth",
    "Written `true`, `false` or `unknown`. A YAML boolean `true` or `false` is read as the same value.",
)];

/// Every generated file, keyed by its path relative to the repository root.
pub fn all(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let model = source::model(root)?;
    let mut files = BTreeMap::new();
    files.insert(format!("{REFERENCE}/cli.md"), cli());
    files.insert(format!("{REFERENCE}/protocol.md"), protocol(&model)?);
    files.insert(format!("{REFERENCE}/documents.md"), documents(&model)?);
    for (file, schema) in schemas(&model)? {
        files.insert(format!("{SCHEMA_DIR}/{file}"), schema);
    }
    files.insert(format!("{REFERENCE}/validation.md"), validation(root)?);
    files.insert(
        format!("{REFERENCE}/canon-ir.md"),
        module_page(
            root,
            "crates/canon/src/ir/mod.rs",
            "canon-ir/1, the compiled form",
            "canon-ir/1",
            "What compilation normalizes, and what it does not.",
        )?,
    );
    files.insert(
        format!("{REFERENCE}/evaluation.md"),
        module_page(
            root,
            "crates/canon/src/eval/mod.rs",
            "Evaluation",
            "Evaluation",
            "How canon evaluate decides claims, obligations, actions and outcomes, and when it refuses.",
        )?,
    );
    files.insert(
        format!("{REFERENCE}/conformance.md"),
        module_page(
            root,
            "crates/canon/src/conform/mod.rs",
            "Conformance scenarios",
            "Conformance scenarios",
            "The canon-conformance/1 scenario format, the registry and the report of canon conform run.",
        )?,
    );
    files.insert(
        format!("{REFERENCE}/investigation-example.md"),
        example(root)?,
    );
    Ok(files)
}

fn source_link(path: &str) -> String {
    format!("[`{path}`]({SOURCE}/{path})")
}

// ---------------------------------------------------------------------------------------------
// The CLI reference, walked from the clap definition.

fn commands(command: &clap::Command, out: &mut Vec<clap::Command>) {
    for sub in command.get_subcommands() {
        if sub.get_name() == "help" {
            continue;
        }
        out.push(sub.clone());
        commands(sub, out);
    }
}

fn anchor(command: &clap::Command) -> String {
    command
        .get_bin_name()
        .unwrap_or(command.get_name())
        .replace(' ', "-")
}

fn cli() -> String {
    let mut root = canon_cli::Cli::command();
    root.build();
    let mut all = Vec::new();
    commands(&root, &mut all);

    let about = root
        .get_about()
        .map(ToString::to_string)
        .unwrap_or_default();
    let mut out = front_matter(
        "canon CLI reference",
        "canon CLI",
        "Every command and option of the canon command line, generated from its definition.",
    );
    out.push_str(&format!(
        "Generated from the clap definition of the `canon` command line in {}. It lists only \
         commands that exist; commands the design proposes are on the \
         [status page](../status/where-this-stands.md).\n\n",
        source_link("crates/canon-cli/src/lib.rs")
    ));
    out.push_str(&format!(
        "`canon`: {}. `canon --version` prints its version, and `--help` after any command \
         prints the text shown below.\n\n## Commands\n\n",
        cell(&about)
    ));
    let rows: Vec<Vec<String>> = all
        .iter()
        .map(|command| {
            let name = command.get_bin_name().unwrap_or(command.get_name());
            vec![
                format!("[`{name}`](#{})", anchor(command)),
                cell(
                    &command
                        .get_about()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                ),
            ]
        })
        .collect();
    out.push_str(&table(&["Command", "What it does"], &rows));

    for command in &all {
        let name = command.get_bin_name().unwrap_or(command.get_name());
        out.push_str(&format!("\n## `{name}`\n\n"));
        let about = command
            .get_long_about()
            .or(command.get_about())
            .map(ToString::to_string)
            .unwrap_or_default();
        out.push_str(&format!("{}\n\n", cell(&about)));
        let subcommands: Vec<_> = command
            .get_subcommands()
            .filter(|sub| sub.get_name() != "help")
            .collect();
        if !subcommands.is_empty() {
            let list: Vec<String> = subcommands
                .iter()
                .map(|sub| format!("[`{}`](#{})", sub.get_name(), anchor(sub)))
                .collect();
            out.push_str(&format!("Subcommands: {}.\n\n", list.join(", ")));
        }
        let options: Vec<Vec<String>> = command
            .get_arguments()
            .filter(|arg| !matches!(arg.get_id().as_str(), "help" | "version"))
            .map(|arg| {
                let value = arg
                    .get_value_names()
                    .and_then(|names| names.first().map(ToString::to_string))
                    .unwrap_or_else(|| arg.get_id().as_str().to_uppercase());
                let written = match arg.get_long() {
                    Some(long) => format!("--{long} <{value}>"),
                    None => format!("<{value}>"),
                };
                let defaults: Vec<String> = arg
                    .get_default_values()
                    .iter()
                    .map(|default| default.to_string_lossy().into_owned())
                    .collect();
                vec![
                    code(&written),
                    if arg.is_required_set() { "yes" } else { "no" }.to_owned(),
                    if defaults.is_empty() {
                        "none".to_owned()
                    } else {
                        code(&defaults.join(", "))
                    },
                    cell(
                        &arg.get_long_help()
                            .or(arg.get_help())
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    ),
                ]
            })
            .collect();
        if !options.is_empty() {
            out.push_str(&table(
                &["Option", "Required", "Default", "Meaning"],
                &options,
            ));
            out.push('\n');
        }
        let help: String = command
            .clone()
            .render_long_help()
            .to_string()
            .lines()
            .map(|line| format!("{}\n", line.trim_end()))
            .collect();
        out.push_str(&format!("```text\n{}\n```\n", help.trim_end()));
    }

    out.push_str("\n## Exit status\n\n");
    let rows: Vec<Vec<String>> = canon_cli::EXIT_STATUSES
        .iter()
        .map(|(status, meaning)| vec![code(&status.to_string()), cell(meaning)])
        .collect();
    out.push_str(&table(&["Status", "Meaning"], &rows));
    out
}

// ---------------------------------------------------------------------------------------------
// The protocol/1 and evaluation-document references and their JSON Schemas, from the model types.

/// A document type the model reads or writes, and how the reference presents it.
struct Document {
    /// The model type a document of this format is read into, or written from.
    root: &'static str,
    /// The `*FORMAT` constant naming the format.
    format: &'static str,
    /// Where its JSON Schema goes, under `SCHEMA_DIR`, if Canon reads it.
    schema: Option<&'static str>,
}

const PROTOCOL: Document = Document {
    root: "Protocol",
    format: "FORMAT",
    schema: Some("protocol-1.schema.json"),
};

const EVALUATION_DOCUMENTS: [Document; 3] = [
    Document {
        root: "Case",
        format: "CASE_FORMAT",
        schema: Some("case-1.schema.json"),
    },
    Document {
        root: "EvidenceRecord",
        format: "EVIDENCE_FORMAT",
        schema: Some("evidence-1.schema.json"),
    },
    Document {
        root: "Decision",
        format: "DECISION_FORMAT",
        schema: None,
    },
];

/// The types that get their own section, in the order reached from `root`. A `try_from` target is
/// folded into the type it is read as. A type that is never read from a document is shown only
/// under a root that is itself written by Canon, not read.
fn sections<'a>(model: &'a Model, root: &str) -> Result<Vec<&'a TypeDef>, String> {
    let reachable = model.reachable(root)?;
    let written_by_canon = model.get(root)?.reading == Reading::NotRead;
    let written: BTreeSet<String> = reachable
        .iter()
        .filter_map(|name| match &model.types[name].reading {
            Reading::TryFrom(written) => Some(written.clone()),
            _ => None,
        })
        .collect();
    reachable
        .iter()
        .filter(|name| !written.contains(*name))
        .map(|name| model.get(name))
        .filter(|def| {
            written_by_canon || !matches!(def, Ok(def) if def.reading == Reading::NotRead)
        })
        .collect()
}

fn section_anchor(name: &str) -> String {
    name.to_lowercase()
}

/// Where each type's section is: an anchor on this page or a link to another page.
type Links = BTreeMap<String, String>;

fn render_ty(ty: &Ty, links: &Links) -> String {
    match ty {
        Ty::Text => "text".to_owned(),
        Ty::Count => "integer, 0 or more".to_owned(),
        Ty::Any => "any JSON value".to_owned(),
        Ty::Identifier(name) if text_format(name).is_some() => {
            format!("[text](#text-formats) (`{name}`)")
        }
        Ty::Identifier(name) => format!("[identifier](#identifiers) (`{name}`)"),
        Ty::Named(name) => match links.get(name) {
            Some(href) => format!("[`{name}`]({href})"),
            None => code(name),
        },
        Ty::Option(inner) | Ty::Boxed(inner) => render_ty(inner, links),
        Ty::List(inner) => format!("list of {}", render_ty(inner, links)),
        Ty::Map(key, value) => format!(
            "map from {} to {}",
            render_ty(key, links),
            render_ty(value, links)
        ),
    }
}

fn unwrap_option(ty: &Ty) -> &Ty {
    match ty {
        Ty::Option(inner) => inner,
        other => other,
    }
}

fn custom(name: &str) -> Result<&'static str, String> {
    CUSTOM
        .iter()
        .find(|(custom, _)| *custom == name)
        .map(|(_, written)| *written)
        .ok_or_else(|| {
            format!("`{name}` has a hand-written Deserialize; describe it in canon-docs CUSTOM")
        })
}

/// One row of a predicate's written form: the key, its value, what it means, and for a modifier
/// the variant key it belongs beside.
struct WrittenKey {
    key: String,
    ty: Ty,
    doc: String,
    beside: Option<String>,
}

/// The written form of a `try_from` enum: one key per variant, plus each modifier key, which
/// belongs beside the variant whose payload struct has a field of that name.
fn written_keys(model: &Model, def: &TypeDef, written: &str) -> Result<Vec<WrittenKey>, String> {
    let Shape::Enum(variants) = &def.shape else {
        return Err(format!("{}: try_from is read only for enums", def.name));
    };
    let Shape::Struct(fields) = &model.get(written)?.shape else {
        return Err(format!("{written} must be a struct"));
    };
    let mut keys = Vec::new();
    for field in fields {
        if let Some(variant) = variants
            .iter()
            .find(|variant| variant.name.to_lowercase() == field.name)
        {
            keys.push(WrittenKey {
                key: field.name.clone(),
                ty: unwrap_option(&field.ty).clone(),
                doc: variant.doc.clone(),
                beside: None,
            });
            continue;
        }
        let owner = variants.iter().find_map(|variant| {
            let Some(Ty::Named(payload)) = &variant.payload else {
                return None;
            };
            let Ok(TypeDef {
                shape: Shape::Struct(payload_fields),
                ..
            }) = model.get(payload)
            else {
                return None;
            };
            payload_fields
                .iter()
                .find(|payload_field| payload_field.name == field.name)
                .map(|payload_field| (variant.name.to_lowercase(), payload_field.clone()))
        });
        let (beside, payload_field) = owner.ok_or_else(|| {
            format!(
                "{written}.{}: no variant of {} owns this key",
                field.name, def.name
            )
        })?;
        keys.push(WrittenKey {
            key: field.name.clone(),
            ty: unwrap_option(&field.ty).clone(),
            doc: [field.doc.as_str(), payload_field.doc.as_str()]
                .iter()
                .filter(|doc| !doc.trim().is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" "),
            beside: Some(beside),
        });
    }
    for variant in variants {
        let key = variant.name.to_lowercase();
        if !keys.iter().any(|written| written.key == key) {
            return Err(format!(
                "{}::{} has no key in {written}",
                def.name, variant.name
            ));
        }
    }
    Ok(keys)
}

/// A doc comment as a table cell, or a dash where the source documents nothing.
fn meaning(doc: &str) -> String {
    let shown = cell(doc);
    if shown.is_empty() {
        "—".to_owned()
    } else {
        shown
    }
}

/// The first paragraph of a doc comment.
fn first_paragraph(doc: &str) -> &str {
    doc.split("\n\n").next().unwrap_or(doc)
}

/// Replaces the `*FORMAT` constants a doc comment names with their values.
fn resolve(doc: &str, model: &Model) -> String {
    let mut resolved = doc.to_owned();
    for (name, value) in &model.formats {
        resolved = resolved.replace(&format!("[`{name}`]"), &format!("`{value}`"));
    }
    resolved
}

/// The format a field's doc comment says the field must equal, if it names one.
fn required_format<'a>(doc: &str, model: &'a Model) -> Option<&'a str> {
    model
        .formats
        .iter()
        .find(|(name, _)| doc.contains(&format!("[`{name}`]")))
        .map(|(_, value)| value.as_str())
}

/// `is_identifier`'s doc comment ("Whether `id` is a well-formed identifier: …") as a sentence
/// about identifiers.
fn identifier_sentence(rule: &str) -> String {
    let rule = plain(rule);
    match rule.strip_prefix("Whether `id` is a well-formed identifier: ") {
        Some(rest) => format!("A well-formed identifier is {rest}"),
        None => rule,
    }
}

/// The identifier kinds the given types use.
fn used_identifiers(model: &Model, sections: &[&TypeDef]) -> Result<BTreeSet<String>, String> {
    fn collect(ty: &Ty, out: &mut BTreeSet<String>) {
        match ty {
            Ty::Identifier(name) => {
                out.insert(name.clone());
            }
            Ty::Option(inner) | Ty::List(inner) | Ty::Boxed(inner) => collect(inner, out),
            Ty::Map(key, value) => {
                collect(key, out);
                collect(value, out);
            }
            Ty::Text | Ty::Count | Ty::Any | Ty::Named(_) => {}
        }
    }
    let mut used = BTreeSet::new();
    for def in sections {
        match (&def.reading, &def.shape) {
            (Reading::TryFrom(written), _) => {
                for key in written_keys(model, def, written)? {
                    collect(&key.ty, &mut used);
                }
            }
            (_, Shape::Struct(fields)) => {
                for field in fields {
                    collect(&field.ty, &mut used);
                }
            }
            (_, Shape::Enum(_)) => {}
        }
    }
    Ok(used)
}

fn fields_table(fields: &[source::Field], links: &Links, model: &Model, read: bool) -> String {
    let rows: Vec<Vec<String>> = fields
        .iter()
        .map(|field| {
            // A written document (`canon-decision/1`) leaves out an absent slot and an empty
            // list (`crates/canon/src/eval/decision.rs`), so only its other fields are always
            // there.
            let presence = match (read, field.optional, &field.ty) {
                (false, _, Ty::Option(_)) => "when present",
                (false, _, Ty::List(_)) => "when not empty",
                (false, _, _) => "always",
                (true, true, _) => "optional",
                (true, false, _) => "required",
            };
            vec![
                code(&field.name),
                render_ty(&field.ty, links),
                presence.to_owned(),
                meaning(&resolve(&field.doc, model)),
            ]
        })
        .collect();
    let presence = if read { "Required" } else { "Present" };
    table(&["Key", "Value", presence, "Meaning"], &rows)
}

/// One section per type, at heading `level`.
fn type_sections(
    model: &Model,
    defs: &[&TypeDef],
    links: &Links,
    level: usize,
    root_title: &str,
) -> Result<String, String> {
    let hashes = "#".repeat(level);
    let mut out = String::new();
    for (index, def) in defs.iter().enumerate() {
        let title = if index == 0 {
            root_title.to_owned()
        } else {
            format!("`{}`", def.name)
        };
        out.push_str(&format!(
            "\n{hashes} {title}\n\n{}\n",
            block(&resolve(&def.doc, model), 1)
        ));
        match (&def.reading, &def.shape) {
            (Reading::Derived, Shape::Struct(fields)) => {
                out.push_str(&fields_table(fields, links, model, true));
            }
            (Reading::NotRead, Shape::Struct(fields)) => {
                out.push_str(&fields_table(fields, links, model, false));
            }
            (Reading::NotRead, Shape::Enum(variants)) => {
                let rows: Vec<Vec<String>> = variants
                    .iter()
                    .map(|variant| vec![code(&snake_case(&variant.name)), meaning(&variant.doc)])
                    .collect();
                out.push_str(&table(&["Value", "Meaning"], &rows));
            }
            (Reading::TryFrom(written), _) => {
                if let Some(doc) = model.module_docs.get("predicate.rs")
                    && def.name == "Predicate"
                {
                    out.push_str(&block(doc, 1));
                    out.push('\n');
                }
                let rows: Vec<Vec<String>> = written_keys(model, def, written)?
                    .iter()
                    .map(|key| {
                        let meaning = match &key.beside {
                            Some(beside) => {
                                format!("Allowed only beside `{beside}`. {}", cell(&key.doc))
                                    .trim_end()
                                    .to_owned()
                            }
                            None => meaning(&key.doc),
                        };
                        vec![code(&key.key), render_ty(&key.ty, links), meaning]
                    })
                    .collect();
                out.push_str(&table(&["Key", "Value", "Meaning"], &rows));
            }
            (Reading::Custom, Shape::Enum(variants)) => {
                out.push_str(&format!("{}\n\n", custom(&def.name)?));
                let rows: Vec<Vec<String>> = variants
                    .iter()
                    .map(|variant| vec![code(&variant.name.to_lowercase()), meaning(&variant.doc)])
                    .collect();
                out.push_str(&table(&["Value", "Meaning"], &rows));
            }
            _ => {
                return Err(format!(
                    "{}: canon-docs cannot describe how this type is read",
                    def.name
                ));
            }
        }
    }
    Ok(out)
}

/// A variant name as Canon writes it: `RevisionMismatch` is `revision_mismatch`.
fn snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn identifiers_section(model: &Model, defs: &[&TypeDef], what: &str) -> Result<String, String> {
    let mut out = format!(
        "\n## Identifiers\n\nAn identifier is text. {} Each kind of identifier is its own type, so \
         one cannot stand in for another. These are the kinds {what} use:\n\n",
        cell(&identifier_sentence(&model.identifier_rule)),
    );
    let used = used_identifiers(model, defs)?;
    let rows: Vec<Vec<String>> = model
        .identifiers
        .iter()
        .filter(|(name, _)| used.contains(name) && text_format(name).is_none())
        .map(|(name, doc)| vec![code(name), cell(doc)])
        .collect();
    out.push_str(&table(&["Type", "Identifies"], &rows));
    let formats: Vec<Vec<String>> = model
        .identifiers
        .iter()
        .filter(|(name, _)| used.contains(name))
        .filter_map(|(name, doc)| {
            let (pattern, note) = text_format(name)?;
            Some(vec![
                code(name),
                cell(&format!("{doc} {note}")),
                code(pattern),
            ])
        })
        .collect();
    if !formats.is_empty() {
        out.push_str(&format!(
            "\n## Text formats\n\nThese values are text in a fixed form, not identifiers. Canon \
             refuses text that does not match its pattern, and also checks what the pattern \
             cannot:\n\n{}",
            table(&["Type", "Written as", "Pattern"], &formats)
        ));
    }
    Ok(out)
}

/// The `identifier!` newtypes in `ids.rs` that are not identifiers but text in a fixed form: each
/// with the pattern its JSON Schema states and what Canon checks beyond the pattern.
const TEXT_FORMATS: &[(&str, &str, &str)] = &[
    (
        "Age",
        "^(0|[1-9][0-9]*)[smhd]$",
        "Canon also refuses an age too long to count in seconds.",
    ),
    (
        "Instant",
        "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$",
        "Calendar validity is checked by Canon, not by the pattern: the date must exist (leap \
         years included) and the time be at most 23:59:59.",
    ),
];

/// The pattern and the note of a type [`TEXT_FORMATS`] lists.
fn text_format(name: &str) -> Option<(&'static str, &'static str)> {
    TEXT_FORMATS
        .iter()
        .find(|(format, _, _)| *format == name)
        .map(|(_, pattern, note)| (*pattern, *note))
}

fn schema_link(file: &str) -> String {
    format!("[JSON Schema]({SITE}/schemas/{file})")
}

fn protocol(model: &Model) -> Result<String, String> {
    let format = model.format(PROTOCOL.format)?;
    let defs = sections(model, PROTOCOL.root)?;
    let links: Links = defs
        .iter()
        .map(|def| (def.name.clone(), format!("#{}", section_anchor(&def.name))))
        .collect();
    let mut out = front_matter(
        &format!("{format} reference"),
        format,
        &format!("Every key of a {format} document, generated from Canon's model types."),
    );
    out.push_str(&format!(
        "Generated from the model types in {}: their doc comments, fields, field types and serde \
         attributes. The {} is generated from the same types. For what the parts mean together, \
         read [the language](../concepts/protocols.md); for the documents an evaluation reads and \
         writes, see [evaluation documents](./documents.md).\n\n",
        source_link(source::MODEL_DIR),
        schema_link(PROTOCOL.schema.unwrap_or_default()),
    ));
    out.push_str(&block(first_paragraph(&model.module_docs["present.rs"]), 1));
    out.push('\n');
    out.push_str(&type_sections(model, &defs, &links, 2, "The document")?);
    out.push_str(&identifiers_section(
        model,
        &defs,
        &format!("a `{format}` document"),
    )?);
    Ok(out)
}

fn documents(model: &Model) -> Result<String, String> {
    let on_protocol_page: BTreeSet<String> = sections(model, PROTOCOL.root)?
        .iter()
        .map(|def| def.name.clone())
        .collect();
    let mut shown = BTreeSet::new();
    let mut per_document = Vec::new();
    for document in &EVALUATION_DOCUMENTS {
        let defs: Vec<&TypeDef> = sections(model, document.root)?
            .into_iter()
            .filter(|def| !on_protocol_page.contains(&def.name) && shown.insert(def.name.clone()))
            .collect();
        per_document.push((document, defs));
    }
    let mut links: Links = on_protocol_page
        .iter()
        .map(|name| {
            (
                name.clone(),
                format!("./protocol.md#{}", section_anchor(name)),
            )
        })
        .collect();
    for (_, defs) in &per_document {
        for def in defs {
            links.insert(def.name.clone(), format!("#{}", section_anchor(&def.name)));
        }
    }
    let mut out = front_matter(
        "Evaluation documents",
        "Evaluation documents",
        "The case snapshot and evidence records canon evaluate reads, and the decision it writes.",
    );
    out.push_str(&format!(
        "Generated from the model types in {}. `canon evaluate` reads a compiled protocol, one \
         case snapshot and a set of evidence records, and writes a decision. Every key is listed; \
         a key Canon does not know is refused.\n",
        source_link(source::MODEL_DIR),
    ));
    let mut all_defs = Vec::new();
    for (document, defs) in &per_document {
        let format = model.format(document.format)?;
        let root = model.get(document.root)?;
        out.push_str(&format!("\n## `{format}`\n\n"));
        match document.schema {
            Some(file) => out.push_str(&format!(
                "Read by `canon evaluate`. The {} is generated from the same types.\n",
                schema_link(file)
            )),
            None => out.push_str("Written by `canon evaluate`.\n"),
        }
        let rest: Vec<&TypeDef> = defs
            .iter()
            .copied()
            .filter(|def| def.name != root.name)
            .collect();
        let mut ordered = vec![root];
        ordered.extend(rest);
        out.push_str(&type_sections(
            model,
            &ordered,
            &links,
            3,
            &format!("`{}`", root.name),
        )?);
        all_defs.extend(ordered);
    }
    out.push_str(&identifiers_section(model, &all_defs, "these documents")?);
    Ok(out)
}

fn ty_schema(ty: &Ty) -> Json {
    match ty {
        Ty::Text => obj([("type", json::str("string"))]),
        Ty::Count => obj([("type", json::str("integer")), ("minimum", Json::Int(0))]),
        Ty::Any => obj([]),
        Ty::Identifier(name) if text_format(name).is_some() => {
            obj([("$ref", json::str(format!("#/$defs/{name}")))])
        }
        Ty::Identifier(_) => obj([("$ref", json::str("#/$defs/Identifier"))]),
        Ty::Named(name) => obj([("$ref", json::str(format!("#/$defs/{name}")))]),
        Ty::Option(inner) | Ty::Boxed(inner) => ty_schema(inner),
        Ty::List(inner) => obj([("type", json::str("array")), ("items", ty_schema(inner))]),
        Ty::Map(key, value) => obj([
            ("type", json::str("object")),
            ("propertyNames", ty_schema(key)),
            ("additionalProperties", ty_schema(value)),
        ]),
    }
}

fn with_description(schema: Json, doc: &str) -> Json {
    let text = plain(doc);
    match schema {
        Json::Obj(mut members) if !text.is_empty() => {
            members.insert(0, ("description".to_owned(), json::str(text)));
            Json::Obj(members)
        }
        other => other,
    }
}

fn object_schema(properties: Vec<(String, Json)>, required: Vec<String>) -> Json {
    let mut schema = vec![
        ("type".to_owned(), json::str("object")),
        ("additionalProperties".to_owned(), Json::Bool(false)),
        ("properties".to_owned(), Json::Obj(properties)),
    ];
    if !required.is_empty() {
        schema.push((
            "required".to_owned(),
            Json::Arr(required.into_iter().map(Json::Str).collect()),
        ));
    }
    Json::Obj(schema)
}

fn schema(model: &Model, document: &Document) -> Result<Json, String> {
    let file = document
        .schema
        .ok_or_else(|| format!("{} is not read, so it has no schema", document.root))?;
    let mut defs = vec![(
        "Identifier".to_owned(),
        with_description(
            obj([
                ("type", json::str("string")),
                ("minLength", Json::Int(1)),
                ("pattern", json::str("^\\S+$")),
            ]),
            &format!(
                "{} The pattern checks only for whitespace; Canon also refuses control and \
                 Unicode format characters.",
                identifier_sentence(&model.identifier_rule)
            ),
        ),
    )];
    for (name, _, _) in TEXT_FORMATS {
        if !model
            .identifiers
            .iter()
            .any(|(declared, _)| declared == name)
        {
            return Err(format!(
                "TEXT_FORMATS lists `{name}`, which ids.rs does not declare"
            ));
        }
    }
    let document_defs = sections(model, document.root)?;
    let used = used_identifiers(model, &document_defs)?;
    for (name, doc) in &model.identifiers {
        if let Some((pattern, note)) = text_format(name)
            && used.contains(name)
        {
            defs.push((
                name.clone(),
                with_description(
                    obj([
                        ("type", json::str("string")),
                        ("pattern", json::str(pattern)),
                    ]),
                    &format!("{doc} {note}"),
                ),
            ));
        }
    }
    for def in document_defs {
        let schema = match (&def.reading, &def.shape) {
            (Reading::Derived, Shape::Struct(fields)) => {
                let properties = fields
                    .iter()
                    .map(|field| {
                        let mut schema = ty_schema(&field.ty);
                        if let Some(format) = required_format(&field.doc, model)
                            && let Json::Obj(members) = &mut schema
                        {
                            members.push(("const".to_owned(), json::str(format)));
                        }
                        (
                            field.name.clone(),
                            with_description(schema, &resolve(&field.doc, model)),
                        )
                    })
                    .collect();
                let required = fields
                    .iter()
                    .filter(|field| !field.optional)
                    .map(|field| field.name.clone())
                    .collect();
                object_schema(properties, required)
            }
            (Reading::TryFrom(written), _) => {
                let keys = written_keys(model, def, written)?;
                let forms = keys
                    .iter()
                    .filter(|key| key.beside.is_none())
                    .map(|key| {
                        let mut properties = vec![(
                            key.key.clone(),
                            with_description(ty_schema(&key.ty), &key.doc),
                        )];
                        for modifier in keys
                            .iter()
                            .filter(|modifier| modifier.beside.as_deref() == Some(&key.key))
                        {
                            properties.push((
                                modifier.key.clone(),
                                with_description(ty_schema(&modifier.ty), &modifier.doc),
                            ));
                        }
                        object_schema(properties, vec![key.key.clone()])
                    })
                    .collect();
                obj([("oneOf", Json::Arr(forms))])
            }
            (Reading::Custom, Shape::Enum(variants)) => {
                custom(&def.name)?;
                obj([(
                    "oneOf",
                    Json::Arr(vec![
                        obj([("type", json::str("boolean"))]),
                        obj([(
                            "enum",
                            Json::Arr(
                                variants
                                    .iter()
                                    .map(|variant| json::str(variant.name.to_lowercase()))
                                    .collect(),
                            ),
                        )]),
                    ]),
                )])
            }
            _ => {
                return Err(format!(
                    "{}: canon-docs cannot describe how this type is read",
                    def.name
                ));
            }
        };
        defs.push((
            def.name.clone(),
            with_description(schema, &resolve(&def.doc, model)),
        ));
    }
    let root = model.get(document.root)?;
    Ok(obj([
        ("$comment", json::str(crate::md::MARKER)),
        (
            "$schema",
            json::str("https://json-schema.org/draft/2020-12/schema"),
        ),
        ("$id", json::str(format!("{SITE}/schemas/{file}"))),
        ("title", json::str(model.format(document.format)?)),
        ("description", json::str(plain(&resolve(&root.doc, model)))),
        ("$ref", json::str(format!("#/$defs/{}", document.root))),
        ("$defs", Json::Obj(defs)),
    ]))
}

/// Every JSON Schema, keyed by file name.
fn schemas(model: &Model) -> Result<Vec<(&'static str, String)>, String> {
    std::iter::once(&PROTOCOL)
        .chain(EVALUATION_DOCUMENTS.iter())
        .filter_map(|document| document.schema.map(|file| (file, document)))
        .map(|(file, document)| Ok((file, schema(model, document)?.pretty())))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Pages lifted from module documentation.

fn module_page(
    root: &Path,
    path: &str,
    title: &str,
    label: &str,
    description: &str,
) -> Result<String, String> {
    let doc = source::module_doc(root, path)?;
    let mut out = front_matter(title, label, description);
    out.push_str(&format!(
        "Generated from the module documentation of {}.\n\n",
        source_link(path)
    ));
    out.push_str(&block(&doc, 1));
    Ok(out)
}

fn validation(root: &Path) -> Result<String, String> {
    let path = "crates/canon/src/validate/mod.rs";
    let mut out = module_page(
        root,
        path,
        "Validation",
        "Validation",
        "What canon validate checks, in which order, and every problem code it reports.",
    )?;
    out.push_str("\n## Problem codes\n\n");
    let rows: Vec<Vec<String>> = source::problem_codes(root)?
        .iter()
        .map(|problem| vec![code(&problem.code), cell(&problem.doc)])
        .collect();
    out.push_str(&table(&["Code", "Meaning"], &rows));
    out.push_str(
        "\nA document that is not well-formed YAML, or not shaped like `protocol/1`, fails \
         earlier, while parsing; the command line reports it as `parse`.\n",
    );
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// The worked example, from the investigation fixture.

fn read(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    fs::read_to_string(&path).map_err(|error| format!("reading {}: {error}", path.display()))
}

fn words(predicate: &Predicate) -> String {
    let nested = |inner: &Predicate| match inner {
        Predicate::All(_) | Predicate::Any(_) | Predicate::Not(_) => format!("({})", words(inner)),
        _ => words(inner),
    };
    match predicate {
        Predicate::All(members) if members.is_empty() => "always (an empty `all`)".to_owned(),
        Predicate::Any(members) if members.is_empty() => "never (an empty `any`)".to_owned(),
        Predicate::All(members) => format!(
            "all of: {}",
            members.iter().map(nested).collect::<Vec<_>>().join("; ")
        ),
        Predicate::Any(members) => format!(
            "any of: {}",
            members.iter().map(nested).collect::<Vec<_>>().join("; ")
        ),
        Predicate::Not(inner) => format!("not {}", nested(inner)),
        Predicate::Evidence(matching) => match &matching.result {
            Some(result) => format!(
                "evidence of kind {} with result {}",
                code(matching.kind.as_str()),
                code(result)
            ),
            None => format!("evidence of kind {}", code(matching.kind.as_str())),
        },
        Predicate::Claim(test) => format!(
            "claim {} is {}",
            code(test.claim.as_str()),
            code(&test.is.to_string())
        ),
    }
}

fn describe(description: &Option<String>) -> String {
    description.as_deref().map(cell).unwrap_or_default()
}

fn declarations(protocol: &Protocol) -> String {
    let mut out = String::new();
    let mut section = |title: &str, header: &[&str], rows: Vec<Vec<String>>| {
        out.push_str(&format!("\n### {title}\n\n"));
        if rows.is_empty() {
            out.push_str("None declared.\n");
        } else {
            out.push_str(&table(header, &rows));
        }
    };
    section(
        "Artifacts",
        &["Artifact", "Description"],
        protocol
            .artifacts
            .iter()
            .map(|(id, artifact)| vec![code(id.as_str()), describe(&artifact.description)])
            .collect(),
    );
    section(
        "Evidence kinds",
        &["Evidence kind", "Description"],
        protocol
            .evidence_kinds
            .iter()
            .map(|(id, kind)| vec![code(id.as_str()), describe(&kind.description)])
            .collect(),
    );
    section(
        "Claims",
        &["Claim", "True when", "Description"],
        protocol
            .claims
            .iter()
            .map(|(id, claim)| {
                vec![
                    code(id.as_str()),
                    words(&claim.true_when),
                    describe(&claim.description),
                ]
            })
            .collect(),
    );
    section(
        "Obligations",
        &["Obligation", "Description"],
        protocol
            .obligations
            .iter()
            .map(|(id, obligation)| vec![code(id.as_str()), describe(&obligation.description)])
            .collect(),
    );
    let list = |items: Vec<String>| {
        if items.is_empty() {
            "none".to_owned()
        } else {
            items.join(", ")
        }
    };
    section(
        "Actions",
        &[
            "Action",
            "Precondition",
            "Requires authority for",
            "Effect",
            "May produce",
            "Description",
        ],
        protocol
            .actions
            .iter()
            .map(|(id, action)| {
                vec![
                    code(id.as_str()),
                    action
                        .precondition
                        .as_ref()
                        .map(words)
                        .unwrap_or_else(|| "none: always applies".to_owned()),
                    list(
                        action
                            .requires
                            .iter()
                            .map(|requirement| code(requirement.capability.as_str()))
                            .collect(),
                    ),
                    action
                        .effect
                        .as_ref()
                        .map(|effect| code(effect.as_str()))
                        .unwrap_or_else(|| "none".to_owned()),
                    list(
                        action
                            .may_produce
                            .iter()
                            .map(|production| code(production.evidence.as_str()))
                            .collect(),
                    ),
                    describe(&action.description),
                ]
            })
            .collect(),
    );
    section(
        "Outcomes",
        &["Outcome", "Requires", "Description"],
        protocol
            .outcomes
            .iter()
            .map(|(id, outcome)| {
                vec![
                    code(id.as_str()),
                    words(&outcome.requires),
                    describe(&outcome.description),
                ]
            })
            .collect(),
    );
    out
}

fn invalid_variants(root: &Path) -> Result<String, String> {
    let dir = root.join(INVALID);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|error| format!("reading {}: {error}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".yaml"))
        .collect();
    names.sort();
    let mut rows = Vec::new();
    for name in names {
        let relative = format!("{INVALID}/{name}");
        let text = read(root, &relative)?;
        let link = format!("[`{name}`]({SOURCE}/{relative})");
        match model::parse(&text) {
            Err(error) => rows.push(vec![link, code("parse"), cell(&error.to_string())]),
            Ok(protocol) => match validate::validate(&protocol) {
                Ok(()) => return Err(format!("{relative} is valid, but it is kept as invalid")),
                Err(problems) => {
                    for problem in problems {
                        rows.push(vec![
                            link.clone(),
                            code(problem.code()),
                            cell(&problem.to_string()),
                        ]);
                    }
                }
            },
        }
    }
    Ok(table(&["Variant", "Code", "Problem"], &rows))
}

/// The case snapshot the example evaluates.
const EXAMPLE_CASE: &str = "format: canon-case/1
id: INV-18
protocol: investigation
artifacts:
  explanation:
    revision: r1
";

/// One evidence record of the example: an id, a kind and an optional result.
type ExampleRecord = (&'static str, &'static str, Option<&'static str>);

/// The evidence situations the example evaluates: a label and the records on file, each an id, a
/// kind and an optional result. The values shown for them are computed by Canon's evaluator.
const EXAMPLE_EVIDENCE: [(&str, &[ExampleRecord]); 5] = [
    ("Nothing yet", &[]),
    (
        "A supporting observation only",
        &[("obs-1", "supporting_observation", None)],
    ),
    (
        "A supporting observation, and a falsification attempt the explanation survived",
        &[
            ("obs-1", "supporting_observation", None),
            ("fal-1", "falsification_attempt", Some("survived")),
        ],
    ),
    (
        "A supporting observation, and a falsification attempt that refuted it",
        &[
            ("obs-1", "supporting_observation", None),
            ("fal-1", "falsification_attempt", Some("refuted")),
        ],
    ),
    (
        "A supporting observation, and two falsification attempts that disagree",
        &[
            ("obs-1", "supporting_observation", None),
            ("fal-1", "falsification_attempt", Some("survived")),
            ("fal-2", "falsification_attempt", Some("refuted")),
        ],
    ),
];

fn evidence_record(id: &str, kind: &str, result: Option<&str>) -> String {
    let result = result
        .map(|result| format!("result: {result}\n"))
        .unwrap_or_default();
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: explanation\n\
         subject_revision: r1\n"
    )
}

/// Evaluates the example case under each evidence situation with Canon's own evaluator.
fn evaluations(compiled: &ir::Ir) -> Result<String, String> {
    let case = eval::read_case(EXAMPLE_CASE).map_err(|refusal| refusal.to_string())?;
    let mut rows = Vec::new();
    let mut shown = None;
    for (label, records) in EXAMPLE_EVIDENCE {
        let texts: Vec<String> = records
            .iter()
            .map(|(id, kind, result)| evidence_record(id, kind, *result))
            .collect();
        let evidence = texts
            .iter()
            .map(|text| eval::read_evidence(text))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|refusal| refusal.to_string())?;
        let decision = eval::evaluate(compiled, &case, &evidence)
            .map_err(|refusal| format!("{label}: {} ({})", refusal, refusal.code()))?;
        let values: Vec<String> = decision
            .claims
            .iter()
            .map(|(claim, entry)| {
                format!(
                    "{} is {}",
                    code(claim.as_str()),
                    code(&entry.value.to_string())
                )
            })
            .collect();
        rows.push(vec![cell(label), values.join("; ")]);
        if shown.is_none() && records.len() == 2 {
            shown = Some((texts, eval::render(&decision)));
        }
    }
    let (texts, decision) = shown.ok_or("no example situation has two records")?;
    let mut out = String::from(
        "\n## Evaluating a case\n\n`canon evaluate` reads the compiled protocol, a \
         [`canon-case/1`](./documents.md) snapshot and a directory of \
         [`canon-evidence/1`](./documents.md) records, and prints a `canon-decision/1` \
         document. These values are computed by Canon's evaluator for the case below; see \
         [evaluation](./evaluation.md) for the rules.\n\n",
    );
    out.push_str(&format!(
        "```yaml title=\"case.yaml\"\n{}```\n\n",
        EXAMPLE_CASE
    ));
    out.push_str(&table(&["Evidence on record", "Value"], &rows));
    out.push_str(
        "\nWith a supporting observation and a falsification attempt the explanation survived, \
         the evidence directory holds these two records:\n\n",
    );
    for text in &texts {
        out.push_str(&format!("```yaml\n{text}```\n\n"));
    }
    out.push_str(&format!(
        "and `canon evaluate --ir investigation.ir.json --case case.yaml --evidence evidence/` \
         prints:\n\n```json\n{}\n```\n\n{} Both records are bound to `explanation` at its \
         current revision; a record bound to any other revision would be listed under the claim \
         as excluded and would not count.\n",
        decision.trim_end(),
        sections_sentence(compiled)
    ));
    Ok(out)
}

/// What the example's decision says besides the claims, and which sections it leaves out because
/// the protocol declares nothing for them. Read off the compiled protocol, as the decision is.
fn sections_sentence(compiled: &ir::Ir) -> String {
    let sections = [
        (
            compiled.obligations.is_empty(),
            "obligation",
            "obligations",
            "each declared obligation `open` or `discharged`",
        ),
        (
            compiled.actions.is_empty(),
            "action",
            "actions",
            "each declared action `admissible`, `approval-required` or `blocked` under the \
             authority decisions given (none here)",
        ),
        (
            compiled.outcomes.is_empty(),
            "outcome",
            "outcomes",
            "each declared outcome `legitimate` or `blocked`",
        ),
    ];
    let written: Vec<&str> = sections
        .iter()
        .filter(|(empty, ..)| !empty)
        .map(|(.., gives)| *gives)
        .collect();
    let mut sentence = match written.split_last() {
        None => String::from("The decision gives the claims only."),
        Some((only, [])) => format!("Besides the claims, the decision gives {only}."),
        Some((last, rest)) => format!(
            "Besides the claims, the decision gives {}, and {last}.",
            rest.join(", ")
        ),
    };
    for (_, singular, key, _) in sections.iter().filter(|(empty, ..)| *empty) {
        sentence.push_str(&format!(
            " This protocol declares no {singular}, so the decision has no `{key}` section."
        ));
    }
    sentence.push_str(" See [evaluation](./evaluation.md) for each section.");
    sentence
}

fn example(root: &Path) -> Result<String, String> {
    let text = read(root, FIXTURE)?;
    let protocol = model::parse(&text).map_err(|error| format!("{FIXTURE}: {error}"))?;
    let compiled = ir::compile(&protocol).map_err(|problems| {
        let shown: Vec<String> = problems.iter().map(ToString::to_string).collect();
        format!("{FIXTURE} does not validate: {}", shown.join("; "))
    })?;

    let mut out = front_matter(
        "Worked example: an investigation",
        "An investigation",
        "The investigation protocol, what it declares, and what Canon's validator and compiler make of it.",
    );
    out.push_str(&format!(
        "Generated from {} by running Canon's own parser, validator and compiler over it. The \
         protocol is deliberately not about software: Canon is meant to express any \
         evidence-governed undertaking, and this is the fixture Canon's own tests use.\n\n",
        source_link(FIXTURE)
    ));
    if let Some(description) = &protocol.protocol.description {
        out.push_str(&format!("> {}\n\n", cell(description)));
    }
    out.push_str(&format!(
        "## The protocol\n\n```yaml title=\"{FIXTURE}\"\n{}\n```\n",
        text.trim_end()
    ));
    out.push_str(&format!(
        "\n## What it declares\n\nProtocol {} at revision {}.\n",
        code(protocol.protocol.id.as_str()),
        protocol.protocol.revision
    ));
    out.push_str(&declarations(&protocol));
    out.push_str(&format!(
        "\n## Validation\n\n`canon validate --path {FIXTURE}` accepts it: the \
         validator reports no problem. See [validation](./validation.md) for what it checks.\n"
    ));
    out.push_str(&format!(
        "\n## The compiled form\n\n`canon compile --path {FIXTURE}` prints \
         its [`canon-ir/1`](./canon-ir.md):\n\n```json\n{}\n```\n",
        compiled.canonical_json().trim_end()
    ));
    out.push_str(&evaluations(&compiled)?);
    out.push_str(&format!(
        "\n## Broken variants\n\nThe repository keeps broken copies of this \
         protocol in {}. Each is rejected; these are the problems the validator reports for \
         them.\n\n",
        source_link(INVALID)
    ));
    out.push_str(&invalid_variants(root)?);
    Ok(out)
}
