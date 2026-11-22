//! Whether a protocol stays above an organization's floor (story:protocol-floor, Atlas ADR 0086
//! § Decision 2).
//!
//! [`check`] is a pure function of two compiled protocols, `canon-ir/1` as [`crate::ir::compile`]
//! returns it: the floor and the protocol. It classifies every change from the floor to the
//! protocol with [`crate::diff::diff`] and adds no category of its own. The protocol stays above
//! the floor when every change is `TIGHTENING`, `EXPANSION` or `NO SEMANTIC CHANGE`: only
//! tightening and expansion pass. It is below the floor when any change is `RELAXATION` or
//! `BREAKING`, and the verdict holds each of those, in the classifier's order.
//!
//! A floor is an ordinary protocol with an id, a revision and a description of its own, and the
//! classifier refuses two documents of different ids. The header is required of an IR and cannot
//! be dropped, so [`check`] gives the protocol the floor's header before it compares them: the
//! floor and the protocol are then two documents of one protocol whose headers do not differ, and
//! nothing is said about the id, the revision or the description of either. Every other
//! declaration is compared as `canon diff` compares it, with the limits of that classification: a
//! protocol that adds an artifact, a case input or an order form under an `all`, or changes the
//! case an action opens, is `BREAKING`, so below the floor, though it may tighten.
//!
//! One rule is the floor's own (decision D124, story:floor-guards-added-actions): an added action
//! is an `EXPANSION`, but one that repeats an action the floor guards with a capability, with the
//! same precondition and at least what it may produce and revise, and that does not require that
//! capability (at that scope), is a `RELAXATION`: it decides the floor's finding without the
//! capability. An added action with another precondition is not the floor's finding and passes.
//!
//! The protocol does not name its floor; the caller supplies it. Passing a compiled import as the
//! floor checks that a refinement only tightens what it imports.

use std::fmt;

use crate::diff::{self, Category, Change, Refusal};
use crate::ir::{Ir, Scope};
use crate::model::{CapabilityId, Predicate};

/// The floor's answer for one protocol: how many changes of each admitted category it makes, and
/// each change that puts it below the floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// Changes classified `TIGHTENING`.
    pub tightening: usize,
    /// Changes classified `EXPANSION`.
    pub expansion: usize,
    /// Changes classified `NO SEMANTIC CHANGE`.
    pub no_semantic_change: usize,
    /// Every `RELAXATION` and `BREAKING` change, in the classifier's deterministic order.
    pub below: Vec<Change>,
}

impl Verdict {
    /// Whether the protocol stays above the floor: no change is a `RELAXATION` or `BREAKING`.
    pub fn holds(&self) -> bool {
        self.below.is_empty()
    }
}

impl fmt::Display for Verdict {
    /// Above the floor, `floor: holds` and the three counts, one per line; below it,
    /// `floor: breached` and one line per change, ``<CATEGORY>: <section> `<id>` <what>``.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.holds() {
            writeln!(f, "floor: holds")?;
            writeln!(f, "{}: {}", Category::Tightening, self.tightening)?;
            writeln!(f, "{}: {}", Category::Expansion, self.expansion)?;
            writeln!(
                f,
                "{}: {}",
                Category::NoSemanticChange,
                self.no_semantic_change
            )
        } else {
            writeln!(f, "floor: breached")?;
            self.below
                .iter()
                .try_for_each(|change| writeln!(f, "{change}"))
        }
    }
}

/// Classifies every change from `floor` to `protocol` and returns the verdict. The classifier
/// refuses documents of different ids; both are given the floor's header first, so it does not
/// refuse these.
pub fn check(floor: &Ir, protocol: &Ir) -> Result<Verdict, Refusal> {
    let mut protocol = protocol.clone();
    protocol.protocol = floor.protocol.clone();
    let diff = diff::diff(floor, &protocol)?;
    let mut verdict = Verdict {
        tightening: 0,
        expansion: 0,
        no_semantic_change: 0,
        below: Vec::new(),
    };
    for change in diff.changes {
        match change.category {
            Category::Tightening => verdict.tightening += 1,
            Category::Expansion => verdict.expansion += 1,
            Category::NoSemanticChange => verdict.no_semantic_change += 1,
            Category::Relaxation | Category::Breaking => verdict.below.push(change),
        }
    }
    verdict
        .below
        .extend(unguarded_added_actions(floor, &protocol));
    Ok(verdict)
}

/// The capability requirements the floor makes of the action `guarded` that the added action
/// `added` does not make, one `RELAXATION` for each, when `added` does what `guarded` does: it has
/// the same precondition and may produce and revise everything `guarded` may. The floor's finding
/// is what such an action decides, so one that lacks a capability the floor requires of it lets the
/// finding be published, or the artifact revised, by a principal the floor excludes. An action of
/// the floor that no requirement guards, or that declares nothing which could be matched (the
/// always-true precondition, nothing produced, nothing revised), has no finding to repeat.
fn unguarded_added_actions(floor: &Ir, protocol: &Ir) -> Vec<Change> {
    let always = Predicate::All(Vec::new());
    let mut changes = Vec::new();
    for (id, added) in &protocol.actions {
        if floor.actions.contains_key(id) {
            continue;
        }
        let held: Vec<_> = added.requirements().collect();
        for (guarded_id, guarded) in &floor.actions {
            let identifies = guarded.precondition != always
                || !guarded.may_produce.is_empty()
                || !guarded.revises.is_empty();
            let repeats = identifies
                && added.precondition == guarded.precondition
                && guarded
                    .may_produce
                    .iter()
                    .all(|kind| added.may_produce.contains(kind))
                && guarded
                    .revises
                    .iter()
                    .all(|artifact| added.revises.contains(artifact));
            if !repeats {
                continue;
            }
            for requirement in guarded.requirements().filter(|r| !held.contains(r)) {
                changes.push(Change {
                    category: Category::Relaxation,
                    section: "action",
                    id: id.as_str().to_owned(),
                    what: format!(
                        "added without capability `{}`, which the floor requires of `{}`",
                        requirement_name(requirement),
                        guarded_id.as_str()
                    ),
                });
            }
        }
    }
    changes
}

/// A requirement as the classifier names it: the capability, then its scope when it has one,
/// `metrics.read{cluster: prod-eu, namespace: foo}`.
fn requirement_name((capability, scope): (&CapabilityId, &Scope)) -> String {
    if scope.is_empty() {
        return capability.as_str().to_owned();
    }
    let members: Vec<String> = scope
        .iter()
        .map(|(key, value)| format!("{}: {}", key.as_str(), value.as_str()))
        .collect();
    format!("{}{{{}}}", capability.as_str(), members.join(", "))
}
