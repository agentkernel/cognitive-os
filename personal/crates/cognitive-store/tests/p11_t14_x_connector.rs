#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P11-T14 D01: X/Twitter connector walking skeleton.
//!
//! Failure-first: raw secret, evasion, publish-without-HITL, receipt-as-
//! completion, unknown metrics as 0, scraped content, P0 hero path, secret
//! in status. Green path proves SecretStore bind → original preview →
//! confirm → persist-before-dispatch → honest unknown readback.

use cognitive_store::{
    ConfirmCaller, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    SqliteAuthorityStore, XConnectorBindSpec, XConnectorConfirmSpec, XConnectorDispatchSpec,
    XConnectorPreviewSpec, XConnectorStore, prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    SqliteAuthorityStore,
    ProjectAggregateStore,
    XConnectorStore,
) {
    let temporary = TempDir::new().expect("temp");
    let root = temporary.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    );
    prepare_personal_databases(&layout).expect("prepare");
    let path = layout.authority_database_path();
    let authority = SqliteAuthorityStore::open(&path).expect("authority");
    let projects = ProjectAggregateStore::open_path(&path).expect("projects");
    let connector = XConnectorStore::from_authority_store(&authority);
    (temporary, authority, projects, connector)
}

fn activate(projects: &ProjectAggregateStore) -> String {
    let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
    projects
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = projects
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("G1")
        .new_ref
}

fn bind(connector: &XConnectorStore, project_id: &str, now_ms: i64) -> String {
    connector
        .bind_account(
            ConfirmCaller::OwnerManagement,
            &XConnectorBindSpec {
                project_id,
                handle: "@owner",
                secret_ref: "secretref:opaque-x-handle",
                consent: "owner-per-source",
                argv: &[],
                env_pairs: &[],
                hero_claim: false,
                default_demo: false,
                p0_success_path: false,
                platform_qualified_claim: false,
                evasion: false,
                now_ms,
            },
        )
        .expect("bind")
        .account_id
}

#[test]
fn p11_t14_raw_secret_is_rejected() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let raw = connector.bind_account(
        ConfirmCaller::OwnerManagement,
        &XConnectorBindSpec {
            project_id: &project_id,
            handle: "@owner",
            secret_ref: "sk-live-twitter",
            consent: "owner-per-source",
            argv: &[],
            env_pairs: &[],
            hero_claim: false,
            default_demo: false,
            p0_success_path: false,
            platform_qualified_claim: false,
            evasion: false,
            now_ms: 20,
        },
    );
    match raw {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("raw secret"), "{detail}");
        }
        other => panic!("expected raw secret reject, got {other:?}"),
    }
    let env = connector.bind_account(
        ConfirmCaller::OwnerManagement,
        &XConnectorBindSpec {
            project_id: &project_id,
            handle: "@owner",
            secret_ref: "secretref:opaque-x-handle",
            consent: "owner-per-source",
            argv: &[],
            env_pairs: &[("TWITTER_TOKEN", "Bearer aaaa")],
            hero_claim: false,
            default_demo: false,
            p0_success_path: false,
            platform_qualified_claim: false,
            evasion: false,
            now_ms: 21,
        },
    );
    match env {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("raw secret"), "{detail}");
        }
        other => panic!("expected env secret reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_evasion_is_rejected() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let captcha = connector.bind_account(
        ConfirmCaller::OwnerManagement,
        &XConnectorBindSpec {
            project_id: &project_id,
            handle: "@owner",
            secret_ref: "secretref:opaque-x-handle",
            consent: "owner-per-source",
            argv: &["--solve-captcha"],
            env_pairs: &[],
            hero_claim: false,
            default_demo: false,
            p0_success_path: false,
            platform_qualified_claim: false,
            evasion: false,
            now_ms: 30,
        },
    );
    match captcha {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("evasion"), "{detail}");
        }
        other => panic!("expected evasion reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_hero_path_is_rejected() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let hero = connector.bind_account(
        ConfirmCaller::OwnerManagement,
        &XConnectorBindSpec {
            project_id: &project_id,
            handle: "@owner",
            secret_ref: "secretref:opaque-x-handle",
            consent: "owner-per-source",
            argv: &[],
            env_pairs: &[],
            hero_claim: true,
            default_demo: false,
            p0_success_path: false,
            platform_qualified_claim: false,
            evasion: false,
            now_ms: 40,
        },
    );
    match hero {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("P0 hero"), "{detail}");
        }
        other => panic!("expected hero reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_scraped_content_is_rejected() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let account_id = bind(&connector, &project_id, 50);
    let scraped = connector.request_preview(
        ConfirmCaller::OwnerManagement,
        &XConnectorPreviewSpec {
            account_id: &account_id,
            project_id: &project_id,
            content: "copied from someone else",
            content_kind: "scraped",
            rights_attestation: "original-owner-rights",
            evasion: false,
            chat_approve: false,
            now_ms: 51,
        },
    );
    match scraped {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("scraped"), "{detail}");
        }
        other => panic!("expected scraped reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_publish_without_hitl_confirm_is_rejected() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let account_id = bind(&connector, &project_id, 60);
    let preview = connector
        .request_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorPreviewSpec {
                account_id: &account_id,
                project_id: &project_id,
                content: "original note from the owner",
                content_kind: "original",
                rights_attestation: "original-owner-rights",
                evasion: false,
                chat_approve: false,
                now_ms: 61,
            },
        )
        .expect("preview");
    let dispatch = connector.dispatch_publish(
        ConfirmCaller::OwnerManagement,
        &XConnectorDispatchSpec {
            preview_id: &preview.preview_id,
            claim_complete: false,
            impressions: None,
            now_ms: 62,
        },
    );
    match dispatch {
        Err(ProjectAggregateError::Unconfirmed { detail }) => {
            assert!(detail.contains("HITL confirm"), "{detail}");
        }
        other => panic!("expected unconfirmed reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_receipt_is_not_completion() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let account_id = bind(&connector, &project_id, 70);
    let preview = connector
        .request_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorPreviewSpec {
                account_id: &account_id,
                project_id: &project_id,
                content: "original note from the owner",
                content_kind: "original",
                rights_attestation: "original-owner-rights",
                evasion: false,
                chat_approve: false,
                now_ms: 71,
            },
        )
        .expect("preview");
    connector
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorConfirmSpec {
                preview_id: &preview.preview_id,
                expected_digest: &preview.content_digest,
                chat_approve: false,
                now_ms: 72,
            },
        )
        .expect("confirm");
    let claimed = connector.dispatch_publish(
        ConfirmCaller::OwnerManagement,
        &XConnectorDispatchSpec {
            preview_id: &preview.preview_id,
            claim_complete: true,
            impressions: None,
            now_ms: 73,
        },
    );
    match claimed {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("receipt is not completion"), "{detail}");
        }
        other => panic!("expected receipt-as-completion reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_unknown_metrics_never_serialize_as_zero() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let account_id = bind(&connector, &project_id, 80);
    let preview = connector
        .request_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorPreviewSpec {
                account_id: &account_id,
                project_id: &project_id,
                content: "original note from the owner",
                content_kind: "original",
                rights_attestation: "original-owner-rights",
                evasion: false,
                chat_approve: false,
                now_ms: 81,
            },
        )
        .expect("preview");
    connector
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorConfirmSpec {
                preview_id: &preview.preview_id,
                expected_digest: &preview.content_digest,
                chat_approve: false,
                now_ms: 82,
            },
        )
        .expect("confirm");
    let zero = connector.dispatch_publish(
        ConfirmCaller::OwnerManagement,
        &XConnectorDispatchSpec {
            preview_id: &preview.preview_id,
            claim_complete: false,
            impressions: Some("0"),
            now_ms: 83,
        },
    );
    match zero {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("unknown metrics"), "{detail}");
        }
        other => panic!("expected unknown≠0 reject, got {other:?}"),
    }
}

#[test]
fn p11_t14_secrets_are_not_in_status() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let account_id = bind(&connector, &project_id, 90);
    let status = connector
        .status(ConfirmCaller::OwnerManagement, &account_id)
        .expect("status");
    let debug = format!("{status:?}");
    assert!(!debug.contains("secretref:"));
    assert!(!debug.contains("sk-"));
    assert!(!status.platform_qualified);
    assert!(!status.is_p0_hero);
    assert_eq!(status.impressions, "unknown");
    assert!(status.receipt_is_not_completion);
}

#[test]
fn p11_t14_green_path_bind_preview_confirm_dispatch_unknown_readback() {
    let (_tmp, _authority, projects, connector) = stores();
    let project_id = activate(&projects);
    let account_id = bind(&connector, &project_id, 100);
    let preview = connector
        .request_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorPreviewSpec {
                account_id: &account_id,
                project_id: &project_id,
                content: "original note from the owner",
                content_kind: "original",
                rights_attestation: "original-owner-rights",
                evasion: false,
                chat_approve: false,
                now_ms: 101,
            },
        )
        .expect("preview");
    assert_eq!(preview.content_digest.len(), 64);
    connector
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &XConnectorConfirmSpec {
                preview_id: &preview.preview_id,
                expected_digest: &preview.content_digest,
                chat_approve: false,
                now_ms: 102,
            },
        )
        .expect("confirm");
    let published = connector
        .dispatch_publish(
            ConfirmCaller::OwnerManagement,
            &XConnectorDispatchSpec {
                preview_id: &preview.preview_id,
                claim_complete: false,
                impressions: Some("unknown"),
                now_ms: 103,
            },
        )
        .expect("dispatch");
    assert!(published.intent_persisted);
    assert!(published.dispatched);
    assert_eq!(published.readback_status, "unknown");
    assert_eq!(published.impressions, "unknown");
    assert!(published.receipt_is_not_completion);
    let status = connector
        .status(ConfirmCaller::OwnerManagement, &account_id)
        .expect("status");
    assert!(status.confirmed);
    assert!(status.dispatched);
    assert_eq!(status.impressions, "unknown");
    assert_ne!(status.impressions, "0");
    assert!(!status.platform_qualified);
}
