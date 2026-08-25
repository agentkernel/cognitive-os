//! Authority channel-binding gate (REQ-SHELL-CHANNEL-001 / REQ-SHELL-UX-001).
//!
//! Task and management channels use disjoint credentials and session material.
//! A task-channel credential presented against a privileged management action
//! (vector `SHELL-CHANNEL-ISOLATION-003`) MUST deny with
//! `SHELL_CHANNEL_BINDING_MISMATCH` and MUST NOT leak management context.
//!
//! This gate is deterministic runtime authority — not a client-side guard.
//! TS SDK channel binding is a fail-closed mirror and does not substitute for
//! this surface.

use cognitive_contracts::generated::error_registry::RegisteredErrorCode;

/// Client channel brand carried by a credential (authority-visible).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityChannel {
    Task,
    Management,
}

/// Input shape aligned with `shell-channel-isolation-003.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelBindingRequest {
    /// Channel brand of the presented credential.
    pub credential_channel: AuthorityChannel,
    /// Requested operation name (e.g. `system.configure`).
    pub requested_action: String,
    /// Whether a valid PrivilegedManagementSession is bound.
    pub privileged_session: bool,
}

/// Observable admission outcome for channel binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelBindingDecision {
    pub decision: &'static str,
    pub error_code: Option<&'static str>,
    pub error_category: Option<&'static str>,
    /// Always false on deny: refusal bodies must not expose management context.
    pub management_context_leaked: bool,
}

/// Returns true when `action` requires the management channel.
pub fn is_privileged_management_action(action: &str) -> bool {
    matches!(
        action,
        "system.configure"
            | "gateway.configure"
            | "diagnostics.configure"
            | "capability.revoke"
            | "execution.stop"
            | "effect.reconcile"
            | "session.create_restricted"
            | "status.inspect"
    )
}

/// Admit or deny a request based on credential channel vs required channel.
///
/// Channel mismatch is evaluated before session/step-up checks so that a
/// task credential never reaches management session material
/// (`management_context_leaked` stays false).
pub fn admit_channel_binding(request: &ChannelBindingRequest) -> ChannelBindingDecision {
    let requires_management = is_privileged_management_action(&request.requested_action);

    if requires_management && request.credential_channel == AuthorityChannel::Task {
        return ChannelBindingDecision {
            decision: "deny",
            error_code: Some(RegisteredErrorCode::ShellChannelBindingMismatch.as_str()),
            error_category: Some("auth"),
            management_context_leaked: false,
        };
    }

    if !requires_management && request.credential_channel == AuthorityChannel::Management {
        // Management credential on a task-only verb is also a binding mismatch.
        return ChannelBindingDecision {
            decision: "deny",
            error_code: Some(RegisteredErrorCode::ShellChannelBindingMismatch.as_str()),
            error_category: Some("auth"),
            management_context_leaked: false,
        };
    }

    if requires_management && !request.privileged_session {
        // Channel matched; session gate is out of this vector's expected shape.
        // Fail closed without leaking management context.
        return ChannelBindingDecision {
            decision: "deny",
            error_code: Some(RegisteredErrorCode::ManagementSessionExpired.as_str()),
            error_category: Some("auth"),
            management_context_leaked: false,
        };
    }

    ChannelBindingDecision {
        decision: "allow",
        error_code: None,
        error_category: None,
        management_context_leaked: false,
    }
}

/// Convenience: vector-shaped constructor for the SHELL-CHANNEL-ISOLATION-003 input.
pub fn request_from_vector_input(
    task_conversation_credential: bool,
    requested_action: impl Into<String>,
    privileged_session: bool,
) -> ChannelBindingRequest {
    ChannelBindingRequest {
        credential_channel: if task_conversation_credential {
            AuthorityChannel::Task
        } else {
            AuthorityChannel::Management
        },
        requested_action: requested_action.into(),
        privileged_session,
    }
}
