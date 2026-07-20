//! PrivilegedManagementSession: the mandatory gate in front of every
//! management verb.
//!
//! Type shape source: `specs/schemas/privileged-management-session.schema.json`
//! (registered machine contract; REQ-MGMT-SESSION-001). The schema has no
//! generated binding yet (Lane-CTR CORE_SET follow-up), so this module
//! hand-builds the type against the schema shape and validates the same
//! constraints deterministically. Gate semantics: RFC-0001 §7.5,
//! `docs/standards/authn-authz-capability.md` §5, vectors
//! `management-session-denials.json` / `management-gate-denials.json`.
//!
//! Batch-1 verification depth: document SHAPE validation plus the
//! deterministic state/expiry/scope/risk/step-up gate. Cryptographic
//! verification of `authority_signature` and the session-issuing lifecycle
//! (renewal, idle-timeout bookkeeping) belong to the Management API batch.

use crate::error::ManagementDenial;
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_domain::WallTimestamp;
use serde_json::Value;

/// Session lifecycle state (schema `state` enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Created, not yet activated.
    Pending,
    /// Active and usable within scope.
    Active,
    /// Idle or absolute expiry passed.
    Expired,
    /// Immediately revoked.
    Revoked,
    /// Closed by its principal or authority.
    Closed,
}

impl SessionState {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "active" => Some(Self::Active),
            "expired" => Some(Self::Expired),
            "revoked" => Some(Self::Revoked),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }
}

/// Risk class ordering R0 < R1 < R2 < R3 (schema `risk_ceiling` enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskClass {
    /// Reversible, low blast radius.
    R0,
    /// Standard governed action.
    R1,
    /// High risk: trusted surface required.
    R2,
    /// Critical: dual independent approval required.
    R3,
}

impl RiskClass {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "R0" => Some(Self::R0),
            "R1" => Some(Self::R1),
            "R2" => Some(Self::R2),
            "R3" => Some(Self::R3),
            _ => None,
        }
    }
}

/// Session scope (schema `scope` object): management domains, action names
/// and resource prefixes this session may reach at most.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionScope {
    /// Management domains (`cognitiveos.management[.sub]*`).
    pub domains: Vec<String>,
    /// Action names.
    pub actions: Vec<String>,
    /// Resource URI prefixes.
    pub resources: Vec<String>,
}

/// One privileged management session (schema shape; see module docs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivilegedManagementSession {
    /// `pms_`-prefixed session identity.
    pub session_id: String,
    /// Object version.
    pub object_version: i64,
    /// Management domain the session was issued under.
    pub management_domain: String,
    /// Issuing session authority reference.
    pub session_authority: String,
    /// Authenticated human principal reference.
    pub human_principal: String,
    /// Actor chain digest.
    pub actor_chain_digest: String,
    /// Authentication context reference.
    pub authentication_context_ref: String,
    /// Activity context reference.
    pub activity_context_ref: String,
    /// Scope upper bound.
    pub scope: SessionScope,
    /// Risk ceiling (upper bound, not an approval).
    pub risk_ceiling: RiskClass,
    /// Policy version the session binds.
    pub policy_version: i64,
    /// Revocation epoch the session binds.
    pub revocation_epoch: i64,
    /// Issue instant.
    pub issued_at: WallTimestamp,
    /// Last activity instant.
    pub last_activity_at: WallTimestamp,
    /// Idle timeout (seconds, 30..=3600).
    pub idle_timeout_seconds: i64,
    /// Absolute expiry instant.
    pub absolute_expires_at: WallTimestamp,
    /// Lifecycle state.
    pub state: SessionState,
    /// Declared step-up methods (may be empty).
    pub step_up_methods: Vec<String>,
    /// Session content digest.
    pub session_digest: String,
    /// Authority signature (shape-checked in this batch; see module docs).
    pub authority_signature: String,
}

/// One management action presented at the gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagementAction {
    /// Action name (must lie inside `scope.actions`).
    pub action: String,
    /// Management domain (must lie inside `scope.domains`).
    pub domain: String,
    /// Target resource (must be covered by `scope.resources`).
    pub resource: String,
    /// Risk class of the action (must not exceed `risk_ceiling`).
    pub risk: RiskClass,
    /// Whether policy requires a step-up for this action.
    pub step_up_required: bool,
    /// Whether a policy-approved step-up has been satisfied.
    pub step_up_satisfied: bool,
}

const SCHEMA_VERSION: &str = "cognitiveos.privileged-management-session/0.1";

fn denial(detail: String) -> ManagementDenial {
    // A document that fails the registered session shape is not a session:
    // the generic registered auth denial applies (fail closed), matching
    // the M3 protected-read denial code.
    ManagementDenial::new(
        RegisteredErrorCode::ContextAuthDenied,
        format!("management session rejected: {detail}"),
    )
}

fn field<'v>(value: &'v Value, name: &str) -> Result<&'v Value, ManagementDenial> {
    value
        .get(name)
        .ok_or_else(|| denial(format!("missing required member `{name}`")))
}

fn string_field(value: &Value, name: &str) -> Result<String, ManagementDenial> {
    field(value, name)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| denial(format!("member `{name}` must be a string")))
}

fn integer_field(value: &Value, name: &str) -> Result<i64, ManagementDenial> {
    field(value, name)?
        .as_i64()
        .ok_or_else(|| denial(format!("member `{name}` must be an integer")))
}

fn timestamp_field(value: &Value, name: &str) -> Result<WallTimestamp, ManagementDenial> {
    let text = string_field(value, name)?;
    WallTimestamp::parse(&text).map_err(|err| {
        denial(format!(
            "member `{name}` is not a canonical timestamp: {err}"
        ))
    })
}

fn string_array(value: &Value, name: &str) -> Result<Vec<String>, ManagementDenial> {
    let items = field(value, name)?
        .as_array()
        .ok_or_else(|| denial(format!("member `{name}` must be an array")))?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| denial(format!("member `{name}` must contain strings")))?,
        );
    }
    if out.is_empty() {
        return Err(denial(format!("member `{name}` must not be empty")));
    }
    Ok(out)
}

/// `^pms_[A-Za-z0-9._-]{8,128}$` (schema `session_id`).
fn valid_session_id(value: &str) -> bool {
    value.strip_prefix("pms_").is_some_and(|rest| {
        (8..=128).contains(&rest.len())
            && rest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
    })
}

/// `^cognitiveos\.management(?:\.[a-z0-9_-]+)*$` (schema domain pattern).
fn valid_management_domain(value: &str) -> bool {
    if value == "cognitiveos.management" {
        return true;
    }
    value
        .strip_prefix("cognitiveos.management.")
        .is_some_and(|rest| {
            !rest.is_empty()
                && rest.split('.').all(|segment| {
                    !segment.is_empty()
                        && segment.bytes().all(|b| {
                            b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-'
                        })
                })
        })
}

/// `^sha256:[0-9a-f]{64}$` (common-defs digest).
fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

impl PrivilegedManagementSession {
    /// Parse and shape-validate a session document against the registered
    /// schema constraints. Any violation fails closed with the registered
    /// auth denial — an invalid document is not a session.
    pub fn from_json_value(value: &Value) -> Result<Self, ManagementDenial> {
        let schema_version = string_field(value, "schema_version")?;
        if schema_version != SCHEMA_VERSION {
            return Err(denial(format!(
                "schema_version `{schema_version}` != `{SCHEMA_VERSION}`"
            )));
        }
        let session_id = string_field(value, "session_id")?;
        if !valid_session_id(&session_id) {
            return Err(denial(
                "session_id violates the registered pattern".to_owned(),
            ));
        }
        let object_version = integer_field(value, "object_version")?;
        if object_version < 1 {
            return Err(denial("object_version must be >= 1".to_owned()));
        }
        let management_domain = string_field(value, "management_domain")?;
        if !valid_management_domain(&management_domain) {
            return Err(denial(
                "management_domain violates the registered pattern".to_owned(),
            ));
        }
        let session_authority = nonempty(value, "session_authority")?;
        let human_principal = nonempty(value, "human_principal")?;
        let actor_chain_digest = string_field(value, "actor_chain_digest")?;
        if !valid_digest(&actor_chain_digest) {
            return Err(denial(
                "actor_chain_digest is not a sha256 digest".to_owned(),
            ));
        }
        let authentication_context_ref = nonempty(value, "authentication_context_ref")?;
        let activity_context_ref = nonempty(value, "activity_context_ref")?;

        let scope_value = field(value, "scope")?;
        let domains = string_array(scope_value, "domains")?;
        for domain in &domains {
            if !valid_management_domain(domain) {
                return Err(denial(format!(
                    "scope domain `{domain}` violates the registered pattern"
                )));
            }
        }
        let actions = string_array(scope_value, "actions")?;
        for action in &actions {
            if !valid_action_name(action) {
                return Err(denial(format!(
                    "scope action `{action}` violates the registered pattern"
                )));
            }
        }
        let resources = string_array(scope_value, "resources")?;

        let risk_ceiling = RiskClass::parse(&string_field(value, "risk_ceiling")?)
            .ok_or_else(|| denial("risk_ceiling must be one of R0..R3".to_owned()))?;
        let policy_version = integer_field(value, "policy_version")?;
        if policy_version < 1 {
            return Err(denial("policy_version must be >= 1".to_owned()));
        }
        let revocation_epoch = integer_field(value, "revocation_epoch")?;
        if revocation_epoch < 0 {
            return Err(denial("revocation_epoch must be >= 0".to_owned()));
        }
        let issued_at = timestamp_field(value, "issued_at")?;
        let last_activity_at = timestamp_field(value, "last_activity_at")?;
        let idle_timeout_seconds = integer_field(value, "idle_timeout_seconds")?;
        if !(30..=3600).contains(&idle_timeout_seconds) {
            return Err(denial(
                "idle_timeout_seconds must lie in 30..=3600".to_owned(),
            ));
        }
        let absolute_expires_at = timestamp_field(value, "absolute_expires_at")?;
        let state = SessionState::parse(&string_field(value, "state")?)
            .ok_or_else(|| denial("state is not a registered session state".to_owned()))?;
        let step_up_methods = match value.get("step_up_methods") {
            None => Vec::new(),
            Some(_) => string_array(value, "step_up_methods")?,
        };
        let session_digest = string_field(value, "session_digest")?;
        if !valid_digest(&session_digest) {
            return Err(denial("session_digest is not a sha256 digest".to_owned()));
        }
        let authority_signature = string_field(value, "authority_signature")?;
        if authority_signature.len() < 16 {
            return Err(denial("authority_signature is too short".to_owned()));
        }

        Ok(Self {
            session_id,
            object_version,
            management_domain,
            session_authority,
            human_principal,
            actor_chain_digest,
            authentication_context_ref,
            activity_context_ref,
            scope: SessionScope {
                domains,
                actions,
                resources,
            },
            risk_ceiling,
            policy_version,
            revocation_epoch,
            issued_at,
            last_activity_at,
            idle_timeout_seconds,
            absolute_expires_at,
            state,
            step_up_methods,
            session_digest,
            authority_signature,
        })
    }

    /// The deterministic management gate: decide whether this session may
    /// authorize `action` at instant `now`. Fail-closed order — lifecycle
    /// state, absolute expiry, domain, action, resource, risk ceiling,
    /// step-up. A denial dispatches nothing and touches no state.
    pub fn authorize(
        &self,
        action: &ManagementAction,
        now: &WallTimestamp,
    ) -> Result<(), ManagementDenial> {
        match self.state {
            SessionState::Active => {}
            SessionState::Expired => {
                return Err(ManagementDenial::new(
                    RegisteredErrorCode::ManagementSessionExpired,
                    "session state is expired; a newly authenticated session is required",
                ));
            }
            SessionState::Revoked => {
                return Err(ManagementDenial::new(
                    RegisteredErrorCode::ManagementSessionRevoked,
                    "session was revoked and cannot authorize new management work",
                ));
            }
            SessionState::Closed => {
                return Err(ManagementDenial::new(
                    RegisteredErrorCode::ManagementSessionExpired,
                    "session is closed; a newly authenticated session is required",
                ));
            }
            SessionState::Pending => {
                return Err(ManagementDenial::new(
                    RegisteredErrorCode::ManagementStepUpRequired,
                    "session is pending activation; complete authentication step-up first",
                ));
            }
        }
        // Absolute expiry derived from the instant, independent of the
        // recorded state (fail closed; instant comparison per ADR-0005).
        if now.instant_key() >= self.absolute_expires_at.instant_key() {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementSessionExpired,
                "session absolute expiry has passed",
            ));
        }
        if !self.scope.domains.iter().any(|d| d == &action.domain) {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementScopeMismatch,
                format!(
                    "management domain `{}` lies outside the session scope",
                    action.domain
                ),
            ));
        }
        if !self.scope.actions.iter().any(|a| a == &action.action) {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementScopeMismatch,
                format!("action `{}` lies outside the session scope", action.action),
            ));
        }
        let resource_covered = self
            .scope
            .resources
            .iter()
            .any(|prefix| action.resource == *prefix || action.resource.starts_with(prefix));
        if !resource_covered {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementScopeMismatch,
                format!(
                    "resource `{}` lies outside the session scope",
                    action.resource
                ),
            ));
        }
        // The ceiling is part of the scope upper bound: an action riskier
        // than the ceiling lies outside what this session can ever reach.
        if action.risk > self.risk_ceiling {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementScopeMismatch,
                "action risk class exceeds the session risk ceiling",
            ));
        }
        if action.step_up_required && !action.step_up_satisfied {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementStepUpRequired,
                "action requires a policy-approved step-up reauthentication",
            ));
        }
        Ok(())
    }
}

/// `^[a-z][a-z0-9_.:-]{2,127}$` (schema scope action pattern).
fn valid_action_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes.split_first() {
        Some((head, tail)) => {
            head.is_ascii_lowercase()
                && (2..=127).contains(&tail.len())
                && tail.iter().all(|b| {
                    b.is_ascii_lowercase()
                        || b.is_ascii_digit()
                        || *b == b'_'
                        || *b == b'.'
                        || *b == b':'
                        || *b == b'-'
                })
        }
        None => false,
    }
}

fn nonempty(value: &Value, name: &str) -> Result<String, ManagementDenial> {
    let text = string_field(value, name)?;
    if text.is_empty() {
        return Err(denial(format!("member `{name}` must not be empty")));
    }
    Ok(text)
}
