//! Deterministic R1 structured approval gate (F-011 / IMP-05).
use crate::{ManagementDenial, RiskClass};
use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_contracts::generated::management_approval_decision::{
    ManagementApprovalDecision, ManagementApprovalDecisionDecision,
    ManagementApprovalDecisionRiskClass, ManagementApprovalDecisionSchemaVersion,
};
use cognitive_contracts::generated::management_approval_request::{
    ManagementApprovalRequest, ManagementApprovalRequestConfirmationSurface,
    ManagementApprovalRequestRiskClass, ManagementApprovalRequestSchemaVersion,
};
use cognitive_domain::WallTimestamp;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
const DOMAIN: &str = "management-approval/0.1";

#[derive(Debug, Clone)]
pub struct ManagementActionProposal {
    pub proposal_id: String,
    pub session_ref: String,
    pub action: String,
    pub target_refs: Vec<String>,
    pub parameters: Value,
    pub risk_class: RiskClass,
    pub proposer_principal: String,
    pub proposer_actor_chain_digest: String,
    pub created_at: WallTimestamp,
    pub expires_at: WallTimestamp,
    pub proposal_digest: String,
}
impl ManagementActionProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        session: &str,
        action: &str,
        targets: Vec<String>,
        parameters: Value,
        risk: RiskClass,
        proposer: &str,
        chain: String,
        created: &WallTimestamp,
        expires: &WallTimestamp,
    ) -> Result<Self, ManagementDenial> {
        let value = serde_json::json!({"proposal_id":id,"session_ref":session,"action":action,"target_refs":targets,"parameters":parameters,"risk_class":format!("{risk:?}"),"proposer_principal":proposer,"proposer_actor_chain_digest":chain,"created_at":created.as_str(),"expires_at":expires.as_str()});
        let proposal_digest = digest(&value)?;
        Ok(Self {
            proposal_id: id.to_owned(),
            session_ref: session.to_owned(),
            action: action.to_owned(),
            target_refs: targets,
            parameters,
            risk_class: risk,
            proposer_principal: proposer.to_owned(),
            proposer_actor_chain_digest: chain,
            created_at: created.clone(),
            expires_at: expires.clone(),
            proposal_digest,
        })
    }
}
#[derive(Debug, Clone)]
pub enum ApprovalPresentation {
    Missing,
    NaturalLanguage(String),
    Structured(ManagementApprovalDecision),
}
#[derive(Debug)]
pub struct ApprovalGate {
    _max_resends: usize,
    requests: BTreeMap<String, ManagementApprovalRequest>,
    aggregation: BTreeMap<String, String>,
    uses: BTreeSet<String>,
    dispatches: u64,
}
impl ApprovalGate {
    pub fn new(max_resends: usize) -> Self {
        Self {
            _max_resends: max_resends,
            requests: BTreeMap::new(),
            aggregation: BTreeMap::new(),
            uses: BTreeSet::new(),
            dispatches: 0,
        }
    }
    pub fn dispatches(&self) -> u64 {
        self.dispatches
    }
    pub fn issue_request(
        &mut self,
        p: &ManagementActionProposal,
        human: &str,
        channel: &str,
        requested: &WallTimestamp,
        expires: &WallTimestamp,
    ) -> Result<ManagementApprovalRequest, ManagementDenial> {
        let key = format!("{}|{human}|{channel}", p.proposal_digest);
        if let Some(id) = self.aggregation.get(&key) {
            return self
                .requests
                .get(id)
                .cloned()
                .ok_or_else(|| deny("aggregated request missing"));
        }
        let challenge = digest(
            &serde_json::json!({"proposal_digest":p.proposal_digest,"human":human,"channel":channel,"requested_at":requested.as_str()}),
        )?;
        let request_id = format!("mar_{}", &challenge[7..23]);
        let request = ManagementApprovalRequest {
            aggregation_key: Some(key.clone()),
            challenge_digest: Digest(challenge),
            channel_identity: channel.to_owned(),
            confirmation_surface: ManagementApprovalRequestConfirmationSurface::ChatStructured,
            expires_at: expires.as_str().to_owned(),
            human_principal: human.to_owned(),
            method: "chat_structured".to_owned(),
            proposal_digest: Digest(p.proposal_digest.clone()),
            proposal_ref: format!("management-proposal://{}", p.proposal_id),
            proposer_actor_chain_digest: Digest(p.proposer_actor_chain_digest.clone()),
            proposer_principal: p.proposer_principal.clone(),
            request_id: request_id.clone(),
            requested_at: requested.as_str().to_owned(),
            risk_class: ManagementApprovalRequestRiskClass::R1,
            schema_version:
                ManagementApprovalRequestSchemaVersion::CognitiveosManagementApprovalRequest01,
            session_ref: None,
            single_use: true,
        };
        self.aggregation.insert(key, request_id.clone());
        self.requests.insert(request_id, request.clone());
        Ok(request)
    }
    pub fn test_decision(
        &self,
        r: &ManagementApprovalRequest,
        approver: &str,
        chain: String,
    ) -> ManagementApprovalDecision {
        ManagementApprovalDecision {
            approver_actor_chain_digest: Digest(chain),
            approver_principal: approver.to_owned(),
            authority_signature: "os-authority-signature".to_owned(),
            challenge_digest: r.challenge_digest.clone(),
            decided_at: r.requested_at.clone(),
            deciding_authority: "authority://cognitiveos/management".to_owned(),
            decision: ManagementApprovalDecisionDecision::Approve,
            decision_digest: Digest(r.challenge_digest.0.clone()),
            decision_id: format!("mad_{}", &r.request_id[4..]),
            expires_at: r.expires_at.clone(),
            independent_from_proposer: Some(true),
            object_version: 1,
            policy_version: 1,
            proposal_digest: r.proposal_digest.clone(),
            proposal_ref: r.proposal_ref.clone(),
            request_ref: Some(format!("management-approval-request://{}", r.request_id)),
            risk_class: ManagementApprovalDecisionRiskClass::R1,
            safe_reason_codes: None,
            schema_version:
                ManagementApprovalDecisionSchemaVersion::CognitiveosManagementApprovalDecision01,
            session_ref: format!("approval-context://{}", r.request_id),
            single_use: Some(true),
            step_up_method: None,
        }
    }
    pub fn authorize(
        &mut self,
        p: &ManagementActionProposal,
        r: &ManagementApprovalRequest,
        presentation: ApprovalPresentation,
        now: &WallTimestamp,
    ) -> Result<(), ManagementDenial> {
        let d = match presentation {
            ApprovalPresentation::Structured(d) => d,
            ApprovalPresentation::Missing | ApprovalPresentation::NaturalLanguage(_) => {
                return Err(deny("structured approval decision required"));
            }
        };
        let expected_ref = format!("management-approval-request://{}", r.request_id);
        let expiry =
            WallTimestamp::parse(&r.expires_at).map_err(|_| deny("invalid request expiry"))?;
        if p.risk_class != RiskClass::R1
            || r.proposal_digest.0 != p.proposal_digest
            || d.proposal_digest.0 != p.proposal_digest
            || d.challenge_digest != r.challenge_digest
            || d.request_ref.as_deref() != Some(&expected_ref)
            || d.single_use != Some(true)
            || now.instant_key() >= expiry.instant_key()
            || self.uses.contains(&r.request_id)
        {
            return Err(deny(
                "approval missing, stale, replayed, or challenge-mismatched",
            ));
        }
        if d.approver_principal == p.proposer_principal
            || d.approver_actor_chain_digest.0 == p.proposer_actor_chain_digest
        {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::ManagementSelfAuthorizationDenied,
                "proposer-entangled approval denied",
            ));
        }
        self.uses.insert(r.request_id.clone());
        self.dispatches += 1;
        Ok(())
    }
}
fn digest(value: &Value) -> Result<String, ManagementDenial> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(value)
        .map_err(|e| deny(format!("canonical approval: {e}")))?;
    cognitive_contracts::canonical::digest(&bytes, DOMAIN)
        .map_err(|e| deny(format!("approval digest: {e}")))
}
fn deny(detail: impl Into<String>) -> ManagementDenial {
    ManagementDenial::new(
        RegisteredErrorCode::ManagementIndependentApprovalRequired,
        detail,
    )
}
