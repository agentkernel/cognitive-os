//! D-018 / REQ-EVT-001 publication-envelope behavior.
#![allow(clippy::unwrap_used)]
use cognitive_contracts::generated::{
    common_defs::{Digest, Lineage, Provenance, Retention, ValidTime},
    governed_object_header::{
        GovernedObjectHeader, GovernedObjectHeaderScopeDomain, GovernedObjectHeaderSensitivity,
    },
    object_reference::{StrongReference, StrongReferenceKind, UuidV7},
};
use cognitive_domain::{EventId, LifecycleDomain, ObjectId, Version};
use cognitive_kernel::ports::{CommittedEvent, GovernanceObjectStore, StorePortError};
use cognitive_runtime::assemble_persisted_event;

struct HeaderStore(Option<GovernedObjectHeader>);

impl GovernanceObjectStore for HeaderStore {
    fn load_governed_object_header(
        &self,
        _object_id: &ObjectId,
    ) -> Result<Option<GovernedObjectHeader>, StorePortError> {
        Ok(self.0.clone())
    }
}
fn strong(n: &str) -> StrongReference {
    StrongReference {
        content_digest: Digest(format!("sha256:{}", n.repeat(64)[..64].to_owned())),
        id: UuidV7(format!("00000000-0000-7000-8000-{n:0>12}")),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    }
}
#[test]
fn committed_internal_fact_becomes_registered_governed_event() {
    let event=CommittedEvent{sequence:7,event_id:EventId::parse("00000000-0000-7000-8000-000000000007").unwrap(),object_id:ObjectId::parse("00000000-0000-7000-9000-000000000042").unwrap(),domain:LifecycleDomain::Task,object_version:Version::INITIAL,event_type:"task.updated".to_owned(),canonical_json:r#"{"causation":{"causation_id":"event://cause","correlation_id":"corr://m5"},"event_time":"2026-07-21T00:00:00Z","payload":{"state":"ACTIVE"}}"#.to_owned()};
    let header = GovernedObjectHeader {
        authority_ref: strong("2"),
        compartments: vec![],
        content_digest: Digest(format!("sha256:{}", "a".repeat(64))),
        created_at: "2026-07-21T00:00:00Z".to_owned(),
        id: UuidV7("00000000-0000-7000-8000-000000000007".to_owned()),
        lineage: Lineage {
            parents: vec![],
            transform: "outbox-publication".to_owned(),
        },
        object_version: 1,
        owner_ref: strong("1"),
        policy_refs: vec![],
        provenance: Provenance {
            created_by: "authority://cognitiveos/state".to_owned(),
            source_refs: vec!["event://internal/7".to_owned()],
        },
        purpose_constraints: vec!["watch".to_owned()],
        resource_scope_ref: strong("3"),
        retention: Retention {
            expires_at: None,
            legal_hold: false,
            policy: "event-log".to_owned(),
        },
        schema_version: "cognitiveos.event/0.1".to_owned(),
        scope_domain: GovernedObjectHeaderScopeDomain::Platform,
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        tenant_id: None,
        r#type: "Event".to_owned(),
        valid_time: ValidTime {
            from: "2026-07-21T00:00:00Z".to_owned(),
            until: None,
        },
    };
    let out = assemble_persisted_event(&HeaderStore(Some(header)), &event, "2026-07-21T00:00:01Z")
        .unwrap();
    assert_eq!(out["header"]["type"], "Event");
    assert_eq!(out["payload"]["state"], "ACTIVE");
    assert_eq!(out["immutable"], true);
}

#[test]
fn publication_fails_closed_without_a_durable_governance_header() {
    let event = CommittedEvent {
        sequence: 7,
        event_id: EventId::parse("00000000-0000-7000-8000-000000000007").unwrap(),
        object_id: ObjectId::parse("00000000-0000-7000-9000-000000000042").unwrap(),
        domain: LifecycleDomain::Task,
        object_version: Version::INITIAL,
        event_type: "task.updated".to_owned(),
        canonical_json:
            r#"{"causation":{"correlation_id":"corr://m5"},"event_time":"2026-07-21T00:00:00Z"}"#
                .to_owned(),
    };
    let err =
        assemble_persisted_event(&HeaderStore(None), &event, "2026-07-21T00:00:01Z").unwrap_err();
    assert!(err.to_string().contains("governance header"));
}
