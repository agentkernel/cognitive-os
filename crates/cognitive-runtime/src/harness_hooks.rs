//! Deterministic harness hooks — private MVP (P8-T04/D01–D02).
//!
//! Daemon-owned lifecycle interception points for admission, pre-dispatch,
//! post-effect, and verification. Owner-programmable hooks are digest-bound
//! observation programs invoked only on the management channel. They cannot
//! relax axioms, write authority, complete Tasks, or mint capabilities.
//! Graded Skill/rule loading remains a later slice.

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::channel_binding::AuthorityChannel;

/// Lifecycle interception points owned by the daemon harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessHookEvent {
    Admission,
    PreDispatch,
    PostEffect,
    Verification,
}

impl HarnessHookEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admission => "admission",
            Self::PreDispatch => "pre-dispatch",
            Self::PostEffect => "post-effect",
            Self::Verification => "verification",
        }
    }
}

/// Owner-supplied deterministic hook declaration (digest-bound program).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessHookDeclaration {
    pub hook_id: String,
    pub event: HarnessHookEvent,
    pub program_digest: String,
    pub may_relax_axioms: bool,
    pub may_write_authority: bool,
}

/// Registered daemon-owned hook fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredHarnessHook {
    pub hook_id: String,
    pub event: HarnessHookEvent,
    pub declaration_digest: String,
    pub program_digest: String,
}

/// Observation emitted when a registered hook is invoked (non-authoritative).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessHookObservation {
    pub hook_id: String,
    pub event: HarnessHookEvent,
    pub declaration_digest: String,
    pub program_digest: String,
    pub decision: &'static str,
}

/// Fail-closed harness hook errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HarnessHookError {
    #[error("harness hook is missing required identity or digest material")]
    MissingIdentity,
    #[error("harness hook claims permission to relax axioms")]
    AxiomRelaxationForbidden,
    #[error("harness hook claims authority-writer capability")]
    AuthorityWriterForbidden,
    #[error("harness hook declaration or program digest mismatch")]
    DigestMismatch,
    #[error("harness hook is not registered for the requested event")]
    HookNotRegistered,
    #[error("harness hook invocation requires the management channel")]
    ChannelIsolationViolation,
}

/// Validate and register a daemon-owned deterministic harness hook.
pub fn register_harness_hook(
    declaration: &HarnessHookDeclaration,
) -> Result<RegisteredHarnessHook, HarnessHookError> {
    if declaration.hook_id.trim().is_empty() || declaration.program_digest.trim().is_empty() {
        return Err(HarnessHookError::MissingIdentity);
    }
    if declaration.may_relax_axioms {
        return Err(HarnessHookError::AxiomRelaxationForbidden);
    }
    if declaration.may_write_authority {
        return Err(HarnessHookError::AuthorityWriterForbidden);
    }

    let declaration_digest = bind_declaration_digest(declaration);
    Ok(RegisteredHarnessHook {
        hook_id: declaration.hook_id.clone(),
        event: declaration.event,
        declaration_digest,
        program_digest: declaration.program_digest.clone(),
    })
}

/// Invoke a registered owner-programmable hook over exact digests (observation only).
pub fn invoke_registered_harness_hook(
    registered: &RegisteredHarnessHook,
    expected_declaration_digest: &str,
    expected_program_digest: &str,
    event: HarnessHookEvent,
    channel: AuthorityChannel,
) -> Result<HarnessHookObservation, HarnessHookError> {
    if channel != AuthorityChannel::Management {
        return Err(HarnessHookError::ChannelIsolationViolation);
    }
    if expected_declaration_digest.trim().is_empty()
        || expected_program_digest.trim().is_empty()
        || expected_declaration_digest != registered.declaration_digest
        || expected_program_digest != registered.program_digest
    {
        return Err(HarnessHookError::DigestMismatch);
    }
    if event != registered.event {
        return Err(HarnessHookError::HookNotRegistered);
    }
    Ok(HarnessHookObservation {
        hook_id: registered.hook_id.clone(),
        event: registered.event,
        declaration_digest: registered.declaration_digest.clone(),
        program_digest: registered.program_digest.clone(),
        decision: "observe",
    })
}

fn bind_declaration_digest(declaration: &HarnessHookDeclaration) -> String {
    let mut hasher = Sha256::new();
    hasher.update(declaration.hook_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(declaration.event.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(declaration.program_digest.as_bytes());
    hasher.update(b"\0");
    hasher.update([u8::from(declaration.may_relax_axioms)]);
    hasher.update(b"\0");
    hasher.update([u8::from(declaration.may_write_authority)]);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn valid_declaration(event: HarnessHookEvent) -> HarnessHookDeclaration {
        HarnessHookDeclaration {
            hook_id: format!("hook.{}.example", event.as_str()),
            event,
            program_digest: "aa".repeat(32),
            may_relax_axioms: false,
            may_write_authority: false,
        }
    }

    #[test]
    fn registers_and_invokes_lifecycle_hooks() {
        for event in [
            HarnessHookEvent::Admission,
            HarnessHookEvent::PreDispatch,
            HarnessHookEvent::PostEffect,
            HarnessHookEvent::Verification,
        ] {
            let declaration = valid_declaration(event);
            let registered = register_harness_hook(&declaration).expect("register");
            assert_eq!(registered.event, event);
            assert_eq!(registered.declaration_digest.len(), 64);
            let observation = invoke_registered_harness_hook(
                &registered,
                &registered.declaration_digest,
                &declaration.program_digest,
                event,
                AuthorityChannel::Management,
            )
            .expect("invoke");
            assert_eq!(observation.decision, "observe");
            assert_eq!(observation.event, event);
        }
    }

    #[test]
    fn rejects_axiom_relaxation_and_authority_writer() {
        let mut declaration = valid_declaration(HarnessHookEvent::Admission);
        declaration.may_relax_axioms = true;
        assert_eq!(
            register_harness_hook(&declaration).unwrap_err(),
            HarnessHookError::AxiomRelaxationForbidden
        );

        declaration = valid_declaration(HarnessHookEvent::PreDispatch);
        declaration.may_write_authority = true;
        assert_eq!(
            register_harness_hook(&declaration).unwrap_err(),
            HarnessHookError::AuthorityWriterForbidden
        );
    }

    #[test]
    fn rejects_stale_digest_and_event_mismatch() {
        let declaration = valid_declaration(HarnessHookEvent::Verification);
        let registered = register_harness_hook(&declaration).unwrap();
        assert_eq!(
            invoke_registered_harness_hook(
                &registered,
                "deadbeef",
                &declaration.program_digest,
                HarnessHookEvent::Verification,
                AuthorityChannel::Management,
            )
            .unwrap_err(),
            HarnessHookError::DigestMismatch
        );
        assert_eq!(
            invoke_registered_harness_hook(
                &registered,
                &registered.declaration_digest,
                "bb".repeat(32),
                HarnessHookEvent::Verification,
                AuthorityChannel::Management,
            )
            .unwrap_err(),
            HarnessHookError::DigestMismatch
        );
        assert_eq!(
            invoke_registered_harness_hook(
                &registered,
                &registered.declaration_digest,
                &declaration.program_digest,
                HarnessHookEvent::Admission,
                AuthorityChannel::Management,
            )
            .unwrap_err(),
            HarnessHookError::HookNotRegistered
        );
    }

    #[test]
    fn rejects_task_channel_hook_invocation() {
        let declaration = valid_declaration(HarnessHookEvent::PreDispatch);
        let registered = register_harness_hook(&declaration).unwrap();
        assert_eq!(
            invoke_registered_harness_hook(
                &registered,
                &registered.declaration_digest,
                &declaration.program_digest,
                HarnessHookEvent::PreDispatch,
                AuthorityChannel::Task,
            )
            .unwrap_err(),
            HarnessHookError::ChannelIsolationViolation
        );
    }
}
