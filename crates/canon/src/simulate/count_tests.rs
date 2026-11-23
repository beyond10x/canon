//! story:children-form-counts-unopened-children, the correction of wave 14 (findings W14-1 and
//! W14-4): the walk never counts more children than were opened, at the current revision or in
//! all, and a count too great to evaluate is refused before the walk.

use super::*;
use crate::model::ArtifactId;
use crate::{ir, model};

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// Three readers about `sources`, each opened once, an action that revises `sources`, and two
/// counting forms over the readers' records, one at the current revision and one in all. Outcome
/// `early` is legitimate only where three children ended `ok` while reader `c` is unopened, which
/// no case reaches: two children write two records.
const READERS: &str = "format: protocol/1\n\
protocol: {id: early.min, revision: 1}\n\
artifacts: {sources: {}, marks: {}}\n\
evidence_kinds:\n  case_outcome: {}\n  a_opened: {}\n  b_opened: {}\n  c_opened: {}\n\
claims:\n\
\x20 a.opened: {true_when: {evidence: {kind: a_opened, subject: marks}}}\n\
\x20 b.opened: {true_when: {evidence: {kind: b_opened, subject: marks}}}\n\
\x20 c.opened: {true_when: {evidence: {kind: c_opened, subject: marks}}}\n\
\x20 ended.now: {true_when: {children: {in: [ok], subject: sources, revision: current, count: 3}}}\n\
\x20 ended.any: {true_when: {children: {in: [ok], subject: sources, count: 3}}}\n\
actions:\n\
\x20 a.open: {precondition: {claim: a.opened, is: unknown}, case_open: {protocol: reader, subject: sources}, may_produce: [{evidence: a_opened}]}\n\
\x20 b.open: {precondition: {claim: b.opened, is: unknown}, case_open: {protocol: reader, subject: sources}, may_produce: [{evidence: b_opened}]}\n\
\x20 c.open: {precondition: {claim: c.opened, is: unknown}, case_open: {protocol: reader, subject: sources}, may_produce: [{evidence: c_opened}]}\n\
\x20 redo: {precondition: {claim: a.opened}, revises: [sources]}\n\
outcomes:\n\
\x20 fresh: {requires: {claim: ended.now}}\n\
\x20 early: {requires: {all: [{claim: ended.any}, {claim: c.opened, is: unknown}]}}\n";

/// The walk's dimensions of [`READERS`], its base state, and the position of the counted
/// `case_outcome` dimension.
fn readers(ir: &Ir) -> (Vec<Walked<'_>>, State, usize) {
    let space = Space::new(ir);
    let (dimensions, _) = recovery::split(ir, &space, space.walked());
    let base = space.initial_states().remove(0);
    let counted = dimensions
        .iter()
        .position(|walked| walked.count >= 2 && walked.current_count)
        .expect("a dimension counted at the current revision too");
    (dimensions, base, counted)
}

/// The layout of the three readers' marks and counts, every reader recording in `counted`, which
/// holds its earlier records.
fn layout<'d, 'a>(dimensions: &'d [Walked<'a>], base: usize, counted: usize) -> Layout<'d, 'a> {
    let holds: Vec<bool> = (0..dimensions.len()).map(|at| at == counted).collect();
    Layout::holding(dimensions, base, &holds)
        .marking(vec![0, 1, 2])
        .counting(&[vec![counted], vec![counted], vec![counted]])
}

/// W14-1, the table of [`Layout::ended`]: reader `a` opened, `sources` was revised, reader `b`
/// opened, so two children are counted and one of them, `b`, at the current revision. Whatever
/// `open` records a child that ends replaces, the count at the current revision is never raised
/// above what a case holds: it is raised only when the child's own `open` record was the one
/// held as earlier or expired and nothing else (the child was not counted there and now records
/// at the current revision), and lowered when a child that ends against the earlier revision
/// replaces a current one. A step that replaces the current `open` record and the earlier one
/// may be `b`'s, already counted, so it raises nothing.
#[test]
fn a_child_that_ends_is_counted_at_the_current_revision_at_most_once() {
    let ir = compiled(READERS);
    let (dimensions, base, counted) = readers(&ir);
    let layout = layout(&dimensions, base.len(), counted);
    let mut empty = base.clone();
    empty.resize(layout.length(), 0);
    let sources = ArtifactId::new("sources");
    let state = layout.raise(&layout.revise(&layout.raise(&empty, 0), &[&sources]), 1);
    assert_eq!(layout.children(&state, counted), Some((2, Some(1))));
    let table = [
        (None, false, 1),
        (Some((true, false)), false, 1),
        (Some((false, true)), false, 2),
        (Some((true, true)), false, 1),
        (None, true, 1),
        (Some((true, false)), true, 0),
        (Some((false, true)), true, 1),
        (Some((true, true)), true, 0),
    ];
    for (replacing, late, now) in table {
        let replacing = replacing.map(|(current, expired)| Replacing { current, expired });
        assert_eq!(
            layout.children(
                &layout.ended(state.clone(), counted, replacing, late),
                counted
            ),
            Some((2, Some(now))),
            "replacing {replacing:?}, late {late}"
        );
    }
}

/// W14-1, the records [`Layout::with_children`] evaluates: a state that counts two children, both
/// at the current revision, and holds a current `ok` record and an earlier one is evaluated with
/// two records, never three, whatever the count at the current revision says.
#[test]
fn a_state_is_evaluated_with_no_more_records_than_children() {
    let ir = compiled(READERS);
    let (dimensions, base, counted) = readers(&ir);
    let layout = layout(&dimensions, base.len(), counted);
    let mut empty = base.clone();
    empty.resize(layout.length(), 0);
    let ok = dimensions[counted]
        .classes
        .iter()
        .position(|class| *class == Some("ok"))
        .expect("an `ok` class");
    let sources = ArtifactId::new("sources");
    let mut state = layout.add(
        &layout.revise(&layout.add(&empty, counted, ok, 0), &[&sources]),
        counted,
        ok,
        0,
    );
    let at = layout.counters[counted].expect("a counter");
    state[at] = 2;
    state[at + 1] = 2;
    assert_eq!(layout.children(&state, counted), Some((2, Some(2))));
    let (held, expired) = layout.with_children(&state, layout.held(&state), layout.expired(&state));
    let current: usize = held
        .iter()
        .filter(|entry| entry.0 == counted)
        .map(|entry| entry.3.len())
        .sum();
    let earlier = expired.iter().filter(|(at, _)| *at == counted).count();
    assert_eq!((current, earlier), (1, 1));
}

/// A protocol whose one opener stays admissible, so it opens a child at every step, and whose
/// outcome counts `count` children.
fn unguarded(count: u64) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{sources: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         actions:\n  open: {{case_open: {{protocol: reader, subject: sources}}}}\n\
         outcomes:\n  done: {{requires: {{children: {{in: [passed], subject: sources, count: \
         {count}}}}}}}\n"
    )
}

/// W14-4: each state is evaluated with a record per child it counts, so a walk's cost grows with
/// the square of the count (count 1000: 53 s in a debug build). A `children` form that counts more
/// than 64 children is refused before the walk (`child-count-bound`), naming the first
/// declaration that holds one; a count of 64 is walked.
#[test]
fn a_count_above_the_bound_is_refused_naming_its_declaration() {
    let refusal = simulate(&compiled(&unguarded(65))).expect_err("refused");
    assert_eq!(refusal.code(), "child-count-bound");
    assert_eq!(
        refusal.to_string(),
        "protocol `p` revision 1: outcome `done` counts 65 children (`count: 65`), more than the \
         bound of 64 that canon simulate walks"
    );
    let walked = simulate(&compiled(&unguarded(64)))
        .expect("walked")
        .to_string();
    assert!(
        walked.ends_with("1 finding (2 stuck or cycle states)\n"),
        "{walked}"
    );
}
