//! The requirement checks `canon conform run --requirements` adds to the runner: the
//! `canon-requirements/1` catalogue read and checked against the registry for coverage, and every
//! evaluate step checked for determinism. The module docs of [`super`] describe both; this file
//! holds them. Pure, like the runner: the caller passes the catalogue's text in.

use std::collections::BTreeSet;
use std::fmt;

use super::{
    EvaluateInputs, REQUIREMENT_PREFIX, Scenario, StepKind, evaluation, first_difference,
    parse_failure,
};
use crate::ir;
use crate::model::{self, REQUIREMENTS_FORMAT, RequirementCatalogue, RequirementStatus};

/// The text does not parse as a `canon-requirements/1` catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogueError {
    message: String,
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CatalogueError {}

fn refuse<T>(message: String) -> Result<T, CatalogueError> {
    Err(CatalogueError { message })
}

/// Parses `canon-requirements/1` catalogue text. A text that does not deserialize is refused by
/// kind and position only, as a registry file is; then the format, each requirement id (`CANON-`
/// followed by an identifier, listed once) and each statement and source (not empty) are checked,
/// in source order, and the first problem is the refusal.
pub fn parse_catalogue(source: &str) -> Result<RequirementCatalogue, CatalogueError> {
    let catalogue: RequirementCatalogue = match serde_yaml_ng::from_str(source) {
        Ok(catalogue) => catalogue,
        Err(error) => {
            return refuse(parse_failure(
                source,
                "not a canon-requirements/1 catalogue",
                error.location(),
            ));
        }
    };
    if catalogue.format != REQUIREMENTS_FORMAT {
        return refuse(format!(
            "format is `{}`, expected `{REQUIREMENTS_FORMAT}`",
            model::one_line(&catalogue.format)
        ));
    }
    let mut seen = BTreeSet::new();
    for (id, requirement) in catalogue.requirements.iter() {
        let shown = model::one_line(id.as_str());
        let named = id
            .as_str()
            .strip_prefix(REQUIREMENT_PREFIX)
            .is_some_and(model::is_identifier);
        if !named {
            return refuse(format!(
                "requirement `{shown}` is not a {REQUIREMENT_PREFIX}* requirement"
            ));
        }
        if !seen.insert(id.as_str()) {
            return refuse(format!("requirement `{shown}` is listed more than once"));
        }
        if requirement.statement.trim().is_empty() {
            return refuse(format!("requirement `{shown}` has an empty statement"));
        }
        if requirement.source.trim().is_empty() {
            return refuse(format!("requirement `{shown}` has an empty source"));
        }
    }
    Ok(catalogue)
}

/// One scenario the registry run parsed, with what the checks need of its run: whether it passed,
/// and its compiled fixture, when the fixture compiled.
pub(super) struct Ran {
    pub(super) scenario: Scenario,
    pub(super) passed: bool,
    pub(super) compiled: Option<ir::Ir>,
}

/// What the checks found: one line per failed check, in the order the module docs give, and the
/// counts of the `requirements:` summary line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Checks {
    pub(super) lines: String,
    pub(super) failures: usize,
    implemented: usize,
    covered: usize,
    planned: usize,
}

impl Checks {
    fn fail(&mut self, line: String) {
        self.failures += 1;
        self.lines.push_str(&line);
        self.lines.push('\n');
    }

    /// `requirements: <n> implemented, <n> covered, <n> planned`, with its newline.
    pub(super) fn summary(&self) -> String {
        format!(
            "requirements: {} implemented, {} covered, {} planned\n",
            self.implemented, self.covered, self.planned
        )
    }
}

/// Checks the registry's scenarios against the catalogue: coverage, in catalogue order (each
/// implemented requirement no passing scenario covers, each planned one a passing scenario
/// covers), then each requirement a scenario covers and the catalogue does not list, in registry
/// order, then each evaluate step whose three runs differ, in registry and step order.
pub(super) fn check(catalogue: &RequirementCatalogue, ran: &[Ran]) -> Checks {
    let mut checks = Checks::default();
    let covered: BTreeSet<&str> = ran
        .iter()
        .filter(|ran| ran.passed)
        .flat_map(|ran| ran.scenario.covers.iter().map(String::as_str))
        .collect();
    for (id, requirement) in catalogue.requirements.iter() {
        let shown = model::one_line(id.as_str());
        let is_covered = covered.contains(id.as_str());
        match requirement.status {
            RequirementStatus::Implemented => {
                checks.implemented += 1;
                if is_covered {
                    checks.covered += 1;
                } else {
                    checks.fail(format!("uncovered: requirement `{shown}`"));
                }
            }
            RequirementStatus::Planned => {
                checks.planned += 1;
                if is_covered {
                    checks.fail(format!("planned requirement `{shown}` is covered"));
                }
            }
        }
    }
    for ran in ran {
        for requirement in &ran.scenario.covers {
            if !catalogue
                .requirements
                .ids()
                .any(|id| id.as_str() == requirement)
            {
                checks.fail(format!(
                    "unlisted: requirement `{}` covered by scenario `{}`",
                    model::one_line(requirement),
                    model::one_line(&ran.scenario.id)
                ));
            }
        }
    }
    for ran in ran {
        let Some(compiled) = &ran.compiled else {
            continue;
        };
        for step in &ran.scenario.steps {
            let StepKind::Evaluate { inputs, .. } = &step.kind else {
                continue;
            };
            if let Some((run, line)) = nondeterminism(compiled, inputs) {
                checks.fail(format!(
                    "nondeterministic: scenario `{}` step `{}`: {run} differs at line {line}",
                    model::one_line(&ran.scenario.id),
                    model::one_line(&step.id)
                ));
            }
        }
    }
    checks
}

/// Runs an evaluate step three times, as written, again as written, and with its `evidence`,
/// `authority` and `decisions` lists reversed, and returns the first run that differs from the
/// first (`rerun` or `permuted`) with the line, counted from 1, where it first differs.
fn nondeterminism(compiled: &ir::Ir, inputs: &EvaluateInputs) -> Option<(&'static str, usize)> {
    let reversed = |list: &[serde_yaml_ng::Value]| list.iter().rev().cloned().collect::<Vec<_>>();
    let permuted = EvaluateInputs {
        case: inputs.case.clone(),
        evidence: reversed(&inputs.evidence),
        authority: inputs.authority.as_deref().map(reversed),
        decisions: inputs.decisions.as_deref().map(reversed),
        at: inputs.at.clone(),
    };
    let written = output(compiled, inputs);
    [
        ("rerun", output(compiled, inputs)),
        ("permuted", output(compiled, &permuted)),
    ]
    .into_iter()
    .find_map(|(run, text)| first_difference(&text, &written).map(|line| (run, line)))
}

/// One run's output, as compared: the canonical `canon-decision/1` document, or the refusal code
/// (`refusal: <code>`), or why a list could not be written back (`unserializable: <why>`).
fn output(compiled: &ir::Ir, inputs: &EvaluateInputs) -> String {
    match evaluation(compiled, inputs) {
        Ok(Ok(decision)) => crate::eval::render(&decision),
        Ok(Err(refusal)) => format!("refusal: {}\n", refusal.code()),
        Err(why) => format!("unserializable: {why}\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "format: canon-requirements/1\nrequirements:\n";

    fn entry(id: &str, status: &str) -> String {
        format!("  {id}:\n    status: {status}\n    statement: s\n    source: design\n")
    }

    #[test]
    fn a_catalogue_parses_in_source_order() {
        let catalogue = parse_catalogue(&format!(
            "{HEAD}{}{}",
            entry("CANON-B-001", "implemented"),
            entry("CANON-A-001", "planned")
        ))
        .expect("parses");
        let ids: Vec<&str> = catalogue.requirements.ids().map(|id| id.as_str()).collect();
        assert_eq!(ids, ["CANON-B-001", "CANON-A-001"]);
        assert_eq!(
            catalogue
                .requirements
                .iter()
                .map(|(_, requirement)| requirement.status)
                .collect::<Vec<_>>(),
            [RequirementStatus::Implemented, RequirementStatus::Planned]
        );
    }

    #[test]
    fn the_catalogue_shape_is_enforced() {
        let cases = [
            (
                format!("{HEAD}{}extra: 1\n", entry("CANON-A-001", "implemented")),
                "not a canon-requirements/1 catalogue at line 7 column 1",
            ),
            (
                format!(
                    "format: canon-requirements/2\nrequirements:\n{}",
                    entry("CANON-A-001", "implemented")
                ),
                "format is `canon-requirements/2`, expected `canon-requirements/1`",
            ),
            (
                format!("{HEAD}{}", entry("CLAIM-001", "implemented")),
                "requirement `CLAIM-001` is not a CANON-* requirement",
            ),
            (
                format!("{HEAD}{}", entry("'CANON-A 1'", "implemented")),
                "requirement `CANON-A 1` is not a CANON-* requirement",
            ),
            (
                format!("{HEAD}{}", entry("CANON-A-001", "done")),
                "not a canon-requirements/1 catalogue at line 4 column 13",
            ),
            (
                format!("{HEAD}  CANON-A-001:\n    status: implemented\n    statement: s\n"),
                "not a canon-requirements/1 catalogue",
            ),
            (
                format!(
                    "{HEAD}  CANON-A-001:\n    status: implemented\n    statement: ' '\n    source: d\n"
                ),
                "requirement `CANON-A-001` has an empty statement",
            ),
            (
                format!(
                    "{HEAD}  CANON-A-001:\n    status: implemented\n    statement: s\n    source: ''\n"
                ),
                "requirement `CANON-A-001` has an empty source",
            ),
            (
                format!(
                    "{HEAD}{}{}",
                    entry("CANON-A-001", "implemented"),
                    entry("CANON-A-001", "planned")
                ),
                "requirement `CANON-A-001` is listed more than once",
            ),
            (
                "format: canon-requirements/1\n".to_owned(),
                "not a canon-requirements/1 catalogue",
            ),
        ];
        for (source, expected) in cases {
            let message = parse_catalogue(&source).expect_err(&source).to_string();
            assert!(message.contains(expected), "{source}\n=> {message}");
        }
    }

    #[test]
    fn a_catalogue_that_does_not_parse_is_reported_by_kind_and_position_not_content() {
        for source in [
            "SECRET-SCALAR\n",
            "format: canon-requirements/1\nrequirements: {CANON-A-001: {status: SECRET}}\n",
            "format: canon-requirements/1\nrequirements: {}\nSECRET-KEY: 1\n",
        ] {
            let message = parse_catalogue(source).expect_err(source).to_string();
            assert!(!message.contains("SECRET"), "{message}");
        }
    }
}
