//! Composition of a `protocol/1` document and the documents it imports into one document
//! (story:protocol-imports), which [`super::compile`] then compiles like any other. Pure: the caller
//! reads the documents and passes them in, and nothing else is read.
//!
//! [`compose`] checks, in this order, stopping after the first step that finds a problem:
//!
//! 1. every document is supplied once: a protocol id two documents have, the root's included, is
//!    refused (`duplicate-import-document`);
//! 2. the `imports` section of each document: an import id or a `refines` entry that is not an
//!    identifier, an import id listed twice and a digest not written `sha256:` and 64 lowercase
//!    hexadecimal digits (`invalid-identifier`, `duplicate-identifier`,
//!    `invalid-import-digest`), each wrapped in `invalid-import`, naming the document, when it is
//!    not the root's;
//! 3. every import has a supplied document (`missing-import`, naming the importing protocol and
//!    the import), and every supplied document is imported, by the root or by a document it
//!    imports (`unused-import`, naming it);
//! 4. the imports form no cycle (`import-cycle`, naming the cycle as a walk from the root meets it,
//!    from the first protocol on it back to that protocol: `a -> b -> a`). The cycle check comes
//!    before any pin is read: a digest is of a document compiled with its imports, so no pin of a
//!    cycle can be right, and a cycle is only ever met with a wrong pin;
//! 5. each document, every import before the documents that import it: first each of its imports'
//!    pins, the revision (`import-revision-mismatch`) and the digest (`import-digest-mismatch`,
//!    printing the pinned and the actual digest) of the supplied document composed with its own
//!    imports; then its composition with the imports' composed documents (`undeclared-refinement`,
//!    `conflicting-imports`, `unrefined-redeclaration`); then, for an imported document, its
//!    validation, each problem wrapped in `invalid-import`.
//!
//! Composition inlines every declaration of every section of an import under its own identifier.
//! Two imports that declare one identifier in one section with identical declarations give one
//! declaration; with different ones, they are refused, unless the importing document redeclares
//! the identifier. Identical means identical `canon-ir/1`: the declarations' entries in the
//! imports' compiled IR are the same JSON, so two declarations that differ only in the ways the
//! `ir` module documentation lists (the order of `all` members, of `may_produce`, of a case input's
//! values, an absent precondition and the empty `all`, ...) are one declaration. A document may
//! redeclare an identifier an import declares only when that import lists it under `refines`, and
//! the redeclaration replaces the imported declaration. A `refines` entry names an identifier of
//! any section, and must name one the import declares, itself or through its own imports. The
//! composed document has the importing document's format and header, and no imports.

use std::collections::{BTreeMap, BTreeSet};

use super::json::Value;
use super::{Ir, compile};
use crate::model::{CaseSection, Declarations, Import, Protocol, ProtocolId};
use crate::validate::{self, Problem, Section};

/// What every digest an import pins begins with.
pub const DIGEST_PREFIX: &str = "sha256:";

/// The digest an import pins for a compiled protocol: [`DIGEST_PREFIX`] followed by the SHA-256 of
/// the IR's canonical JSON ([`Ir::canonical_json`]) in lowercase hexadecimal.
pub fn digest(ir: &Ir) -> String {
    digest_of(&ir.canonical_json())
}

/// [`DIGEST_PREFIX`] followed by the SHA-256 of `canonical_json` in lowercase hexadecimal.
fn digest_of(canonical_json: &str) -> String {
    let mut text = String::from(DIGEST_PREFIX);
    for byte in sha256(canonical_json.as_bytes()) {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

/// Composes `root` with `imports`, the documents it imports directly or through another import,
/// into one document without imports (see the module docs). The order of `imports` changes nothing.
pub fn compose(root: &Protocol, imports: &[Protocol]) -> Result<Protocol, Vec<Problem>> {
    let root_id = &root.protocol.id;
    let mut documents: BTreeMap<&ProtocolId, &Protocol> = BTreeMap::new();
    documents.insert(root_id, root);
    let mut twice = BTreeSet::new();
    for document in imports {
        if documents.insert(&document.protocol.id, document).is_some() {
            twice.insert(&document.protocol.id);
        }
    }
    if !twice.is_empty() {
        return Err(twice
            .into_iter()
            .map(|protocol| Problem::DuplicateImportDocument {
                protocol: protocol.clone(),
            })
            .collect());
    }

    let mut problems = Vec::new();
    for (id, document) in &documents {
        let found = validate::import_section(document);
        if *id == root_id {
            problems.extend(found);
        } else {
            problems.extend(found.into_iter().map(|problem| invalid(id, problem)));
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }

    let mut reached = BTreeSet::new();
    let mut pending = vec![root_id];
    while let Some(id) = pending.pop() {
        if reached.insert(id) {
            pending.extend(
                imported(documents[id])
                    .map(|(import, _)| import)
                    .filter(|import| documents.contains_key(import)),
            );
        }
    }
    for id in &reached {
        for (import, _) in imported(documents[id]) {
            if !documents.contains_key(import) {
                problems.push(Problem::MissingImport {
                    importer: (*id).clone(),
                    import: import.clone(),
                });
            }
        }
    }
    for id in documents.keys() {
        if !reached.contains(id) {
            problems.push(Problem::UnusedImport {
                protocol: (*id).clone(),
            });
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }

    let mut order = Vec::new();
    if let Some(protocols) = walk(
        root_id,
        &documents,
        &mut Vec::new(),
        &mut BTreeSet::new(),
        &mut order,
    ) {
        return Err(vec![Problem::ImportCycle { protocols }]);
    }

    let mut composed: BTreeMap<&ProtocolId, Composed> = BTreeMap::new();
    for id in order {
        let document = documents[id];
        for (import, pin) in imported(document) {
            let Composed {
                protocol: supplied,
                digest: actual,
                ..
            } = &composed[import];
            if pin.revision != supplied.protocol.revision {
                problems.push(Problem::ImportRevisionMismatch {
                    importer: id.clone(),
                    import: import.clone(),
                    pinned: pin.revision,
                    supplied: supplied.protocol.revision,
                });
            }
            if pin.digest != *actual {
                problems.push(Problem::ImportDigestMismatch {
                    importer: id.clone(),
                    import: import.clone(),
                    pinned: pin.digest.clone(),
                    actual: actual.clone(),
                });
            }
        }
        if !problems.is_empty() {
            return Err(problems);
        }
        let protocol = merge(document, &composed)?;
        if id == root_id {
            return Ok(protocol);
        }
        let ir = compile(&protocol).map_err(|found| {
            found
                .into_iter()
                .map(|problem| invalid(id, problem))
                .collect::<Vec<_>>()
        })?;
        let ir = ir.value();
        let digest = digest_of(&ir.render());
        composed.insert(
            id,
            Composed {
                protocol,
                ir,
                digest,
            },
        );
    }
    unreachable!("the walk from the root ends with the root")
}

/// An imported document composed with its own imports: the document, its compiled `canon-ir/1` as
/// a JSON value, and the digest of that IR.
struct Composed {
    protocol: Protocol,
    ir: Value,
    digest: String,
}

/// A problem of an imported document, naming it.
fn invalid(import: &ProtocolId, problem: Problem) -> Problem {
    Problem::InvalidImport {
        import: import.clone(),
        problem: Box::new(problem),
    }
}

/// A document's imports, by protocol id, each once (the first, when one is listed twice).
fn imported(document: &Protocol) -> impl Iterator<Item = (&ProtocolId, &Import)> {
    let mut by_id: BTreeMap<&ProtocolId, &Import> = BTreeMap::new();
    for (id, import) in document.imports.iter() {
        by_id.entry(id).or_insert(import);
    }
    by_id.into_iter()
}

/// Walks the imports from `id` depth first, imports in id order. Returns the first cycle it meets,
/// from the first protocol on it back to that protocol; otherwise appends every protocol it
/// reaches to `order`, each once, every import before the protocols that import it.
fn walk<'a>(
    id: &'a ProtocolId,
    documents: &BTreeMap<&'a ProtocolId, &'a Protocol>,
    path: &mut Vec<&'a ProtocolId>,
    done: &mut BTreeSet<&'a ProtocolId>,
    order: &mut Vec<&'a ProtocolId>,
) -> Option<Vec<ProtocolId>> {
    if let Some(at) = path.iter().position(|on| *on == id) {
        let mut cycle: Vec<ProtocolId> = path[at..].iter().map(|on| (*on).clone()).collect();
        cycle.push(id.clone());
        return Some(cycle);
    }
    if done.contains(id) {
        return None;
    }
    path.push(id);
    for (import, _) in imported(documents[id]) {
        if let Some(cycle) = walk(import, documents, path, done, order) {
            return Some(cycle);
        }
    }
    path.pop();
    done.insert(id);
    order.push(id);
    None
}

/// One import of the document being composed: its id, its entry, its composed document and that
/// document's `canon-ir/1`.
struct Part<'a> {
    id: &'a ProtocolId,
    import: &'a Import,
    composed: &'a Protocol,
    ir: &'a Value,
}

impl Part<'_> {
    fn refines(&self, id: &str) -> bool {
        self.import.refines.iter().any(|refined| refined == id)
    }

    /// The `canon-ir/1` entry of the declaration `id` of `section` in this import's IR.
    fn declaration(&self, section: Section, id: &str) -> Option<&Value> {
        let path: &[&str] = match section {
            Section::Artifact => &["artifacts"],
            Section::EvidenceKind => &["evidence_kinds"],
            Section::Claim => &["claims"],
            Section::Obligation => &["obligations"],
            Section::Action => &["actions"],
            Section::Outcome => &["outcomes"],
            Section::InvalidationRule => &["invalidation"],
            Section::Input => &["case", "inputs"],
            Section::IndependenceDimension => &["independence_dimensions"],
            other => unreachable!("`{other}` is not a declaration section"),
        };
        let mut value = self.ir;
        for key in path.iter().copied().chain([id]) {
            match value {
                Value::Object(members) => value = members.get(key)?,
                _ => return None,
            }
        }
        Some(value)
    }
}

/// Composes one document with the composed documents of its imports.
fn merge(
    document: &Protocol,
    composed: &BTreeMap<&ProtocolId, Composed>,
) -> Result<Protocol, Vec<Problem>> {
    let parts: Vec<Part<'_>> = imported(document)
        .map(|(id, import)| Part {
            id,
            import,
            composed: &composed[id].protocol,
            ir: &composed[id].ir,
        })
        .collect();
    let importer = &document.protocol.id;
    let mut problems = Vec::new();
    for part in &parts {
        for refined in &part.import.refines {
            if !declares(part.composed, refined) {
                problems.push(Problem::UndeclaredRefinement {
                    importer: importer.clone(),
                    import: part.id.clone(),
                    id: refined.clone(),
                });
            }
        }
    }

    let mut merger = Merger {
        importer,
        parts: &parts,
        problems: &mut problems,
    };
    // Each section of every part, in part order.
    macro_rules! of_parts {
        ($section:ident) => {
            parts
                .iter()
                .map(|part| &part.composed.$section)
                .collect::<Vec<_>>()
        };
    }
    let none = Declarations::default();
    let artifacts = merger.section(Section::Artifact, &document.artifacts, of_parts!(artifacts));
    let evidence_kinds = merger.section(
        Section::EvidenceKind,
        &document.evidence_kinds,
        of_parts!(evidence_kinds),
    );
    let claims = merger.section(Section::Claim, &document.claims, of_parts!(claims));
    let obligations = merger.section(
        Section::Obligation,
        &document.obligations,
        of_parts!(obligations),
    );
    let actions = merger.section(Section::Action, &document.actions, of_parts!(actions));
    let outcomes = merger.section(Section::Outcome, &document.outcomes, of_parts!(outcomes));
    let invalidation = merger.section(
        Section::InvalidationRule,
        &document.invalidation,
        of_parts!(invalidation),
    );
    let merged_inputs = merger.section(
        Section::Input,
        document.case.as_ref().map_or(&none, |case| &case.inputs),
        parts
            .iter()
            .map(|part| {
                part.composed
                    .case
                    .as_ref()
                    .map_or(&none, |case| &case.inputs)
            })
            .collect(),
    );
    let independence_dimensions = merger.section(
        Section::IndependenceDimension,
        &document.independence_dimensions,
        of_parts!(independence_dimensions),
    );
    let case = if document.case.is_none() && parts.iter().all(|part| part.composed.case.is_none()) {
        None
    } else {
        Some(CaseSection {
            inputs: merged_inputs,
        })
    };
    let protocol = Protocol {
        format: document.format.clone(),
        protocol: document.protocol.clone(),
        imports: Declarations::default(),
        artifacts,
        evidence_kinds,
        claims,
        obligations,
        actions,
        outcomes,
        invalidation,
        case,
        independence_dimensions,
    };
    if problems.is_empty() {
        Ok(protocol)
    } else {
        Err(problems)
    }
}

/// Whether a document declares `id` in any declaration section.
fn declares(protocol: &Protocol, id: &str) -> bool {
    fn has<K: AsRef<str> + PartialEq, V>(section: &Declarations<K, V>, id: &str) -> bool {
        section.ids().any(|declared| declared.as_ref() == id)
    }
    has(&protocol.artifacts, id)
        || has(&protocol.evidence_kinds, id)
        || has(&protocol.claims, id)
        || has(&protocol.obligations, id)
        || has(&protocol.actions, id)
        || has(&protocol.outcomes, id)
        || has(&protocol.invalidation, id)
        || protocol
            .inputs()
            .any(|(declared, _)| declared.as_str() == id)
        || has(&protocol.independence_dimensions, id)
}

/// Merges one declaration section of a document with the same section of its imports.
struct Merger<'a, 'p> {
    importer: &'a ProtocolId,
    parts: &'a [Part<'p>],
    problems: &'a mut Vec<Problem>,
}

impl Merger<'_, '_> {
    /// The imported declarations, each once, in import order and then source order, but those the
    /// document redeclares, followed by the document's own, in source order.
    /// `theirs` is the section of each part, in part order.
    fn section<K, V>(
        &mut self,
        section: Section,
        own: &Declarations<K, V>,
        theirs: Vec<&Declarations<K, V>>,
    ) -> Declarations<K, V>
    where
        K: AsRef<str> + Clone + PartialEq,
        V: Clone,
    {
        // Each imported identifier with its first declaration, every part that declares it, and
        // whether two parts declare it differently: with different `canon-ir/1` entries.
        let mut imported: Vec<(&K, &V, Vec<&Part<'_>>, bool)> = Vec::new();
        for (part, declarations) in self.parts.iter().zip(theirs) {
            for (id, value) in declarations.iter() {
                match imported.iter_mut().find(|(seen, ..)| *seen == id) {
                    None => imported.push((id, value, vec![part], false)),
                    Some((_, _, providers, conflict)) => {
                        let differs = providers[0].declaration(section, id.as_ref())
                            != part.declaration(section, id.as_ref());
                        if differs && !*conflict && !own.contains(id) {
                            *conflict = true;
                            self.problems.push(Problem::ConflictingImports {
                                importer: self.importer.clone(),
                                section,
                                id: id.as_ref().to_owned(),
                                first: providers[0].id.clone(),
                                second: part.id.clone(),
                            });
                        }
                        providers.push(part);
                    }
                }
            }
        }
        let mut entries = Vec::new();
        for (id, value, providers, _) in imported {
            if own.contains(id) {
                for part in providers {
                    if !part.refines(id.as_ref()) {
                        self.problems.push(Problem::UnrefinedRedeclaration {
                            importer: self.importer.clone(),
                            section,
                            id: id.as_ref().to_owned(),
                            import: part.id.clone(),
                        });
                    }
                }
            } else {
                entries.push((id.clone(), value.clone()));
            }
        }
        entries.extend(own.iter().map(|(id, value)| (id.clone(), value.clone())));
        Declarations::new(entries)
    }
}

/// The SHA-256 digest of `data` (FIPS 180-4).
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut message = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bits.to_be_bytes());
    for block in message.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (word, bytes) in w.iter_mut().zip(block.as_chunks::<4>().0) {
            *word = u32::from_be_bytes(*bytes);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for (k, w) in K.iter().zip(w) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(*k)
                .wrapping_add(w);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (word, add) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *word = word.wrapping_add(add);
        }
    }
    let mut digest = [0u8; 32];
    for (bytes, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *bytes = word.to_be_bytes();
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::sha256;

    fn hex(bytes: [u8; 32]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// The FIPS 180-4 example messages, one block, two blocks, and a message whose padding needs a
    /// block of its own.
    #[test]
    fn sha256_gives_the_published_digests() {
        assert_eq!(
            hex(sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(sha256(&[b'a'; 1_000_000])),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }
}
