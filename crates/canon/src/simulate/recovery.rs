//! A walk's `case_outcome` dimensions per `case_open` subject, and what a revision of that subject
//! does to the children opened about it (story:child-outcome-recovery).
//!
//! The space keeps one `case_outcome` dimension for every artifact no match names, about the least
//! artifact id. A walk of a protocol whose children forms name no `subject` splits it: one
//! dimension for each distinct subject of the actions that open a child case and record in it, so
//! a revision of `a` sets aside only the child outcomes about `a`, and a child's outcome is about
//! the artifact its opening action's `case_open` names. The space does not change; `canon check`
//! counts the one dimension. A protocol with no `case_open`, or whose only subject is the one the
//! space's dimension is about, or in which no action `revises` an artifact and no artifact
//! declares `revised_by`, is not split and is walked and written as before.
//!
//! A child that has not ended when its subject is revised may end against the revision it was
//! opened at, or against the new one. The walk keeps one bit per opening action of a split
//! dimension, set by a revision of the subject while a child of the action has not ended and
//! cleared when none is left: the third value of the action's mark. From it a child's outcome
//! also arrives as an earlier record, written `… earlier`; the step of the opening action leaves
//! it as it is.

use std::collections::{BTreeMap, BTreeSet};

use super::{Added, Layout, Replacing, State, Step};
use crate::check::{Space, Walked};
use crate::ir::Ir;
use crate::model::{ArtifactId, CASE_OUTCOME, EvidenceRecord, one_line};

/// How the walk's dimensions relate to the space's: the first `originals` are the space's, each
/// later one a `case_outcome` dimension about one subject, standing for `source` of it.
pub(super) struct Split {
    pub(super) source: Vec<usize>,
    pub(super) originals: usize,
    /// For each action that opens a child case and whose children record in a split dimension, by
    /// position, the dimension about its `case_open` subject.
    pub(super) redirected: BTreeMap<usize, usize>,
}

/// `dimensions`, the space's, followed by one dimension per `case_open` subject for each
/// `case_outcome` dimension that an opening action about another artifact records in.
pub(super) fn split<'a>(
    ir: &Ir,
    space: &Space<'a>,
    mut dimensions: Vec<Walked<'a>>,
) -> (Vec<Walked<'a>>, Split) {
    let originals = dimensions.len();
    let mut source: Vec<usize> = (0..originals).collect();
    let mut redirected = BTreeMap::new();
    // A revision can move an artifact an action `revises` or another protocol revises
    // (`revised_by`). A protocol with neither has no revision to set a child outcome aside, and
    // the one dimension serves: the walk is as before the split. (The fixture
    // `revise-child-other-subject` revises `a` and opens about `b`, and keeps the split.)
    let revisable = ir.actions.values().any(|action| !action.revises.is_empty())
        || ir
            .artifacts
            .values()
            .any(|artifact| artifact.revised_by.is_some());
    for at in 0..originals {
        if !revisable || dimensions[at].kind.as_str() != CASE_OUTCOME {
            continue;
        }
        let openers: Vec<(usize, &ArtifactId)> = ir
            .actions
            .values()
            .enumerate()
            .filter_map(|(position, action)| {
                let opening = action.case_open.as_ref()?;
                space
                    .produced_by(ir, action)
                    .contains(&at)
                    .then_some((position, &opening.subject))
            })
            .collect();
        let subjects: BTreeSet<&ArtifactId> = openers.iter().map(|(_, subject)| *subject).collect();
        if subjects
            .iter()
            .all(|subject| dimensions[at].subject.as_ref() == Some(*subject))
        {
            continue;
        }
        for subject in subjects {
            let own = &dimensions[at];
            let split = Walked {
                kind: own.kind,
                name: format!(
                    "`{}` about `{}`",
                    one_line(own.kind.as_str()),
                    one_line(subject.as_str())
                ),
                classes: own.classes.clone(),
                profiles: own.profiles.clone(),
                within: own.within.clone(),
                subject: Some(subject.clone()),
                upstreams: own.upstreams.clone(),
                history: own.history,
            };
            for (position, opening) in &openers {
                if *opening == subject {
                    redirected.insert(*position, dimensions.len());
                }
            }
            dimensions.push(split);
            source.push(at);
        }
    }
    (
        dimensions,
        Split {
            source,
            originals,
            redirected,
        },
    )
}

type Held = (usize, usize, u32, Vec<usize>);

impl Split {
    /// Each opener's dimensions that its children record in, the split ones in place of the
    /// space's: opener `i` is `openers[i]`, an action position.
    pub(super) fn redirect(&self, openers: &[usize], produced: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
        produced
            .into_iter()
            .zip(openers)
            .map(|(at, action)| match self.redirected.get(action) {
                Some(split) => at
                    .into_iter()
                    .map(|d| if d == self.source[*split] { *split } else { d })
                    .collect(),
                None => at,
            })
            .collect()
    }

    /// [`Space::walk_evidence`] of a walk's records: a record of a split dimension is the space's
    /// record of its source, about the dimension's subject.
    pub(super) fn evidence(
        &self,
        space: &Space<'_>,
        dimensions: &[Walked<'_>],
        held: &[Held],
        expired: &[(usize, usize)],
    ) -> Vec<EvidenceRecord> {
        if dimensions.len() == self.originals {
            return space.walk_evidence(held, expired);
        }
        let as_source: Vec<Held> = held
            .iter()
            .map(|(d, class, vector, profiles)| {
                (self.source[*d], *class, *vector, profiles.clone())
            })
            .collect();
        let expired_as_source: Vec<(usize, usize)> = expired
            .iter()
            .map(|(d, class)| (self.source[*d], *class))
            .collect();
        let subjects: Vec<Option<ArtifactId>> = held
            .iter()
            .flat_map(|(d, _, _, profiles)| profiles.iter().map(move |_| *d))
            .chain(expired.iter().map(|(d, _)| *d))
            .map(|d| {
                (d >= self.originals)
                    .then(|| dimensions[d].subject.clone())
                    .flatten()
            })
            .collect();
        let mut records = space.walk_evidence(&as_source, &expired_as_source);
        for (record, subject) in records.iter_mut().zip(subjects) {
            if let Some(subject) = subject {
                record.subject = subject;
            }
        }
        records
    }

    /// [`Space::render_walk`] of a walk's state: the records of split dimensions are written
    /// `` evidence `case_outcome` about `b` … ``, after those of the space's dimensions.
    pub(super) fn render(
        &self,
        space: &Space<'_>,
        dimensions: &[Walked<'_>],
        state: &State,
        held: &[Held],
        expired: &[(usize, usize)],
        earlier: &[bool],
    ) -> String {
        if dimensions.len() == self.originals {
            return space.render_walk(state, held, expired, earlier);
        }
        let own_held: Vec<Held> = held
            .iter()
            .filter(|(d, ..)| *d < self.originals)
            .cloned()
            .collect();
        let own_expired: Vec<(usize, usize)> = expired
            .iter()
            .filter(|(d, _)| *d < self.originals)
            .copied()
            .collect();
        let inner = |text: String| text[1..text.len() - 1].to_owned();
        let with = inner(space.render_walk(state, &own_held, &own_expired, earlier));
        let others = inner(space.render_walk(state, &[], &[], earlier));
        let records = if others.is_empty() {
            with.as_str()
        } else {
            with[..with.len() - others.len()].trim_end_matches(", ")
        };
        let mut split: Vec<(usize, usize, u32, Option<usize>)> = held
            .iter()
            .filter(|(d, ..)| *d >= self.originals)
            .flat_map(|(d, class, vector, profiles)| {
                profiles
                    .iter()
                    .map(move |profile| (*d, *class, *vector, Some(*profile)))
            })
            .chain(
                expired
                    .iter()
                    .filter(|(d, _)| *d >= self.originals)
                    .map(|(d, class)| (*d, *class, 0, None)),
            )
            .collect();
        split.sort_by_key(|(d, class, _, profile)| (*d, *class, profile.is_none()));
        let items: Vec<String> = split
            .into_iter()
            .map(|(d, class, vector, profile)| {
                let walked = &dimensions[d];
                let mut item = match walked.classes[class] {
                    Some(result) => {
                        format!("evidence {} result `{}`", walked.name, one_line(result))
                    }
                    None => format!("evidence {}", walked.name),
                };
                match profile {
                    Some(profile) => {
                        item.push_str(&walked.profiles[profile]);
                        let moved: Vec<String> = walked
                            .upstreams
                            .iter()
                            .enumerate()
                            .filter(|(bit, _)| vector & (1 << bit) != 0)
                            .map(|(_, artifact)| format!("`{}`", one_line(artifact.as_str())))
                            .collect();
                        if !moved.is_empty() {
                            item.push_str(&format!(
                                " observed before upstream {} moved",
                                moved.join(", ")
                            ));
                        }
                    }
                    None if earlier.get(d).copied().unwrap_or(false) => item.push_str(" earlier"),
                    None => item.push_str(" expired"),
                }
                item
            })
            .collect();
        let items = items.join(", ");
        let parts: Vec<&str> = [records, items.as_str(), others.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect();
        format!("{{{}}}", parts.join(", "))
    }
}

impl Layout<'_, '_> {
    /// The layout in which the opener at position `i` of `openers` has the third value of its
    /// mark when `flagged[i]` is its subject: the openers whose children record in a split
    /// dimension a `children` form reads.
    pub(super) fn flagging(mut self, ir: &Ir, split: &Split) -> Self {
        self.flagged = self
            .openers
            .iter()
            .map(|action| {
                let action_at = ir.actions.values().nth(*action)?;
                let dimension = split.redirected.get(action)?;
                self.dimensions[*dimension]
                    .history
                    .then(|| {
                        action_at
                            .case_open
                            .as_ref()
                            .map(|open| open.subject.clone())
                    })
                    .flatten()
            })
            .collect();
        self
    }

    /// The position of the first word of the third values, after the marks.
    fn flags(&self) -> usize {
        self.marks + (2 * self.openers.len()).div_ceil(32)
    }

    /// How many words the third values take, none when no action opens a child case.
    pub(super) fn flag_words(&self) -> usize {
        self.openers.len().div_ceil(32)
    }

    /// Whether the mark of opener `at` has the third value in `state`: a child that has not
    /// ended may end against a revision of its subject.
    pub(super) fn before(&self, state: &State, at: usize) -> bool {
        self.flagged.get(at).is_some_and(Option::is_some)
            && state[self.flags() + at / 32] & (1 << (at % 32)) != 0
    }

    /// `walked` after a revision of each of `artifacts`: the mark of each opener about one of
    /// them that has a child not ended takes the third value.
    pub(super) fn after_revision(&self, mut walked: State, artifacts: &[&ArtifactId]) -> State {
        for (at, subject) in self.flagged.iter().enumerate() {
            if subject
                .as_ref()
                .is_some_and(|subject| artifacts.contains(&subject))
                && self.mark(&walked, at) > 0
            {
                walked[self.flags() + at / 32] |= 1 << (at % 32);
            }
        }
        walked
    }

    /// `walked` with the third value of the mark of opener `at` cleared.
    pub(super) fn settled(&self, mut walked: State, at: usize) -> State {
        if self.flagged.get(at).is_some() {
            walked[self.flags() + at / 32] &= !(1 << (at % 32));
        }
        walked
    }

    /// `state` with one more record of `class` in `dimension`, held as earlier.
    fn add_earlier(&self, state: &State, dimension: usize, class: usize) -> State {
        let mut walked = state.clone();
        let profiles = self.dimensions[dimension].profiles.len();
        walked[self.slots[dimension][class][0] + profiles / 32] |= 1 << (profiles % 32);
        walked
    }
}

/// The child-outcome steps from the third value of a mark: the child ends against the revision it
/// was opened at, so its record is an earlier one of the dimension. Each is a step of the
/// ordinary one with the record added as earlier, in the same order, after the ordinary ones.
pub(super) fn late(
    layout: &Layout<'_, '_>,
    state: &State,
    child_records: &[Vec<Added>],
    open_class: &[Option<usize>],
    open_held: &dyn Fn(&State) -> bool,
) -> Vec<(State, Step)> {
    let mut steps = Vec::new();
    for (at, action, mark) in layout.raised(state) {
        if !layout.before(state, at) {
            continue;
        }
        for record in &child_records[at] {
            let Added {
                dimension, class, ..
            } = *record;
            if !layout.holds[dimension] {
                continue;
            }
            let open = open_class[dimension].filter(|open| *open != class);
            let lowered: &[u8] = match (open_class[dimension] == Some(class), mark) {
                (true, _) => &[mark],
                (false, 1) => &[0],
                (false, _) => &[1, 2],
            };
            let mut replacings = vec![None];
            if let Some(open) = open {
                let current = layout.present(state, dimension, open);
                let expired = layout.held_expired(state, dimension, open);
                replacings.extend(
                    [(true, false), (false, true), (true, true)]
                        .into_iter()
                        .filter(|(c, e)| (!c || current) && (!e || expired))
                        .map(|(current, expired)| Some(Replacing { current, expired })),
                );
            }
            for replacing in replacings {
                let replaced = match (open, replacing) {
                    (Some(open), Some(replacing)) => {
                        layout.replace(state, dimension, open, replacing)
                    }
                    _ => state.clone(),
                };
                let added = layout.add_earlier(&replaced, dimension, class);
                for lowered in lowered {
                    let walked = layout.marked(&added, at, *lowered);
                    if walked == *state || !open_held(&walked) {
                        continue;
                    }
                    steps.push((
                        walked,
                        Step::Late {
                            action,
                            record: *record,
                            replacing,
                        },
                    ));
                }
            }
        }
    }
    steps
}
