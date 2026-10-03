#![forbid(unsafe_code)]

//! Bootstrap types for Canon.
//!
//! This is intentionally tiny. The build agents should replace these scaffolds
//! with the real protocol model, compiler, evaluator, and conformance suite.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtocolId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClaimId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimValue {
    pub id: ClaimId,
    pub value: Truth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionStatus {
    Admissible,
    ApprovalRequired { capability: String },
    Blocked { reasons: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionCandidate {
    pub id: ActionId,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontier {
    pub case: CaseId,
    pub revision: u64,
    pub claims: Vec<ClaimValue>,
    pub obligations: Vec<String>,
    pub actions: Vec<ActionCandidate>,
    pub blocked_conclusions: Vec<String>,
}

impl Frontier {
    pub fn contains_action(&self, action: &ActionId) -> bool {
        self.actions.iter().any(|candidate| {
            candidate.id == *action
                && matches!(
                    &candidate.status,
                    ActionStatus::Admissible | ActionStatus::ApprovalRequired { .. }
                )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_does_not_admit_blocked_action() {
        let action = ActionId("merge".into());
        let frontier = Frontier {
            case: CaseId("CHG-1".into()),
            revision: 1,
            claims: vec![],
            obligations: vec![],
            actions: vec![ActionCandidate {
                id: action.clone(),
                status: ActionStatus::Blocked {
                    reasons: vec!["tests.pass is UNKNOWN".into()],
                },
            }],
            blocked_conclusions: vec![],
        };

        assert!(!frontier.contains_action(&action));
    }
}
