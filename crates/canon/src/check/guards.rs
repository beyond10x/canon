//! `ordering-guard-failed` (story:ordering-guards-provable-by-check): each action that declares
//! `runs_only_when` is decided over the states of its own space, as the module docs say ("Ordering
//! guards").
//!
//! No evaluation reads the key, so the guard is lowered: the check builds, beside `ir`, a protocol
//! that holds one more action per guard, whose precondition is the guard and which requires
//! nothing. The evaluator gives that action `admissible` exactly in a state where the guard is
//! `true`, read as a precondition is read: over the records the exclusion stages leave, the claims
//! it tests at their values, the case inputs and the order and `children` forms over the history.
//! The claim statuses of a decision would hold the value of a guard only where it is one claim
//! test; lowering it to an action holds every predicate.

use std::collections::BTreeMap;

use super::{
    Enumerated, Finding, Refusal, STATE_BOUND, Slice, Space, Strategy, enumerate, whole_product,
    witness,
};
use crate::ir::{self, Ir};
use crate::model::{AccessClass, ActionId, Author, one_line};

/// `ir` with each guard lowered to an action of its own, and each guarded action with the
/// position of its own status and of its guard's among the actions of that protocol.
pub(super) struct Lowered {
    pub(super) ir: Ir,
    /// Each guarded action in identifier order: its id, its position and its guard's.
    pub(super) guarded: Vec<(ActionId, usize, usize)>,
}

/// `None` when no action of `ir` declares `runs_only_when`: the check is then the one it was
/// before the key existed.
pub(super) fn lower(ir: &Ir) -> Option<Lowered> {
    let guards: Vec<(&ActionId, &crate::model::Predicate)> = ir
        .actions
        .iter()
        .filter_map(|(id, action)| Some((id, action.runs_only_when.as_ref()?)))
        .collect();
    if guards.is_empty() {
        return None;
    }
    let mut lowered = ir.clone();
    let mut names = Vec::new();
    for (id, guard) in &guards {
        // An identifier holds no whitespace (`invalid-identifier`), so this names no declared
        // action; a caller-built IR that has one anyway gets a prime until it does not.
        let mut name = format!("{} runs_only_when", id.as_str());
        while lowered.actions.contains_key(&ActionId::new(name.as_str())) {
            name.push('\'');
        }
        let name = ActionId::new(name.as_str());
        lowered.actions.insert(
            name.clone(),
            ir::Action {
                description: None,
                precondition: (*guard).clone(),
                runs_only_when: None,
                requires: Vec::new(),
                scopes: BTreeMap::new(),
                effect: None,
                author: Author::Program,
                may_produce: Vec::new(),
                judgement_decided: Default::default(),
                records_together: Vec::new(),
                revises: Vec::new(),
                arguments: None,
                case_open: None,
                access: AccessClass::Write,
            },
        );
        names.push(((*id).clone(), name));
    }
    let position = |id: &ActionId| {
        lowered
            .actions
            .keys()
            .position(|key| key == id)
            .expect("declared")
    };
    let guarded = names
        .iter()
        .map(|(id, name)| (id.clone(), position(id), position(name)))
        .collect();
    Some(Lowered {
        ir: lowered,
        guarded,
    })
}

/// The spaces the guards are decided over: a slice of the lowered protocol's space, or `None` for
/// its whole product, each with the positions in [`Lowered::guarded`] of the guards decided over it.
pub(super) type Plan = Vec<(Option<Slice>, Vec<usize>)>;

/// `` action `a` runs_only_when ``, as a bound refusal names the subject.
fn subject(action: &ActionId) -> String {
    format!("action `{}` runs_only_when", one_line(action.as_str()))
}

/// The guards' spaces, each with the guards decided over it, by position in `guarded`: the whole
/// product of the lowered protocol under [`Strategy::Whole`], otherwise each guard's own slice,
/// the action's together with its guard's, one space per distinct slice. A slice above the bound,
/// or under [`Strategy::Whole`] a whole product above it, refuses the check, before any state is
/// evaluated.
pub(super) fn plan(
    lowered: &Lowered,
    space: &Space<'_>,
    strategy: Strategy,
) -> Result<Plan, Refusal> {
    let ir = &lowered.ir;
    if strategy == Strategy::Whole {
        whole_product(ir, space)?;
        return Ok(vec![(None, (0..lowered.guarded.len()).collect())]);
    }
    let action = |at: usize| ir.actions.values().nth(at).expect("declared");
    let mut plan: Plan = Vec::new();
    for (at, (id, own, guard)) in lowered.guarded.iter().enumerate() {
        let slice = space
            .action_slice(ir, action(*own))
            .union(&space.action_slice(ir, action(*guard)));
        let sliced = space.slice(&slice);
        match sliced.size() {
            Some(size) if size <= STATE_BOUND => {}
            size => {
                let count =
                    size.map_or_else(|| format!("more than {}", u128::MAX), |n| n.to_string());
                return Err(Refusal::new(
                    "state-space-bound",
                    format!(
                        "protocol `{}` revision {}: {} reads {count} states, more than the bound \
                         of {STATE_BOUND}: {}",
                        one_line(ir.protocol.id.as_str()),
                        ir.protocol.revision,
                        subject(id),
                        sliced.dimensions()
                    ),
                ));
            }
        }
        match plan.iter_mut().find(|(s, _)| s.as_ref() == Some(&slice)) {
            Some((_, members)) => members.push(at),
            None => plan.push((Some(slice), vec![at])),
        }
    }
    Ok(plan)
}

/// Whether an action of status `action`, whose guard's lowered action has status `guard`, runs
/// while its guard is not true: it runs when `admissible` or `approval-required`, and `blocked`,
/// by its precondition or by a denial, it does not; the guard is true only where its lowered
/// action, which requires nothing, is `admissible`, so `unknown` breaks it as `false` does.
pub(super) fn breaks(action: &str, guard: &str) -> bool {
    matches!(action, "admissible" | "approval-required") && guard != "admissible"
}

/// The `ordering-guard-failed` finding of each guard of `plan`, in identifier order of the
/// guarded actions.
pub(super) fn findings(
    lowered: &Lowered,
    space: &Space<'_>,
    plan: &[(Option<Slice>, Vec<usize>)],
) -> Result<Vec<Finding>, Refusal> {
    let ir = &lowered.ir;
    let mut found: Vec<Option<Finding>> = vec![None; lowered.guarded.len()];
    for (slice, members) in plan {
        let sliced;
        let space = match slice {
            Some(slice) => {
                sliced = space.slice(slice);
                &sliced
            }
            None => space,
        };
        let enumerated = enumerate(ir, space)?;
        for at in members {
            found[*at] = failed(&enumerated, &lowered.guarded[*at]);
        }
    }
    Ok(found.into_iter().flatten().collect())
}

/// The finding of one guarded action over `enumerated`: the state of least weight in which the
/// action runs while its guard is not true.
fn failed(enumerated: &Enumerated<'_, '_>, guarded: &(ActionId, usize, usize)) -> Option<Finding> {
    let (id, own, guard) = guarded;
    let Enumerated {
        space,
        states,
        evaluated,
        ..
    } = enumerated;
    let broken = (0..states.len()).filter(|index| {
        breaks(
            &evaluated[*index].actions[*own],
            &evaluated[*index].actions[*guard],
        )
    });
    let index = witness(space, states, broken)?;
    Some(Finding::OrderingGuardFailed {
        action: id.clone(),
        status: evaluated[index].actions[*own].clone(),
        state: space.render(&states[index]),
    })
}

#[cfg(test)]
mod tests {
    use super::breaks;

    /// An action runs when `admissible` or `approval-required`; `blocked`, by its precondition or
    /// by a denial, it does not, so a denial never breaks a guard. The guard's lowered action
    /// requires nothing, so it is `admissible` where the guard is `true` and `blocked` where it is
    /// `false` or `unknown`, both of which break it.
    #[test]
    fn only_a_running_action_breaks_a_guard_that_is_not_true() {
        for (action, guard, broken) in [
            ("admissible", "blocked", true),
            ("approval-required", "blocked", true),
            ("blocked", "blocked", false),
            ("admissible", "admissible", false),
            ("approval-required", "admissible", false),
            ("blocked", "admissible", false),
        ] {
            assert_eq!(breaks(action, guard), broken, "{action}, guard {guard}");
        }
    }
}
