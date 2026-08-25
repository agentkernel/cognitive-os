//! TargetSelector resolution gate (REQ-SHELL-TARGET-001 / REQ-SHELL-AMBIGUITY-001).
//!
//! Natural-language selectors, aliases, and query forms only produce candidates.
//! Stateful shell dispatch MUST resolve to a unique, authorized, fixed-version
//! strong reference. Multiple visible candidates (vector
//! `SHELL-TARGET-AMBIGUITY-001`: selector `"stop it"` + two `execution://`
//! candidates) MUST return `SHELL_TARGET_AMBIGUOUS` with
//! `decision=clarification_required` and `dispatch=false` — never guess top-1.
//!
//! This gate is deterministic runtime authority. Probabilistic components MUST
//! NOT participate.

use cognitive_contracts::generated::error_registry::RegisteredErrorCode;

/// Input shape aligned with `shell-target-ambiguity-001.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSelectorRequest {
    /// Raw TargetSelector text (natural language, alias, or strong ref).
    pub selector: String,
    /// Discoverable / visible candidate strong refs in the current scope.
    pub visible_candidates: Vec<String>,
}

/// Observable admission outcome for target resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSelectorDecision {
    pub decision: &'static str,
    pub error_code: Option<&'static str>,
    pub error_category: Option<&'static str>,
    /// Always false when clarification or not-found: no effect dispatch.
    pub dispatch: bool,
    /// Resolved unique strong ref when admitted; None on deny/clarify.
    pub resolved_target: Option<String>,
}

/// True when `selector` is already a strong object URI (not NL / alias alone).
pub fn is_strong_reference(selector: &str) -> bool {
    let trimmed = selector.trim();
    if trimmed.is_empty() || trimmed.contains(' ') {
        return false;
    }
    matches!(
        trimmed.split_once("://"),
        Some((scheme, rest))
            if !scheme.is_empty()
                && !rest.is_empty()
                && rest.chars().all(|c| c.is_ascii_alphanumeric()
                    || matches!(c, '-' | '_' | '.' | '/' | '%' | ':' | '@'))
    )
}

/// Admit or deny dispatch based on TargetSelector vs visible candidates.
///
/// Multi-candidate or non-unique NL resolution fails closed with
/// `SHELL_TARGET_AMBIGUOUS`. Zero authorized matches → `SHELL_TARGET_NOT_FOUND`.
/// A single exact strong-ref match may admit with `dispatch=true`.
pub fn admit_target_selector(request: &TargetSelectorRequest) -> TargetSelectorDecision {
    let selector = request.selector.trim();
    let candidates = &request.visible_candidates;

    if candidates.len() > 1 {
        // Multiple visible objects: never guess which "it" / top-1 means.
        return TargetSelectorDecision {
            decision: "clarification_required",
            error_code: Some(RegisteredErrorCode::ShellTargetAmbiguous.as_str()),
            error_category: Some("shell"),
            dispatch: false,
            resolved_target: None,
        };
    }

    if candidates.is_empty() {
        return TargetSelectorDecision {
            decision: "deny",
            error_code: Some(RegisteredErrorCode::ShellTargetNotFound.as_str()),
            error_category: Some("shell"),
            dispatch: false,
            resolved_target: None,
        };
    }

    // Exactly one visible candidate.
    let only = &candidates[0];
    if is_strong_reference(selector) {
        if selector == only.as_str() {
            return TargetSelectorDecision {
                decision: "allow",
                error_code: None,
                error_category: None,
                dispatch: true,
                resolved_target: Some(only.clone()),
            };
        }
        return TargetSelectorDecision {
            decision: "deny",
            error_code: Some(RegisteredErrorCode::ShellTargetNotFound.as_str()),
            error_category: Some("shell"),
            dispatch: false,
            resolved_target: None,
        };
    }

    // Natural language / alias with a single visible candidate still requires
    // clarification: guessing that the sole listing is "it" expands scope risk
    // and is forbidden by REQ-SHELL-TARGET-001 (no top-1 / "it" guess).
    TargetSelectorDecision {
        decision: "clarification_required",
        error_code: Some(RegisteredErrorCode::ShellTargetAmbiguous.as_str()),
        error_category: Some("shell"),
        dispatch: false,
        resolved_target: None,
    }
}

/// Convenience: vector-shaped constructor for SHELL-TARGET-AMBIGUITY-001 input.
pub fn request_from_target_vector_input(
    selector: impl Into<String>,
    visible_candidates: impl IntoIterator<Item = impl Into<String>>,
) -> TargetSelectorRequest {
    TargetSelectorRequest {
        selector: selector.into(),
        visible_candidates: visible_candidates.into_iter().map(Into::into).collect(),
    }
}
