//! Three-valued claim evaluation over an evidence set (the rules are in the module docs of
//! [`super`]).

use std::collections::{BTreeMap, BTreeSet};

use super::Refusal;
use super::independence::Independence;
use super::invalidation::Invalidated;
use crate::ir::Ir;
use crate::model::{
    ArtifactId, ClaimId, EvidenceId, EvidenceKindId, EvidenceMatch, EvidenceRecord, InputId,
    InputTest, InputValue, Predicate, Truth, one_line,
};

/// The evidence a predicate reads: the records the exclusion stages `left`, which evidence matches
/// read, the `history`, every record the evidence check accepted, which the order forms read
/// (`order.rs`), and the case's `independence`, which decides which records an evidence match that
/// demands independence reads (`independence.rs`). The default reads no record and lists no
/// producer.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EvidenceSets<'a> {
    pub(crate) left: &'a [EvidenceRecord],
    pub(crate) history: &'a [EvidenceRecord],
    /// `None` lists no producer: no record satisfies a demand for independence.
    pub(crate) independence: Option<&'a Independence>,
}

impl EvidenceSets<'_> {
    /// Whether `matching` reads `record`: of its kind and subject ([`reads`]), and produced by a
    /// producer the case lists on every dimension the match demands independence on.
    pub(crate) fn of_match(&self, matching: &EvidenceMatch, record: &EvidenceRecord) -> bool {
        reads(&matching.kind, matching.subject.as_ref(), record)
            && independent(self.independence, matching, record)
    }
}

/// Whether `record` satisfies the independence `matching` demands, under the case's
/// `independence`; every record does when the match demands none.
pub(crate) fn independent(
    independence: Option<&Independence>,
    matching: &EvidenceMatch,
    record: &EvidenceRecord,
) -> bool {
    let demanded = matching.independent_on.as_deref();
    match independence {
        Some(independence) => independence.accepts(demanded, record),
        None => demanded.is_none(),
    }
}

impl<'a> EvidenceSets<'a> {
    /// `records` both as what is left and as the history: what an evaluation that excluded nothing
    /// reads.
    #[cfg(test)]
    pub(crate) fn all(records: &'a [EvidenceRecord]) -> Self {
        Self {
            left: records,
            history: records,
            independence: None,
        }
    }
}

/// The value of each case input, as the case snapshot gives it, keyed by input id: what input
/// tests read. The case check (`case.rs`) has refused a case that leaves a declared input out.
pub(crate) type Inputs = BTreeMap<InputId, InputValue>;

/// An input test's value: `true` when the case's value of its input is one the test lists, `false`
/// when it is another. An input the case gives no value for, which only an IR a caller builds can
/// name, is established by nothing and is `unknown`, as an undeclared claim is.
fn input_test(test: &InputTest, inputs: &Inputs) -> Truth {
    match inputs.get(&test.input) {
        Some(value) if test.holds_for(value) => Truth::True,
        Some(_) => Truth::False,
        None => Truth::Unknown,
    }
}

/// The value of every claim `ir` declares, keyed by claim id. The walk recurses through claim
/// tests, so it runs only after `depth.rs` has bounded every claim's effective depth and refused a
/// cycle; the cycle refusal here, `claim-cycle`, is the same one, kept so the walk never loops.
///
/// A claim is evaluated in its own context: no evidence match anywhere in its evaluation reads a
/// record invalidated for it, including the matches of every claim it tests, through any number of
/// claim references. A claim it tests is evaluated, for that use, with the records invalidated for
/// either of them kept out; the value reported for that claim is its own, in its own context.
pub(super) fn values(
    ir: &Ir,
    evidence: EvidenceSets<'_>,
    invalidated: &Invalidated,
    inputs: &Inputs,
) -> Result<BTreeMap<ClaimId, Truth>, Refusal> {
    let mut walk = Walk {
        ir,
        evidence,
        invalidated,
        inputs,
        values: BTreeMap::new(),
        in_progress: Vec::new(),
    };
    let mut reported = BTreeMap::new();
    for id in ir.claims.keys() {
        let own = walk.own(id);
        reported.insert(id.clone(), walk.value_in(id, &own)?);
    }
    Ok(reported)
}

/// The records kept out of an evaluation: those invalidated for the claim whose context it is.
type Context = BTreeSet<EvidenceId>;

/// One evaluation: each claim decided so far in each context, and the claims being decided,
/// outermost first.
struct Walk<'a> {
    ir: &'a Ir,
    evidence: EvidenceSets<'a>,
    /// What the invalidation stage kept from each claim.
    invalidated: &'a Invalidated,
    /// The case's input values, which input tests read.
    inputs: &'a Inputs,
    values: BTreeMap<(ClaimId, Context), Truth>,
    in_progress: Vec<ClaimId>,
}

impl Walk<'_> {
    /// The records invalidated for `id`.
    fn own(&self, id: &ClaimId) -> Context {
        self.invalidated.for_claim(id).cloned().unwrap_or_default()
    }

    /// The value of one claim in one context, memoized, so each claim's predicate is evaluated
    /// once per context. A claim reached again while it is still being decided closes a cycle,
    /// which is refused.
    fn value_in(&mut self, id: &ClaimId, context: &Context) -> Result<Truth, Refusal> {
        let key = (id.clone(), context.clone());
        if let Some(value) = self.values.get(&key) {
            return Ok(*value);
        }
        if let Some(start) = self.in_progress.iter().position(|open| open == id) {
            let path: Vec<String> = self.in_progress[start..]
                .iter()
                .chain([id])
                .map(|claim| one_line(claim.as_str()))
                .collect();
            return Err(Refusal::new(
                "claim-cycle",
                format!("claims test each other in a cycle: {}", path.join(" -> ")),
            ));
        }
        let ir = self.ir;
        let value = match ir.claims.get(id) {
            Some(claim) => {
                self.in_progress.push(id.clone());
                let value = self.predicate(&claim.true_when, context);
                self.in_progress.pop();
                value?
            }
            // A compiled protocol resolves every claim reference; nothing establishes an
            // undeclared one.
            None => Truth::Unknown,
        };
        self.values.insert(key, value);
        Ok(value)
    }

    fn predicate(&mut self, predicate: &Predicate, context: &Context) -> Result<Truth, Refusal> {
        Ok(match predicate {
            Predicate::All(members) => all(&self.members(members, context)?),
            Predicate::Any(members) => any(&self.members(members, context)?),
            Predicate::Not(inner) => not(self.predicate(inner, context)?),
            Predicate::Evidence(matching) => {
                let evidence = self.evidence;
                evidence_match_in(
                    matching,
                    evidence.left.iter().filter(|record| {
                        !context.contains(&record.id) && evidence.of_match(matching, record)
                    }),
                )
            }
            // The history, whatever is invalidated for the claim: order is not invalidated.
            Predicate::First(_) | Predicate::Before(_) => {
                super::order::value(predicate, self.evidence.history).unwrap_or(Truth::Unknown)
            }
            Predicate::Claim(test) => {
                let mut tested_in: Context = self.own(&test.claim);
                tested_in.extend(context.iter().cloned());
                tested(self.value_in(&test.claim, &tested_in)?, test.is)
            }
            Predicate::Input(test) => input_test(test, self.inputs),
        })
    }

    fn members(&mut self, members: &[Predicate], context: &Context) -> Result<Vec<Truth>, Refusal> {
        members
            .iter()
            .map(|member| self.predicate(member, context))
            .collect()
    }
}

/// A predicate's value over claim values already decided, the case's input values and the
/// evidence a section may read: the one evaluator the `obligations`, `actions` and `outcomes`
/// sections use, with the rules claims are evaluated by (the module docs of [`super`]). A claim
/// `claims` does not hold is `unknown`.
pub(super) fn predicate(
    predicate: &Predicate,
    claims: &BTreeMap<ClaimId, Truth>,
    inputs: &Inputs,
    evidence: EvidenceSets<'_>,
) -> Truth {
    let members = |members: &[Predicate]| -> Vec<Truth> {
        members
            .iter()
            .map(|member| self::predicate(member, claims, inputs, evidence))
            .collect()
    };
    match predicate {
        Predicate::All(members_of) => all(&members(members_of)),
        Predicate::Any(members_of) => any(&members(members_of)),
        Predicate::Not(inner) => not(self::predicate(inner, claims, inputs, evidence)),
        Predicate::Evidence(matching) => evidence_match_in(
            matching,
            evidence
                .left
                .iter()
                .filter(|record| evidence.of_match(matching, record)),
        ),
        Predicate::First(_) | Predicate::Before(_) => {
            super::order::value(predicate, evidence.history).unwrap_or(Truth::Unknown)
        }
        Predicate::Claim(test) => tested(
            claims.get(&test.claim).copied().unwrap_or(Truth::Unknown),
            test.is,
        ),
        Predicate::Input(test) => input_test(test, inputs),
    }
}

/// A claim test: `is: true` is the claim's value, `is: false` its negation, and `is: unknown`
/// whether it is undecided, which is itself decided.
fn tested(value: Truth, is: Truth) -> Truth {
    match is {
        Truth::True => value,
        Truth::False => not(value),
        Truth::Unknown if value == Truth::Unknown => Truth::True,
        Truth::Unknown => Truth::False,
    }
}

fn all(values: &[Truth]) -> Truth {
    if values.contains(&Truth::False) {
        Truth::False
    } else if values.iter().all(|value| *value == Truth::True) {
        Truth::True
    } else {
        Truth::Unknown
    }
}

fn any(values: &[Truth]) -> Truth {
    if values.contains(&Truth::True) {
        Truth::True
    } else if values.iter().all(|value| *value == Truth::False) {
        Truth::False
    } else {
        Truth::Unknown
    }
}

fn not(value: Truth) -> Truth {
    match value {
        Truth::True => Truth::False,
        Truth::False => Truth::True,
        Truth::Unknown => Truth::Unknown,
    }
}

/// Whether an evidence match of `kind`, naming `subject` or none, reads `record`: a record of its
/// kind and, when it names a subject, about that artifact. A record about another artifact is not
/// of the match, so it neither establishes nor contradicts it. The one rule claim values, the
/// excluded evidence a claim lists, the `present` of an evidence reason and the explanation
/// (`crate::explain`) all use.
pub(crate) fn reads(
    kind: &EvidenceKindId,
    subject: Option<&ArtifactId>,
    record: &EvidenceRecord,
) -> bool {
    record.kind == *kind && subject.is_none_or(|subject| record.subject == *subject)
}

#[cfg(test)]
fn evidence_match(matching: &EvidenceMatch, evidence: &[EvidenceRecord]) -> Truth {
    evidence_match_in(matching, evidence.iter())
}

/// An evidence match's value over the records `evidence` yields, of those of its kind and subject:
/// the caller has already kept out every record the match's independence demand does not read.
fn evidence_match_in<'r>(
    matching: &EvidenceMatch,
    evidence: impl Iterator<Item = &'r EvidenceRecord>,
) -> Truth {
    let mut of_kind = evidence
        .filter(|record| reads(&matching.kind, matching.subject.as_ref(), record))
        .peekable();
    if of_kind.peek().is_none() {
        return Truth::Unknown;
    }
    let Some(result) = &matching.result else {
        return Truth::True;
    };
    let (with, without) = of_kind.fold((0usize, 0usize), |(with, without), record| {
        if record.result.as_ref() == Some(result) {
            (with + 1, without)
        } else {
            (with, without + 1)
        }
    });
    match (with, without) {
        (_, 0) => Truth::True,
        (0, _) => Truth::False,
        _ => Truth::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ArtifactId, EvidenceId, EvidenceKindId, Revision};

    fn record(kind: &str, result: Option<&str>) -> EvidenceRecord {
        EvidenceRecord {
            format: "canon-evidence/1".to_owned(),
            producer: None,
            id: EvidenceId::new(format!("{kind}-{result:?}")),
            kind: EvidenceKindId::new(kind),
            result: result.map(str::to_owned),
            subject: ArtifactId::new("a"),
            subject_revision: Revision::new("r1"),
            observed_at: None,
            upstream_revisions: crate::model::Declarations::default(),
        }
    }

    fn matching(kind: &str, result: Option<&str>) -> EvidenceMatch {
        EvidenceMatch {
            kind: EvidenceKindId::new(kind),
            independent_on: None,
            result: result.map(str::to_owned),
            subject: None,
        }
    }

    fn about(subject: &str, record: EvidenceRecord) -> EvidenceRecord {
        EvidenceRecord {
            subject: ArtifactId::new(subject),
            ..record
        }
    }

    /// A match that names a subject reads only the records about it; one that names none reads
    /// records about any artifact.
    #[test]
    fn an_evidence_match_with_a_subject_reads_only_records_about_it() {
        let bound = EvidenceMatch {
            subject: Some(ArtifactId::new("a")),
            ..matching("k", Some("pass"))
        };
        let unbound = matching("k", Some("pass"));
        let other_pass = about("b", record("k", Some("pass")));
        let own_fail = about("a", record("k", Some("fail")));
        let own_pass = about("a", record("k", Some("pass")));
        for (evidence, bound_value, unbound_value, why) in [
            (vec![], Truth::Unknown, Truth::Unknown, "no record"),
            (
                vec![other_pass.clone()],
                Truth::Unknown,
                Truth::True,
                "only a record about another artifact",
            ),
            (
                vec![own_fail.clone(), other_pass.clone()],
                Truth::False,
                Truth::Unknown,
                "the other artifact's record is not counted",
            ),
            (
                vec![own_pass, other_pass],
                Truth::True,
                Truth::True,
                "the subject's own record decides",
            ),
        ] {
            assert_eq!(
                evidence_match(&bound, &evidence),
                bound_value,
                "bound: {why}"
            );
            assert_eq!(
                evidence_match(&unbound, &evidence),
                unbound_value,
                "unbound: {why}"
            );
        }
        let bound_without_result = EvidenceMatch {
            subject: Some(ArtifactId::new("a")),
            ..matching("k", None)
        };
        assert_eq!(
            evidence_match(&bound_without_result, &[about("b", record("k", None))]),
            Truth::Unknown,
            "without a result, a record about another artifact still does not match"
        );
        assert_eq!(
            evidence_match(&bound_without_result, &[own_fail]),
            Truth::True
        );
    }

    #[test]
    fn an_evidence_match_with_a_result_is_three_valued() {
        let m = matching("k", Some("pass"));
        assert_eq!(evidence_match(&m, &[]), Truth::Unknown, "no record");
        assert_eq!(
            evidence_match(&m, &[record("other", Some("pass"))]),
            Truth::Unknown,
            "no record of the kind"
        );
        assert_eq!(
            evidence_match(&m, &[record("k", Some("fail"))]),
            Truth::False,
            "only other results"
        );
        assert_eq!(
            evidence_match(&m, &[record("k", None)]),
            Truth::False,
            "a record without a result does not have the result"
        );
        assert_eq!(
            evidence_match(&m, &[record("k", Some("pass")), record("k", Some("pass"))]),
            Truth::True,
            "only that result"
        );
        assert_eq!(
            evidence_match(&m, &[record("k", Some("pass")), record("k", Some("fail"))]),
            Truth::Unknown,
            "records disagree"
        );
    }

    #[test]
    fn an_evidence_match_without_a_result_needs_only_a_record_of_the_kind() {
        let m = matching("k", None);
        assert_eq!(evidence_match(&m, &[]), Truth::Unknown);
        assert_eq!(
            evidence_match(&m, &[record("k", Some("fail"))]),
            Truth::True
        );
        assert_eq!(evidence_match(&m, &[record("k", None)]), Truth::True);
    }

    #[test]
    fn connectives_follow_strong_kleene_logic() {
        use Truth::{False as F, True as T, Unknown as U};
        for (members, expected_all, expected_any) in [
            (vec![], T, F),
            (vec![T], T, T),
            (vec![F], F, F),
            (vec![U], U, U),
            (vec![T, U], U, T),
            (vec![F, U], F, U),
            (vec![T, F], F, T),
            (vec![T, T], T, T),
            (vec![F, F], F, F),
        ] {
            assert_eq!(all(&members), expected_all, "all {members:?}");
            assert_eq!(any(&members), expected_any, "any {members:?}");
        }
        assert_eq!(not(T), F);
        assert_eq!(not(F), T);
        assert_eq!(not(U), U);
    }

    /// The section evaluator gives each claim's own predicate the value claim evaluation gave the
    /// claim, for every evidence set: one set of rules, two callers.
    #[test]
    fn the_section_evaluator_agrees_with_claim_evaluation() {
        let protocol = crate::model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}, l: {}}\n\
             claims:\n\
             \x20\x20a: {true_when: {any: [{evidence: {kind: k, result: pass}}, {not: {evidence: {kind: l}}}]}}\n\
             \x20\x20b: {true_when: {all: [{claim: a}, {claim: a, is: unknown}]}}\n\
             \x20\x20c: {true_when: {not: {claim: b, is: false}}}\n\
             \x20\x20d: {true_when: {any: [{claim: c, is: unknown}, {evidence: {kind: l}}]}}\n",
        )
        .expect("parses");
        let ir = crate::ir::compile(&protocol).expect("compiles");
        for evidence in [
            vec![],
            vec![record("k", Some("pass"))],
            vec![record("k", Some("fail")), record("l", None)],
            vec![record("k", Some("pass")), record("k", Some("fail"))],
            vec![record("l", None)],
        ] {
            let none = Inputs::new();
            let claims = values(
                &ir,
                EvidenceSets::all(&evidence),
                &Invalidated::default(),
                &none,
            )
            .expect("acyclic");
            for (id, claim) in &ir.claims {
                assert_eq!(
                    predicate(
                        &claim.true_when,
                        &claims,
                        &none,
                        EvidenceSets::all(&evidence)
                    ),
                    claims[id],
                    "{id:?} over {evidence:?}"
                );
            }
        }
    }

    #[test]
    fn the_section_evaluator_reads_a_claim_it_was_not_given_as_unknown() {
        let test = |is| {
            Predicate::Claim(crate::model::ClaimTest {
                claim: ClaimId::new("missing"),
                is,
            })
        };
        let none = BTreeMap::new();
        let inputs = Inputs::new();
        assert_eq!(
            predicate(&test(Truth::True), &none, &inputs, EvidenceSets::default()),
            Truth::Unknown
        );
        assert_eq!(
            predicate(
                &test(Truth::Unknown),
                &none,
                &inputs,
                EvidenceSets::default()
            ),
            Truth::True
        );
        let given = BTreeMap::from([(ClaimId::new("missing"), Truth::False)]);
        assert_eq!(
            predicate(
                &test(Truth::False),
                &given,
                &inputs,
                EvidenceSets::default()
            ),
            Truth::True
        );
    }

    /// An input test is `true` for a value it lists and `false` for another, never `unknown` for
    /// an input the case gives; one the case does not give (only a built IR names one) is
    /// `unknown`. Claim evaluation and the section evaluator agree on it.
    #[test]
    fn an_input_test_is_true_exactly_for_the_values_it_lists() {
        let test = Predicate::Input(InputTest {
            input: InputId::new("severity"),
            r#in: vec![InputValue::new("high"), InputValue::new("critical")],
        });
        let given =
            |value: &str| Inputs::from([(InputId::new("severity"), InputValue::new(value))]);
        let claims = BTreeMap::new();
        assert_eq!(
            predicate(&test, &claims, &given("high"), EvidenceSets::default()),
            Truth::True
        );
        assert_eq!(
            predicate(&test, &claims, &given("low"), EvidenceSets::default()),
            Truth::False
        );
        assert_eq!(
            predicate(&test, &claims, &Inputs::new(), EvidenceSets::default()),
            Truth::Unknown
        );
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 case: {inputs: {severity: {values: [low, high]}}}\n\
                 claims: {severe: {true_when: {input: severity, in: [high]}}, \
                 mild: {true_when: {not: {claim: severe}}}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        for (value, severe, mild) in [
            ("high", Truth::True, Truth::False),
            ("low", Truth::False, Truth::True),
        ] {
            let decided = values(
                &ir,
                EvidenceSets::default(),
                &Invalidated::default(),
                &given(value),
            )
            .expect("acyclic");
            assert_eq!(decided[&ClaimId::new("severe")], severe, "{value}");
            assert_eq!(decided[&ClaimId::new("mild")], mild, "{value}");
        }
    }
}
