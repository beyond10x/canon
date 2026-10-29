//! The static check over a set of compiled protocols that their case openings form no cycle
//! (story:case-composition, D206).
//!
//! An action's `case_open` names the protocol a child case is governed by, never an existing case,
//! so a case cannot be checked against its ancestors at run time; a protocol whose openings can
//! reach it again is refused statically instead. `canon validate` refuses an opening under the
//! protocol's own id, one protocol at a time. [`check_openings`] takes a set of protocols and
//! refuses it when their openings form a cycle: protocol `a` opens a case under `b` when an action
//! of `a` declares `case_open` with protocol `b`, and a cycle is a path of such openings that
//! comes back to where it began. Pure: no file, no clock, no run-time state.
//!
//! The cycle named is the first one found from the least protocol id on any cycle: a depth-first
//! walk from that protocol, through protocols with greater ids only, each opening taken in
//! protocol-id order, a protocol walked through once, closing as soon as a protocol on the walk
//! opens a case under the first. It is written in the order its openings go, from its least id
//! back to it (`a -> c -> b -> a`), a protocol that opens a case under itself as `a -> a`. An
//! opening under a protocol outside the set is an edge out of the set: it closes no cycle. Two
//! protocols of the set with one id are one protocol, with the openings of both.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::Ir;
use crate::model::{ProtocolId, one_line};

/// A cycle of case openings among a set of protocols: [`check_openings`] refuses the set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningCycle {
    /// The protocols on the cycle in the order their openings go, from the least id back to it:
    /// the first and the last entry are that id.
    pub protocols: Vec<ProtocolId>,
}

impl OpeningCycle {
    /// The stable identifier of the refusal.
    pub fn code(&self) -> &'static str {
        "case-open-cycle"
    }
}

impl fmt::Display for OpeningCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path: Vec<String> = self
            .protocols
            .iter()
            .map(|protocol| one_line(protocol.as_str()))
            .collect();
        write!(
            f,
            "protocols open cases under each other in a cycle: {}",
            path.join(" -> ")
        )
    }
}

/// Refuses `protocols` when their case openings form a cycle, naming it as the module docs say;
/// accepts a set without one, a chain of openings included.
pub fn check_openings<'a>(protocols: impl IntoIterator<Item = &'a Ir>) -> Result<(), OpeningCycle> {
    let mut opens: BTreeMap<&ProtocolId, BTreeSet<&ProtocolId>> = BTreeMap::new();
    for ir in protocols {
        let opened = opens.entry(&ir.protocol.id).or_default();
        for action in ir.actions.values() {
            if let Some(opening) = &action.case_open {
                opened.insert(&opening.protocol);
            }
        }
    }
    for start in opens.keys() {
        if let Some(path) = cycle_from(start, &opens) {
            return Err(OpeningCycle {
                protocols: path.into_iter().cloned().collect(),
            });
        }
    }
    Ok(())
}

/// The first cycle a depth-first walk from `start` finds through protocols with greater ids,
/// `start` repeated at its end; `None` when no walk from `start` comes back to it.
fn cycle_from<'a>(
    start: &'a ProtocolId,
    opens: &BTreeMap<&'a ProtocolId, BTreeSet<&'a ProtocolId>>,
) -> Option<Vec<&'a ProtocolId>> {
    let successors = |protocol: &ProtocolId| opens.get(protocol).into_iter().flatten().copied();
    let mut walked: BTreeSet<&ProtocolId> = BTreeSet::from([start]);
    // The walk: each protocol on it, with the openings of it not yet taken.
    let mut path: Vec<(&ProtocolId, Vec<&ProtocolId>)> = Vec::new();
    let further = |protocol: &ProtocolId| -> Vec<&'a ProtocolId> {
        let mut next: Vec<&ProtocolId> =
            successors(protocol).filter(|next| *next > start).collect();
        next.reverse();
        next
    };
    path.push((start, further(start)));
    while let Some((protocol, _)) = path.last() {
        if successors(protocol).any(|next| next == start) {
            let mut cycle: Vec<&ProtocolId> = path.iter().map(|(protocol, _)| *protocol).collect();
            cycle.push(start);
            return Some(cycle);
        }
        let next = path.last_mut().and_then(|(_, next)| {
            while let Some(candidate) = next.pop() {
                if !walked.contains(candidate) {
                    return Some(candidate);
                }
            }
            None
        });
        match next {
            Some(next) => {
                walked.insert(next);
                path.push((next, further(next)));
            }
            None => {
                path.pop();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::compile;
    use crate::model::parse;

    /// A protocol `id` whose actions open a case under each protocol of `opens`.
    fn protocol(id: &str, opens: &[&str]) -> Ir {
        let actions: String = opens
            .iter()
            .enumerate()
            .map(|(n, opened)| {
                format!("  open{n}: {{case_open: {{protocol: '{opened}', subject: thing}}}}\n")
            })
            .collect();
        let actions = if actions.is_empty() {
            String::new()
        } else {
            format!("actions:\n{actions}")
        };
        let text = format!(
            "format: protocol/1\nprotocol: {{id: '{id}', revision: 1}}\nartifacts: {{thing: {{}}}}\n{actions}"
        );
        compile(&parse(&text).expect("parses")).expect("compiles")
    }

    fn cycle(protocols: &[Ir]) -> Option<String> {
        check_openings(protocols).err().map(|cycle| {
            assert_eq!(cycle.code(), "case-open-cycle");
            cycle.to_string()
        })
    }

    /// D206: two protocols that open each other are refused, the cycle named from its least id;
    /// a chain without a cycle is accepted, and so is an empty set.
    #[test]
    fn two_protocols_that_open_each_other_are_refused_and_a_chain_is_accepted() {
        assert_eq!(
            cycle(&[
                protocol("review", &["audit"]),
                protocol("audit", &["review"])
            ]),
            Some(
                "protocols open cases under each other in a cycle: audit -> review -> audit".into()
            )
        );
        assert_eq!(
            cycle(&[
                protocol("audit", &["software.change"]),
                protocol("software.change", &["review"]),
                protocol("review", &[]),
            ]),
            None
        );
        assert_eq!(cycle(&[]), None);
    }

    /// The cycle goes the way its openings go, from its least id, whatever order the set is given
    /// in; ids not on the cycle and openings out of the set take no part.
    #[test]
    fn the_cycle_is_written_in_opening_order_from_its_least_id() {
        let set = [
            protocol("c", &["b", "outside"]),
            protocol("b", &["d"]),
            protocol("a", &["c"]),
            protocol("d", &["a"]),
            protocol("e", &["a"]),
        ];
        let expected = "protocols open cases under each other in a cycle: a -> c -> b -> d -> a";
        assert_eq!(cycle(&set).as_deref(), Some(expected));
        let mut reversed = set.clone();
        reversed.reverse();
        assert_eq!(cycle(&reversed).as_deref(), Some(expected));
    }

    /// Of several cycles, the one through the least id on any cycle is named, and of those
    /// through it, the first a walk in id order finds; a protocol opening itself, which `canon
    /// validate` refuses on its own, is the cycle `p -> p` when a caller builds one.
    #[test]
    fn the_least_cycle_is_named_and_a_self_opening_is_a_cycle_of_one() {
        let set = [
            protocol("m", &["n"]),
            protocol("n", &["m"]),
            protocol("b", &["z", "c"]),
            protocol("c", &["b"]),
            protocol("z", &["b"]),
        ];
        assert_eq!(
            cycle(&set).as_deref(),
            Some("protocols open cases under each other in a cycle: b -> c -> b")
        );
        let mut own = protocol("p", &[]);
        let mut opening = protocol("q", &["p"]);
        let (id, action) = opening.actions.pop_first().expect("an action");
        own.actions.insert(id, action);
        assert_eq!(
            cycle(&[own]).as_deref(),
            Some("protocols open cases under each other in a cycle: p -> p")
        );
    }

    /// Two protocols of the set with one id are one protocol, with the openings of both.
    #[test]
    fn protocols_with_one_id_pool_their_openings() {
        assert_eq!(
            cycle(&[
                protocol("a", &["b"]),
                protocol("a", &[]),
                protocol("b", &["a"]),
            ])
            .as_deref(),
            Some("protocols open cases under each other in a cycle: a -> b -> a")
        );
    }

    /// A walk does not pass through a protocol twice, and one that came back to a protocol other
    /// than the first is not a cycle through the first: a long chain of openings with an inner
    /// loop names the inner loop, from its own least id.
    #[test]
    fn an_inner_loop_is_named_from_its_own_least_id() {
        let set = [
            protocol("a", &["b"]),
            protocol("b", &["c"]),
            protocol("c", &["d"]),
            protocol("d", &["c"]),
        ];
        assert_eq!(
            cycle(&set).as_deref(),
            Some("protocols open cases under each other in a cycle: c -> d -> c")
        );
    }
}
