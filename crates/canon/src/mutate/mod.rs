//! Mutation testing of a scenario set (story:canon-mutate). Pure: the caller reads the protocol,
//! the scenario files and every fixture they name, and passes their text in; `canon mutate` is that
//! caller.
//!
//! [`mutate`] makes every mutant of a `protocol/1` document by the five operators below, runs each
//! mutant against the `canon-conformance/1` scenarios whose fixture is the protocol, and classifies each
//! with the semantic diff ([`diff::diff`]) from the protocol to the mutant. A mutant some scenario
//! fails is **killed**; one no scenario fails **survived**. A mutant is made by changing the parsed
//! model and writing it with [`model::write`], so a mutant is a `protocol/1` document by
//! construction and its site is named from the model; the scenarios read that text where they read
//! the protocol.
//!
//! # The operators
//!
//! Each operator visits sites of the protocol's own declarations: the predicates a claim's
//! `true_when`, an obligation's `discharged_when` and `applicable_when`, an action's `precondition`
//! and an outcome's `requires` hold, and an action's `requires` list. An operator that has no
//! effect at a site makes no mutant there.
//!
//! | Operator | Site | Mutant |
//! |---|---|---|
//! | `drop-conjunct` | each member of each `all`, at any depth, `not` included | the `all` without that member (an `all` of one member becomes `all: []`, which is true) |
//! | `all-to-any` | each `all`, at any depth | an `any` of the same members |
//! | `remove-precondition` | each action that declares a `precondition` | the action without one |
//! | `remove-capability` | each entry of an action's `requires` | the action without that capability requirement (its scope with it) |
//! | `change-result` | each evidence match (`evidence:`) that names a `result` | the match with another result: one mutant per other result an evidence match of the protocol names for the same kind, in sorted order; none when there is no other |
//!
//! Walked past unchanged, though `drop-conjunct` may drop one as a member of an `all`: `any` and
//! `not` themselves; claim tests and input tests; an evidence match without a `result`, and the
//! subject and `independent_on` of every match; the order forms `first`, `before`, `after`,
//! `since` and `descended`, whose results `change-result` neither changes nor counts; `children`
//! forms; an outcome that requires an explicit decision (`requires: {decision: …}`). Untouched
//! altogether: artifacts, evidence kinds (their `max_age` and `supplied_by` included), an action's `effect`, `author`,
//! `may_produce` (with `decided_by`), `records_together`, `revises`, `arguments` and `case_open`,
//! invalidation rules, case inputs, independence dimensions, imports, every description, and the
//! protocol's id and revision.
//!
//! The number of mutants is bounded by the protocol's size: with `N` predicate nodes over every
//! site, `A` actions, `C` capability requirements, `M` evidence matches that name a result and `R`
//! the largest number of results the protocol names for one kind, there are at most
//! `2N + A + C + M·(R−1)`, so at most quadratic in the protocol's size.
//!
//! # Order
//!
//! Mutants come in a fixed order: by operator, in the order of the table; then by site, the
//! sections in the order claims, obligations, actions, outcomes, each declaration in identifier
//! order (byte order of the identifier as written), within a declaration its fields in the order
//! of the table above; then by position, the path from the field's root to the node the operator
//! changes, member indices compared in order (so a node comes before its members), and for
//! `change-result` the other results in sorted order. The same inputs give the same bytes: nothing
//! reads a clock and no map's iteration order reaches the report.
//!
//! # Which scenarios run
//!
//! A scenario runs against the mutants when [`mutate`]'s `names_protocol` says its `fixture` is the
//! protocol file. The mutant's text is handed in for that fixture only; every import, and every
//! other fixture, is read unchanged through the caller's `read_fixture`, once, and the same text is
//! handed to every run. A scenario that names the protocol only under `imports` is not run: an
//! import is pinned by the digest of its compiled `canon-ir/1`, so such a scenario fails on
//! `import-digest-mismatch` for every mutant that changes the IR, whatever the mutant means, and
//! would hide every weak point. Rewriting that pin is not done here. A protocol that imports others
//! is composed with the imports listed by the first scenario, in file order, whose fixture is the
//! protocol, both to compile it and to compile each mutant; its imports are never mutated. The
//! scenarios whose fixture is another file give the same verdict for every mutant and are not run.
//! Each scenario whose fixture is the protocol runs through [`conform::run_registry`], one file at a
//! time in the order given, and a mutant is killed by the first that does not pass.
//!
//! # Classification
//!
//! Each compiled mutant carries the categories of its changes, each once, in the order
//! `TIGHTENING`, `RELAXATION`, `BREAKING`, `EXPANSION`, `NO SEMANTIC CHANGE`; a mutant whose diff
//! holds no change carries `NO SEMANTIC CHANGE`. A mutant is **weakening** when its categories
//! hold `RELAXATION`, and **strengthening** when they are `TIGHTENING` alone; otherwise it is
//! neither. A weakening survivor is a **weak point** of the scenario set. A mutant the validator
//! or the compiler refuses is **invalid**, with the code of each problem, each once, in the order
//! the compiler reports them; it is neither killed nor a survivor, and no scenario runs. A mutant
//! whose written text does not read back as the mutant (a key longer than [`model::write`] keeps)
//! is invalid with the code `unwritable`.
//!
//! # The report
//!
//! One line per mutant, in the order above, then one summary line:
//!
//! ```text
//! killed: <operator> <site>: <categories>[ (weakening|strengthening)]; scenario `<id>` fails
//! survived: <operator> <site>: <categories>[ (strengthening)]; no scenario fails
//! weak point: <operator> <site>: <categories> (weakening); no scenario fails
//! invalid: <operator> <site>: <code>[, <code>…]
//! mutate: <n> mutants, <n> killed, <n> survived (<n> weak points), <n> invalid
//! ```
//!
//! `<site>` is the section and identifier (``claim `explanation.supported` ``), the field
//! (`true_when`, `discharged_when`, `applicable_when`, `precondition`, `requires`), and the path to
//! the node, each step written `all[<i>]` or `any[<i>]` for member `<i>` (from 0) of an `all` or an
//! `any`, or `not`, joined by `.`: `drop-conjunct` names the member it drops (`true_when all[1]`),
//! `all-to-any` the `all` it changes (`true_when` alone for the root), `change-result` the match and
//! ``result `<old>` to `<new>` ``, `remove-precondition` the field, and `remove-capability`
//! ``requires[<i>] capability `<capability>` ``. Weak points count among the survivors.
//!
//! # Refusals
//!
//! Nothing is run, and [`Refusal`] says why, when: the protocol does not parse (`parse`); a
//! scenario file cannot be read or does not parse as a scenario (`scenario-unreadable`); no
//! scenario names the protocol as its fixture (`no-scenario`); an import the first such scenario
//! lists cannot be read or does not parse (`import-unreadable`); the protocol, composed with those
//! imports, does not compile (each problem's own code); or a scenario whose fixture is the protocol does
//! not pass on the protocol itself (`scenario-fails`), since every mutant would then be killed by
//! it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::conform::{self, FixtureError, ScenarioFile};
use crate::diff::{self, Category};
use crate::ir;
use crate::model::{
    self, Action, Claim, Declarations, EvidenceKindId, Obligation, Outcome, OutcomeRequirement,
    Predicate, Protocol,
};
use crate::validate::Problem;

/// A mutation operator, in the order the report lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operator {
    DropConjunct,
    AllToAny,
    RemovePrecondition,
    RemoveCapability,
    ChangeResult,
}

impl Operator {
    /// The operator's name as the report prints it.
    pub fn as_str(self) -> &'static str {
        match self {
            Operator::DropConjunct => "drop-conjunct",
            Operator::AllToAny => "all-to-any",
            Operator::RemovePrecondition => "remove-precondition",
            Operator::RemoveCapability => "remove-capability",
            Operator::ChangeResult => "change-result",
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One mutant: the operator, where it applied, and the mutated protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mutant {
    pub operator: Operator,
    /// The site as the report names it (see the module docs), every identifier on one line.
    pub site: String,
    pub protocol: Protocol,
}

/// What became of one mutant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The scenario, by id, that failed first.
    Killed { scenario: String },
    /// No scenario failed.
    Survived,
    /// The validator or the compiler refused the mutant: the code of each problem.
    Invalid { codes: Vec<&'static str> },
}

/// One mutant, classified and run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classified {
    pub operator: Operator,
    pub site: String,
    /// The categories of the mutant's semantic diff, each once, in [`Category`] order; empty for
    /// an invalid mutant.
    pub categories: Vec<Category>,
    pub verdict: Verdict,
}

impl Classified {
    /// Whether the mutant's diff holds a `RELAXATION`.
    pub fn is_weakening(&self) -> bool {
        self.categories.contains(&Category::Relaxation)
    }

    /// Whether the mutant's diff holds `TIGHTENING` and nothing else.
    pub fn is_strengthening(&self) -> bool {
        self.categories == [Category::Tightening]
    }

    /// Whether the mutant is a weakening survivor.
    pub fn is_weak_point(&self) -> bool {
        self.verdict == Verdict::Survived && self.is_weakening()
    }

    fn line(&self) -> String {
        let mutant = format!("{} {}", self.operator, self.site);
        let categories = || {
            let names: Vec<&str> = self.categories.iter().map(|c| c.as_str()).collect();
            let label = if self.is_weakening() {
                " (weakening)"
            } else if self.is_strengthening() {
                " (strengthening)"
            } else {
                ""
            };
            format!("{}{label}", names.join(", "))
        };
        match &self.verdict {
            Verdict::Invalid { codes } => format!("invalid: {mutant}: {}\n", codes.join(", ")),
            Verdict::Killed { scenario } => format!(
                "killed: {mutant}: {}; scenario `{}` fails\n",
                categories(),
                model::one_line(scenario)
            ),
            Verdict::Survived if self.is_weakening() => {
                format!(
                    "weak point: {mutant}: {}; no scenario fails\n",
                    categories()
                )
            }
            Verdict::Survived => {
                format!("survived: {mutant}: {}; no scenario fails\n", categories())
            }
        }
    }
}

/// Every mutant, classified and run, in the order the module documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub mutants: Vec<Classified>,
}

impl Report {
    /// The number of weakening survivors.
    pub fn weak_points(&self) -> usize {
        self.mutants.iter().filter(|m| m.is_weak_point()).count()
    }

    /// The report text: one line per mutant, then the summary line.
    pub fn render(&self) -> String {
        let mut text: String = self.mutants.iter().map(Classified::line).collect();
        let count = |verdict: fn(&Verdict) -> bool| {
            self.mutants.iter().filter(|m| verdict(&m.verdict)).count()
        };
        let total = self.mutants.len();
        let weak = self.weak_points();
        text.push_str(&format!(
            "mutate: {total} {}, {} killed, {} survived ({weak} {}), {} invalid\n",
            if total == 1 { "mutant" } else { "mutants" },
            count(|v| matches!(v, Verdict::Killed { .. })),
            count(|v| *v == Verdict::Survived),
            if weak == 1 {
                "weak point"
            } else {
                "weak points"
            },
            count(|v| matches!(v, Verdict::Invalid { .. })),
        ));
        text
    }
}

/// Why nothing was run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The protocol does not parse.
    Parse(model::ParseError),
    /// A scenario file could not be read, or does not parse as a scenario.
    ScenarioUnreadable { path: String, why: String },
    /// No scenario names the protocol as its fixture.
    NoScenario,
    /// An import the first scenario naming the protocol lists could not be read or does not parse.
    ImportUnreadable { path: String, why: String },
    /// The protocol, composed with its imports, does not compile.
    Protocol(Vec<Problem>),
    /// A scenario whose fixture is the protocol does not pass on the protocol itself: its report line.
    ScenarioFails { line: String },
}

impl Refusal {
    /// Each line of the refusal: a stable machine-readable code and a one-line message. One line,
    /// except for a protocol that does not compile, which gives one per problem.
    pub fn lines(&self) -> Vec<(&'static str, String)> {
        match self {
            Refusal::Parse(error) => vec![("parse", error.to_string())],
            Refusal::ScenarioUnreadable { path, why } => vec![(
                "scenario-unreadable",
                format!("{}: {why}", model::one_line(path)),
            )],
            Refusal::NoScenario => vec![(
                "no-scenario",
                "no scenario names the protocol as its fixture".to_owned(),
            )],
            Refusal::ImportUnreadable { path, why } => vec![(
                "import-unreadable",
                format!("import `{}`: {why}", model::one_line(path)),
            )],
            Refusal::Protocol(problems) => problems
                .iter()
                .map(|problem| (problem.code(), problem.to_string()))
                .collect(),
            Refusal::ScenarioFails { line } => vec![(
                "scenario-fails",
                format!("the scenarios do not pass on the protocol itself: {line}"),
            )],
        }
    }
}

/// Mutates `protocol` (its text), runs every mutant against the scenarios of `files` whose fixture it is,
/// and classifies each (see the module docs). `names_protocol` says whether a path a scenario
/// names is the protocol file; `read_fixture` returns the text of any other fixture or import a
/// scenario names, as for [`conform::run_registry`]. Neither is called twice with one path.
pub fn mutate<P, R>(
    protocol: &str,
    files: &[ScenarioFile],
    mut names_protocol: P,
    read_fixture: R,
) -> Result<Report, Refusal>
where
    P: FnMut(&str) -> bool,
    R: FnMut(&str) -> Result<String, FixtureError>,
{
    let model = model::parse(protocol).map_err(Refusal::Parse)?;

    // The scenarios whose fixture is the protocol, in file order, each with its file.
    let mut is_protocol: BTreeMap<String, bool> = BTreeMap::new();
    let mut reading: Vec<(conform::Scenario, &ScenarioFile)> = Vec::new();
    for file in files {
        let text = file
            .text
            .as_ref()
            .map_err(|why| Refusal::ScenarioUnreadable {
                path: file.path.clone(),
                why: model::one_line(why),
            })?;
        let scenario = conform::parse(text).map_err(|error| Refusal::ScenarioUnreadable {
            path: file.path.clone(),
            why: error.to_string(),
        })?;
        // Only a scenario's fixture is decided: a scenario that names the protocol only as an
        // import pins it by digest, so it fails on every mutant that changes the IR, whatever the
        // mutant means, and is not run.
        let fixture = &scenario.fixture;
        let reads = *is_protocol
            .entry(fixture.clone())
            .or_insert_with(|| names_protocol(fixture));
        if reads {
            reading.push((scenario, file));
        }
    }
    let Some((first, _)) = reading.first() else {
        return Err(Refusal::NoScenario);
    };

    let mut reader = Reader {
        read: read_fixture,
        cache: BTreeMap::new(),
    };
    let mut imports = Vec::with_capacity(first.imports.len());
    for path in &first.imports {
        let unreadable = |why: String| Refusal::ImportUnreadable {
            path: path.clone(),
            why,
        };
        let text = match reader.get(path) {
            Ok(text) => text,
            Err(FixtureError::Unreadable(why)) => return Err(unreadable(model::one_line(&why))),
            Err(FixtureError::Outside) => {
                return Err(unreadable(
                    "resolves outside the working directory".to_owned(),
                ));
            }
        };
        let document = model::parse(&text).map_err(|_| unreadable("does not parse".to_owned()))?;
        imports.push(document);
    }
    let original = compiled(&model, &imports).map_err(Refusal::Protocol)?;

    // The protocol itself must pass every scenario whose fixture it is, all together, as `canon
    // conform run` runs them. `is_protocol` holds fixture paths only, so the text is handed in for
    // a fixture; an import is the protocol under the same spelling only in a scenario whose fixture
    // is the protocol too, which imports itself and fails here.
    let scenario_files: Vec<ScenarioFile> =
        reading.iter().map(|(_, file)| (*file).clone()).collect();
    let baseline = conform::run_registry(&scenario_files, |path: &str| {
        if is_protocol.get(path).copied().unwrap_or(false) {
            reader.get(path).map(|_| protocol.to_owned())
        } else {
            reader.get(path)
        }
    });
    if !baseline.all_passed() {
        let rendered = baseline.render();
        let line = rendered
            .lines()
            .find(|line| !line.starts_with("passed: "))
            .unwrap_or_default()
            .to_owned();
        return Err(Refusal::ScenarioFails { line });
    }

    let mut classified = Vec::new();
    for mutant in mutants(&model) {
        let text = model::write(&mutant.protocol);
        let verdict_of = |categories, verdict| Classified {
            operator: mutant.operator,
            site: mutant.site.clone(),
            categories,
            verdict,
        };
        if model::parse(&text).as_ref() != Ok(&mutant.protocol) {
            classified.push(verdict_of(
                Vec::new(),
                Verdict::Invalid {
                    codes: vec!["unwritable"],
                },
            ));
            continue;
        }
        let ir = match compiled(&mutant.protocol, &imports) {
            Ok(ir) => ir,
            Err(problems) => {
                let mut codes: Vec<&'static str> = Vec::new();
                for problem in &problems {
                    if !codes.contains(&problem.code()) {
                        codes.push(problem.code());
                    }
                }
                classified.push(verdict_of(Vec::new(), Verdict::Invalid { codes }));
                continue;
            }
        };
        let changes = diff::diff(&original, &ir)
            .expect("a mutant keeps the protocol's id")
            .changes;
        let mut categories: Vec<Category> = changes.iter().map(|change| change.category).collect();
        categories.sort();
        categories.dedup();
        if categories.is_empty() {
            categories.push(Category::NoSemanticChange);
        }
        let mut verdict = Verdict::Survived;
        for (scenario, file) in &reading {
            let report = conform::run_registry(std::slice::from_ref(*file), |path: &str| {
                if path == scenario.fixture {
                    reader.get(path).map(|_| text.clone())
                } else {
                    reader.get(path)
                }
            });
            if !report.all_passed() {
                verdict = Verdict::Killed {
                    scenario: scenario.id.clone(),
                };
                break;
            }
        }
        classified.push(verdict_of(categories, verdict));
    }
    Ok(Report {
        mutants: classified,
    })
}

/// The protocol composed with its imports and compiled.
fn compiled(protocol: &Protocol, imports: &[Protocol]) -> Result<ir::Ir, Vec<Problem>> {
    ir::compose(protocol, imports).and_then(|composed| ir::compile(&composed))
}

/// The caller's fixture reader, each path read once. The protocol file is read through it too, so
/// a protocol path that resolves outside the working directory is refused as any fixture is.
struct Reader<R> {
    read: R,
    cache: BTreeMap<String, Result<String, FixtureError>>,
}

impl<R: FnMut(&str) -> Result<String, FixtureError>> Reader<R> {
    fn get(&mut self, path: &str) -> Result<String, FixtureError> {
        if let Some(found) = self.cache.get(path) {
            return found.clone();
        }
        let found = (self.read)(path);
        self.cache.insert(path.to_owned(), found.clone());
        found
    }
}

/// A section holding predicates, in the order mutants are listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Section {
    Claim,
    Obligation,
    Action,
    Outcome,
}

impl Section {
    fn noun(self) -> &'static str {
        match self {
            Section::Claim => "claim",
            Section::Obligation => "obligation",
            Section::Action => "action",
            Section::Outcome => "outcome",
        }
    }
}

/// A field holding a predicate, or an action's `requires`, in the order mutants are listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Field {
    TrueWhen,
    DischargedWhen,
    ApplicableWhen,
    Precondition,
    Requires,
}

impl Field {
    fn name(self) -> &'static str {
        match self {
            Field::TrueWhen => "true_when",
            Field::DischargedWhen => "discharged_when",
            Field::ApplicableWhen => "applicable_when",
            Field::Precondition => "precondition",
            Field::Requires => "requires",
        }
    }
}

/// Where a mutant applies, as the order of the module docs sorts it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    operator: Operator,
    section: Section,
    id: String,
    field: Field,
    position: Vec<usize>,
    other: String,
}

/// One predicate of the protocol: its section, the declaration's index and id, and its field.
struct Slot<'a> {
    section: Section,
    index: usize,
    id: &'a str,
    field: Field,
    predicate: &'a Predicate,
}

impl Slot<'_> {
    fn site(&self, path: &str) -> String {
        let mut site = format!(
            "{} `{}` {}",
            self.section.noun(),
            model::one_line(self.id),
            self.field.name()
        );
        if !path.is_empty() {
            site.push(' ');
            site.push_str(path);
        }
        site
    }

    /// The protocol with this slot's predicate replaced by `predicate`.
    fn with(&self, protocol: &Protocol, predicate: Predicate) -> Protocol {
        let mut mutated = protocol.clone();
        match (self.section, self.field) {
            (Section::Claim, _) => {
                mutated.claims = replaced(&protocol.claims, self.index, |claim: &mut Claim| {
                    claim.true_when = predicate;
                });
            }
            (Section::Obligation, Field::ApplicableWhen) => {
                mutated.obligations =
                    replaced(&protocol.obligations, self.index, |o: &mut Obligation| {
                        o.applicable_when = Some(predicate);
                    });
            }
            (Section::Obligation, _) => {
                mutated.obligations =
                    replaced(&protocol.obligations, self.index, |o: &mut Obligation| {
                        o.discharged_when = predicate;
                    });
            }
            (Section::Action, _) => {
                mutated.actions = replaced(&protocol.actions, self.index, |a: &mut Action| {
                    a.precondition = Some(predicate);
                });
            }
            (Section::Outcome, _) => {
                mutated.outcomes = replaced(&protocol.outcomes, self.index, |o: &mut Outcome| {
                    o.requires = OutcomeRequirement::Predicate(predicate);
                });
            }
        }
        mutated
    }
}

/// The declarations with the entry at `index` changed by `change`.
fn replaced<K: Clone + PartialEq, V: Clone>(
    declarations: &Declarations<K, V>,
    index: usize,
    change: impl FnOnce(&mut V),
) -> Declarations<K, V> {
    let mut entries: Vec<(K, V)> = declarations
        .iter()
        .map(|(id, value)| (id.clone(), value.clone()))
        .collect();
    if let Some((_, value)) = entries.get_mut(index) {
        change(value);
    }
    Declarations::new(entries)
}

/// Every predicate of the protocol, in source order.
fn slots(protocol: &Protocol) -> Vec<Slot<'_>> {
    let mut slots = Vec::new();
    for (index, (id, claim)) in protocol.claims.iter().enumerate() {
        slots.push(Slot {
            section: Section::Claim,
            index,
            id: id.as_str(),
            field: Field::TrueWhen,
            predicate: &claim.true_when,
        });
    }
    for (index, (id, obligation)) in protocol.obligations.iter().enumerate() {
        slots.push(Slot {
            section: Section::Obligation,
            index,
            id: id.as_str(),
            field: Field::DischargedWhen,
            predicate: &obligation.discharged_when,
        });
        if let Some(predicate) = &obligation.applicable_when {
            slots.push(Slot {
                section: Section::Obligation,
                index,
                id: id.as_str(),
                field: Field::ApplicableWhen,
                predicate,
            });
        }
    }
    for (index, (id, action)) in protocol.actions.iter().enumerate() {
        if let Some(predicate) = &action.precondition {
            slots.push(Slot {
                section: Section::Action,
                index,
                id: id.as_str(),
                field: Field::Precondition,
                predicate,
            });
        }
    }
    for (index, (id, outcome)) in protocol.outcomes.iter().enumerate() {
        if let OutcomeRequirement::Predicate(predicate) = &outcome.requires {
            slots.push(Slot {
                section: Section::Outcome,
                index,
                id: id.as_str(),
                field: Field::Requires,
                predicate,
            });
        }
    }
    slots
}

/// One node of a predicate: its path from the root, as member indices (`not` is member 0) and as
/// the report writes it, and the node.
struct Node<'a> {
    position: Vec<usize>,
    path: String,
    predicate: &'a Predicate,
}

/// Every node of `predicate`, in pre-order.
fn nodes(predicate: &Predicate) -> Vec<Node<'_>> {
    fn walk<'a>(
        predicate: &'a Predicate,
        position: Vec<usize>,
        path: String,
        out: &mut Vec<Node<'a>>,
    ) {
        out.push(Node {
            position: position.clone(),
            path: path.clone(),
            predicate,
        });
        let members: (&str, Vec<&'a Predicate>) = match predicate {
            Predicate::All(members) => ("all", members.iter().collect()),
            Predicate::Any(members) => ("any", members.iter().collect()),
            Predicate::Not(inner) => ("not", vec![&**inner]),
            _ => return,
        };
        let (form, members) = members;
        for (index, member) in members.into_iter().enumerate() {
            let step = if form == "not" {
                "not".to_owned()
            } else {
                format!("{form}[{index}]")
            };
            let mut deeper = position.clone();
            deeper.push(index);
            walk(member, deeper, joined(&path, &step), out);
        }
    }
    let mut out = Vec::new();
    walk(predicate, Vec::new(), String::new(), &mut out);
    out
}

fn joined(path: &str, step: &str) -> String {
    if path.is_empty() {
        step.to_owned()
    } else {
        format!("{path}.{step}")
    }
}

/// `predicate` with the node at `position` replaced by what `change` makes of it.
fn replace_at(
    predicate: &Predicate,
    position: &[usize],
    change: &dyn Fn(&Predicate) -> Predicate,
) -> Predicate {
    let Some((&first, rest)) = position.split_first() else {
        return change(predicate);
    };
    match predicate {
        Predicate::All(members) | Predicate::Any(members) => {
            let members: Vec<Predicate> = members
                .iter()
                .enumerate()
                .map(|(index, member)| {
                    if index == first {
                        replace_at(member, rest, change)
                    } else {
                        member.clone()
                    }
                })
                .collect();
            if matches!(predicate, Predicate::All(_)) {
                Predicate::All(members)
            } else {
                Predicate::Any(members)
            }
        }
        Predicate::Not(inner) => Predicate::Not(Box::new(replace_at(inner, rest, change))),
        _ => predicate.clone(),
    }
}

/// The results the protocol's evidence matches name, per kind, sorted.
fn results<'a>(slots: &[Slot<'a>]) -> BTreeMap<&'a EvidenceKindId, BTreeSet<&'a str>> {
    let mut found: BTreeMap<&EvidenceKindId, BTreeSet<&str>> = BTreeMap::new();
    for slot in slots {
        slot.predicate.visit(&mut |predicate| {
            if let Predicate::Evidence(matching) = predicate
                && let Some(result) = &matching.result
            {
                found.entry(&matching.kind).or_default().insert(result);
            }
        });
    }
    found
}

/// Every mutant of `protocol` by the five operators, in the order the module documents. A mutant
/// equal to the protocol is not one.
pub fn mutants(protocol: &Protocol) -> Vec<Mutant> {
    let slots = slots(protocol);
    let results = results(&slots);
    let mut found: Vec<(Key, Mutant)> = Vec::new();
    let mut push = |key: Key, site: String, mutated: Protocol| {
        if mutated != *protocol {
            let operator = key.operator;
            found.push((
                key,
                Mutant {
                    operator,
                    site,
                    protocol: mutated,
                },
            ));
        }
    };
    for slot in &slots {
        let key = |operator, position: &[usize], other: &str| Key {
            operator,
            section: slot.section,
            id: slot.id.to_owned(),
            field: slot.field,
            position: position.to_vec(),
            other: other.to_owned(),
        };
        for node in nodes(slot.predicate) {
            match node.predicate {
                Predicate::All(members) => {
                    for dropped in 0..members.len() {
                        let changed = replace_at(slot.predicate, &node.position, &|_| {
                            let mut kept = members.clone();
                            kept.remove(dropped);
                            Predicate::All(kept)
                        });
                        let mut position = node.position.clone();
                        position.push(dropped);
                        push(
                            key(Operator::DropConjunct, &position, ""),
                            slot.site(&joined(&node.path, &format!("all[{dropped}]"))),
                            slot.with(protocol, changed),
                        );
                    }
                    let changed = replace_at(slot.predicate, &node.position, &|_| {
                        Predicate::Any(members.clone())
                    });
                    push(
                        key(Operator::AllToAny, &node.position, ""),
                        slot.site(&node.path),
                        slot.with(protocol, changed),
                    );
                }
                Predicate::Evidence(matching) => {
                    let Some(result) = &matching.result else {
                        continue;
                    };
                    let others = results.get(&matching.kind).into_iter().flatten();
                    for other in others.filter(|other| **other != result.as_str()) {
                        let changed = replace_at(slot.predicate, &node.position, &|_| {
                            let mut changed = matching.clone();
                            changed.result = Some((*other).to_owned());
                            Predicate::Evidence(changed)
                        });
                        let what = format!(
                            "result `{}` to `{}`",
                            model::one_line(result),
                            model::one_line(other)
                        );
                        let path = if node.path.is_empty() {
                            what
                        } else {
                            format!("{} {what}", node.path)
                        };
                        push(
                            key(Operator::ChangeResult, &node.position, other),
                            slot.site(&path),
                            slot.with(protocol, changed),
                        );
                    }
                }
                _ => {}
            }
        }
    }
    for (index, (id, action)) in protocol.actions.iter().enumerate() {
        let key = |operator, position: Vec<usize>| Key {
            operator,
            section: Section::Action,
            id: id.as_str().to_owned(),
            field: if operator == Operator::RemovePrecondition {
                Field::Precondition
            } else {
                Field::Requires
            },
            position,
            other: String::new(),
        };
        let shown = model::one_line(id.as_str());
        if action.precondition.is_some() {
            let mut mutated = protocol.clone();
            mutated.actions = replaced(&protocol.actions, index, |a: &mut Action| {
                a.precondition = None;
            });
            push(
                key(Operator::RemovePrecondition, Vec::new()),
                format!("action `{shown}` precondition"),
                mutated,
            );
        }
        for (at, requirement) in action.requires.iter().enumerate() {
            let mut mutated = protocol.clone();
            mutated.actions = replaced(&protocol.actions, index, |a: &mut Action| {
                a.requires.remove(at);
            });
            push(
                key(Operator::RemoveCapability, vec![at]),
                format!(
                    "action `{shown}` requires[{at}] capability `{}`",
                    model::one_line(requirement.capability.as_str())
                ),
                mutated,
            );
        }
    }
    // A stable sort: two mutants with one key (a repeated identifier) keep their source order.
    found.sort_by(|(left, _), (right, _)| left.cmp(right));
    found.into_iter().map(|(_, mutant)| mutant).collect()
}

#[cfg(test)]
mod tests;
