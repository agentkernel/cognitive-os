//! Universal Agent Adapter Contract — private MVP (P8-T02/D01–D02).
//!
//! Daemon-owned adapter capability declaration, registration, and lifecycle
//! facts use AKP as the only adaptation protocol. Public listeners, direct
//! authority writes, Task-completion claims, and Task-channel lifecycle
//! mutations fail closed. Lane-CTR public `agent-adapter-manifest` schema
//! registration remains a later slice.

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::channel_binding::AuthorityChannel;

/// Supported adapter transport profile for the MVP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterTransportProfile {
    AkpHttpJsonSse,
}

impl AdapterTransportProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AkpHttpJsonSse => "akp-http-json-sse",
        }
    }
}

/// Declared adapter capability set. Candidate-only by default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCapabilityDeclaration {
    pub adapter_id: String,
    pub protocol: AdapterTransportProfile,
    pub candidate_only: bool,
    pub public_listener: bool,
    pub authority_writer: bool,
    pub discovery_card_digest: String,
}

/// Registered adapter fact bound to a capability declaration digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredAgentAdapter {
    pub adapter_id: String,
    pub declaration_digest: String,
    pub protocol: AdapterTransportProfile,
    pub candidate_only: bool,
}

/// Daemon-owned adapter lifecycle vocabulary for the private MVP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterLifecycleState {
    Registered,
    Active,
    Paused,
    Stopped,
}

/// Epoch-fenced lifecycle handle bound to a registered declaration digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterLifecycleHandle {
    pub adapter_id: String,
    pub declaration_digest: String,
    pub state: AdapterLifecycleState,
    pub fencing_epoch: u64,
}

/// Fail-closed adapter registration and lifecycle errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentAdapterError {
    #[error("adapter declaration is missing required identity or digest material")]
    MissingIdentity,
    #[error("adapter declaration enables a forbidden public listener")]
    PublicListenerForbidden,
    #[error("adapter declaration claims authority-writer capability")]
    AuthorityWriterForbidden,
    #[error("adapter declaration is not candidate-only")]
    CandidateOnlyRequired,
    #[error("adapter declaration protocol is unsupported")]
    UnsupportedProtocol,
    #[error("adapter declaration digest mismatch")]
    DigestMismatch,
    #[error("adapter lifecycle digest is stale relative to the registered declaration")]
    StaleDeclarationDigest,
    #[error("adapter lifecycle requires the management channel")]
    ChannelIsolationViolation,
    #[error("adapter lifecycle transition is invalid for the current state")]
    InvalidLifecycleTransition,
    #[error("adapter lifecycle fencing epoch mismatch")]
    StaleLifecycleEpoch,
}

/// Validate and register a private MVP agent adapter declaration.
pub fn register_agent_adapter(
    declaration: &AdapterCapabilityDeclaration,
) -> Result<RegisteredAgentAdapter, AgentAdapterError> {
    if declaration.adapter_id.trim().is_empty()
        || declaration.discovery_card_digest.trim().is_empty()
    {
        return Err(AgentAdapterError::MissingIdentity);
    }
    if declaration.public_listener {
        return Err(AgentAdapterError::PublicListenerForbidden);
    }
    if declaration.authority_writer {
        return Err(AgentAdapterError::AuthorityWriterForbidden);
    }
    if !declaration.candidate_only {
        return Err(AgentAdapterError::CandidateOnlyRequired);
    }
    if declaration.protocol != AdapterTransportProfile::AkpHttpJsonSse {
        return Err(AgentAdapterError::UnsupportedProtocol);
    }

    let declaration_digest = bind_declaration_digest(declaration);
    Ok(RegisteredAgentAdapter {
        adapter_id: declaration.adapter_id.clone(),
        declaration_digest,
        protocol: declaration.protocol,
        candidate_only: true,
    })
}

/// Re-validate a registered adapter against a caller-supplied declaration.
pub fn verify_registered_agent_adapter(
    registered: &RegisteredAgentAdapter,
    declaration: &AdapterCapabilityDeclaration,
) -> Result<(), AgentAdapterError> {
    let expected = register_agent_adapter(declaration)?;
    if expected.adapter_id != registered.adapter_id
        || expected.declaration_digest != registered.declaration_digest
        || expected.protocol != registered.protocol
        || expected.candidate_only != registered.candidate_only
    {
        return Err(AgentAdapterError::DigestMismatch);
    }
    Ok(())
}

/// Open a registered adapter into the inactive lifecycle handle (epoch 0).
pub fn open_registered_adapter_lifecycle(
    registered: &RegisteredAgentAdapter,
) -> Result<AdapterLifecycleHandle, AgentAdapterError> {
    if registered.adapter_id.trim().is_empty() || registered.declaration_digest.trim().is_empty() {
        return Err(AgentAdapterError::MissingIdentity);
    }
    if !registered.candidate_only {
        return Err(AgentAdapterError::CandidateOnlyRequired);
    }
    if registered.protocol != AdapterTransportProfile::AkpHttpJsonSse {
        return Err(AgentAdapterError::UnsupportedProtocol);
    }
    Ok(AdapterLifecycleHandle {
        adapter_id: registered.adapter_id.clone(),
        declaration_digest: registered.declaration_digest.clone(),
        state: AdapterLifecycleState::Registered,
        fencing_epoch: 0,
    })
}

/// Activate a registered/paused/stopped adapter over an exact declaration digest.
pub fn activate_adapter_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, AgentAdapterError> {
    require_management_channel(channel)?;
    require_current_digest(handle, expected_declaration_digest)?;
    require_current_epoch(handle, expected_epoch)?;
    match handle.state {
        AdapterLifecycleState::Registered
        | AdapterLifecycleState::Paused
        | AdapterLifecycleState::Stopped => Ok(AdapterLifecycleHandle {
            adapter_id: handle.adapter_id.clone(),
            declaration_digest: handle.declaration_digest.clone(),
            state: AdapterLifecycleState::Active,
            fencing_epoch: handle.fencing_epoch.saturating_add(1),
        }),
        AdapterLifecycleState::Active => Err(AgentAdapterError::InvalidLifecycleTransition),
    }
}

/// Pause an active adapter over an exact declaration digest.
pub fn pause_adapter_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, AgentAdapterError> {
    require_management_channel(channel)?;
    require_current_digest(handle, expected_declaration_digest)?;
    require_current_epoch(handle, expected_epoch)?;
    match handle.state {
        AdapterLifecycleState::Active => Ok(AdapterLifecycleHandle {
            adapter_id: handle.adapter_id.clone(),
            declaration_digest: handle.declaration_digest.clone(),
            state: AdapterLifecycleState::Paused,
            fencing_epoch: handle.fencing_epoch.saturating_add(1),
        }),
        AdapterLifecycleState::Registered
        | AdapterLifecycleState::Paused
        | AdapterLifecycleState::Stopped => Err(AgentAdapterError::InvalidLifecycleTransition),
    }
}

/// Stop an active or paused adapter over an exact declaration digest.
pub fn stop_adapter_lifecycle(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
    expected_epoch: u64,
    channel: AuthorityChannel,
) -> Result<AdapterLifecycleHandle, AgentAdapterError> {
    require_management_channel(channel)?;
    require_current_digest(handle, expected_declaration_digest)?;
    require_current_epoch(handle, expected_epoch)?;
    match handle.state {
        AdapterLifecycleState::Active | AdapterLifecycleState::Paused => {
            Ok(AdapterLifecycleHandle {
                adapter_id: handle.adapter_id.clone(),
                declaration_digest: handle.declaration_digest.clone(),
                state: AdapterLifecycleState::Stopped,
                fencing_epoch: handle.fencing_epoch.saturating_add(1),
            })
        }
        AdapterLifecycleState::Registered | AdapterLifecycleState::Stopped => {
            Err(AgentAdapterError::InvalidLifecycleTransition)
        }
    }
}

fn require_management_channel(channel: AuthorityChannel) -> Result<(), AgentAdapterError> {
    if channel != AuthorityChannel::Management {
        return Err(AgentAdapterError::ChannelIsolationViolation);
    }
    Ok(())
}

fn require_current_digest(
    handle: &AdapterLifecycleHandle,
    expected_declaration_digest: &str,
) -> Result<(), AgentAdapterError> {
    if expected_declaration_digest.trim().is_empty()
        || expected_declaration_digest != handle.declaration_digest
    {
        return Err(AgentAdapterError::StaleDeclarationDigest);
    }
    Ok(())
}

fn require_current_epoch(
    handle: &AdapterLifecycleHandle,
    expected_epoch: u64,
) -> Result<(), AgentAdapterError> {
    if expected_epoch != handle.fencing_epoch {
        return Err(AgentAdapterError::StaleLifecycleEpoch);
    }
    Ok(())
}

fn bind_declaration_digest(declaration: &AdapterCapabilityDeclaration) -> String {
    let mut hasher = Sha256::new();
    hasher.update(declaration.adapter_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(declaration.protocol.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update([u8::from(declaration.candidate_only)]);
    hasher.update(b"\0");
    hasher.update([u8::from(declaration.public_listener)]);
    hasher.update(b"\0");
    hasher.update([u8::from(declaration.authority_writer)]);
    hasher.update(b"\0");
    hasher.update(declaration.discovery_card_digest.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn valid_declaration() -> AdapterCapabilityDeclaration {
        AdapterCapabilityDeclaration {
            adapter_id: "adapter.example.cli".to_owned(),
            protocol: AdapterTransportProfile::AkpHttpJsonSse,
            candidate_only: true,
            public_listener: false,
            authority_writer: false,
            discovery_card_digest: "aa".repeat(32),
        }
    }

    fn registered_handle() -> (RegisteredAgentAdapter, AdapterLifecycleHandle) {
        let registered = register_agent_adapter(&valid_declaration()).expect("register");
        let handle =
            open_registered_adapter_lifecycle(&registered).expect("open registered lifecycle");
        (registered, handle)
    }

    #[test]
    fn registers_candidate_only_akp_adapter() {
        let registered = register_agent_adapter(&valid_declaration()).expect("register");
        assert_eq!(registered.adapter_id, "adapter.example.cli");
        assert!(registered.candidate_only);
        assert_eq!(registered.declaration_digest.len(), 64);
        verify_registered_agent_adapter(&registered, &valid_declaration()).unwrap();
    }

    #[test]
    fn rejects_public_listener_and_authority_writer() {
        let mut declaration = valid_declaration();
        declaration.public_listener = true;
        assert_eq!(
            register_agent_adapter(&declaration).unwrap_err(),
            AgentAdapterError::PublicListenerForbidden
        );

        declaration = valid_declaration();
        declaration.authority_writer = true;
        assert_eq!(
            register_agent_adapter(&declaration).unwrap_err(),
            AgentAdapterError::AuthorityWriterForbidden
        );
    }

    #[test]
    fn rejects_non_candidate_and_digest_tampering() {
        let mut declaration = valid_declaration();
        declaration.candidate_only = false;
        assert_eq!(
            register_agent_adapter(&declaration).unwrap_err(),
            AgentAdapterError::CandidateOnlyRequired
        );

        let registered = register_agent_adapter(&valid_declaration()).unwrap();
        let mut tampered = valid_declaration();
        tampered.discovery_card_digest = "bb".repeat(32);
        assert_eq!(
            verify_registered_agent_adapter(&registered, &tampered).unwrap_err(),
            AgentAdapterError::DigestMismatch
        );
    }

    #[test]
    fn activates_pauses_and_stops_over_declaration_digest() {
        let (registered, handle) = registered_handle();
        assert_eq!(handle.state, AdapterLifecycleState::Registered);
        assert_eq!(handle.fencing_epoch, 0);

        let active = activate_adapter_lifecycle(
            &handle,
            &registered.declaration_digest,
            0,
            AuthorityChannel::Management,
        )
        .expect("activate");
        assert_eq!(active.state, AdapterLifecycleState::Active);
        assert_eq!(active.fencing_epoch, 1);

        let paused = pause_adapter_lifecycle(
            &active,
            &registered.declaration_digest,
            1,
            AuthorityChannel::Management,
        )
        .expect("pause");
        assert_eq!(paused.state, AdapterLifecycleState::Paused);
        assert_eq!(paused.fencing_epoch, 2);

        let stopped = stop_adapter_lifecycle(
            &paused,
            &registered.declaration_digest,
            2,
            AuthorityChannel::Management,
        )
        .expect("stop");
        assert_eq!(stopped.state, AdapterLifecycleState::Stopped);
        assert_eq!(stopped.fencing_epoch, 3);
    }

    #[test]
    fn rejects_stale_digest_and_task_channel_lifecycle() {
        let (registered, handle) = registered_handle();

        assert_eq!(
            activate_adapter_lifecycle(&handle, "deadbeef", 0, AuthorityChannel::Management,)
                .unwrap_err(),
            AgentAdapterError::StaleDeclarationDigest
        );

        assert_eq!(
            activate_adapter_lifecycle(
                &handle,
                &registered.declaration_digest,
                0,
                AuthorityChannel::Task,
            )
            .unwrap_err(),
            AgentAdapterError::ChannelIsolationViolation
        );

        let active = activate_adapter_lifecycle(
            &handle,
            &registered.declaration_digest,
            0,
            AuthorityChannel::Management,
        )
        .expect("activate");
        assert_eq!(
            pause_adapter_lifecycle(
                &active,
                &registered.declaration_digest,
                0,
                AuthorityChannel::Management,
            )
            .unwrap_err(),
            AgentAdapterError::StaleLifecycleEpoch
        );
        assert_eq!(
            stop_adapter_lifecycle(
                &active,
                &registered.declaration_digest,
                1,
                AuthorityChannel::Task,
            )
            .unwrap_err(),
            AgentAdapterError::ChannelIsolationViolation
        );
    }
}
