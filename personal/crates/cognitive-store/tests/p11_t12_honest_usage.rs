#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T12/D01 honest usage: unknown≠0, labelled cost, silent rebind, secret isolation.

use cognitive_store::{
    AgentProviderBindingRecord, CostOutcome, NewUsageEvent, PersonalDataLayout,
    ProviderAccountRecord, ProviderControlPlaneError, ProviderControlPlaneStore,
    ProviderModelRecord, UsageSample, compute_cost, labelled_cost_source,
    prepare_personal_databases,
};
use serde_json::Value;
use tempfile::TempDir;

fn store() -> (TempDir, ProviderControlPlaneStore) {
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
    let store =
        ProviderControlPlaneStore::open_path(&layout.authority_database_path()).expect("open");
    (temporary, store)
}

fn account(name: &str) -> ProviderAccountRecord {
    let now = 1_700_000_000_000;
    ProviderAccountRecord {
        account_id: format!("acct-{name}"),
        display_name: name.to_owned(),
        provider_kind: "openai_official".to_owned(),
        endpoint: "https://api.openai.com/v1".to_owned(),
        secret_ref: Some("secretref:opaque-handle".to_owned()),
        allow_private_network: false,
        allow_insecure_http: false,
        network_scope: "public".to_owned(),
        status: "active".to_owned(),
        catalog_revision: 1,
        last_discovery_error: None,
        created_at_ms: now,
        updated_at_ms: now,
    }
}

fn priced_model() -> ProviderModelRecord {
    ProviderModelRecord {
        account_id: "acct-openai-work".to_owned(),
        model_id: "gpt-4o".to_owned(),
        source: "manually_configured".to_owned(),
        pricing_version: Some("manual-1".to_owned()),
        price_input_per_million: Some("2.50".to_owned()),
        price_output_per_million: Some("10.00".to_owned()),
        price_cache_read_per_million: Some("1.25".to_owned()),
        price_cache_write_per_million: Some("2.50".to_owned()),
        catalog_revision: 1,
    }
}

fn binding(account_id: &str, model_id: &str) -> AgentProviderBindingRecord {
    AgentProviderBindingRecord {
        agent_instance_id: "agent://personal/pi".to_owned(),
        account_id: account_id.to_owned(),
        model_id: model_id.to_owned(),
        revision: 1,
        status: "active".to_owned(),
    }
}

fn usage_event(
    event_id: &str,
    metering_source: &str,
    sample: UsageSample,
    cost: CostOutcome,
) -> NewUsageEvent {
    NewUsageEvent {
        event_id: event_id.to_owned(),
        idempotency_key: event_id.to_owned(),
        recorded_at_ms: 1_700_000_000_000,
        account_id: "acct-openai-work".to_owned(),
        provider_kind: "openai_official".to_owned(),
        model_id: "gpt-4o".to_owned(),
        agent_instance_id: "agent://personal/pi".to_owned(),
        sample,
        duration_ms: Some(12),
        outcome: "ok".to_owned(),
        metering_source: metering_source.to_owned(),
        estimation_method: None,
        cost,
    }
}

fn assert_unknown_cost_never_zero(serialized: &str, value: &Value) {
    assert!(
        serialized.contains("\"cost\":\"unknown\""),
        "unknown cost must serialize as the literal unknown: {serialized}"
    );
    assert!(
        !serialized.contains("\"cost\":0"),
        "unknown cost must not serialize as JSON number 0: {serialized}"
    );
    assert!(
        !serialized.contains("\"cost\":\"0\""),
        "unknown cost must not serialize as string 0: {serialized}"
    );
    for event in value["events"].as_array().unwrap_or(&Vec::new()) {
        if event["cost_label"] == "unknown" || event["cost_status"] == "cost_unavailable" {
            assert_eq!(event["cost"], "unknown");
            assert!(event["cost_micros"].is_null(), "{event}");
        }
    }
    for row in value["accounts"].as_array().unwrap_or(&Vec::new()) {
        let quota = &row["quota"];
        assert_eq!(quota["status"], "unknown");
        assert!(quota["allowance"].is_null(), "{quota}");
        assert!(quota["remaining"].is_null(), "{quota}");
        assert!(quota.get("secret_ref").is_none());
        assert!(row["account"].get("secret_ref").is_none());
    }
}

#[test]
fn p11_t12_unknown_cost_never_serializes_as_zero() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    let sample = UsageSample {
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let cost = compute_cost(&sample, None);
    store
        .record_usage(&usage_event("evt-unknown", "unavailable", sample, cost))
        .expect("record");
    let body = store.honest_usage_read_model().expect("read");
    let serialized = body.to_string();
    assert_unknown_cost_never_zero(&serialized, &body);
    assert_eq!(body["events"][0]["cost_label"], "unknown");
    assert_eq!(body["events"][0]["metering_source"], "unavailable");
}

#[test]
fn p11_t12_labelled_read_maps_existing_enums_honestly() {
    assert_eq!(
        labelled_cost_source("provider_reported", "priced"),
        "actual"
    );
    assert_eq!(
        labelled_cost_source("locally_estimated", "priced"),
        "estimated"
    );
    assert_eq!(
        labelled_cost_source("unavailable", "cost_unavailable"),
        "unknown"
    );
    assert_eq!(
        labelled_cost_source("provider_reported", "cost_unavailable"),
        "unknown"
    );
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    let model = priced_model();
    store.upsert_manual_model(&model).expect("model");
    let priced_sample = UsageSample {
        input_tokens: Some(1_000_000),
        output_tokens: Some(1_000),
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let priced = compute_cost(&priced_sample, Some(&model));
    store
        .record_usage(&usage_event(
            "evt-actual",
            "provider_reported",
            priced_sample,
            priced,
        ))
        .expect("actual");
    let unknown_sample = UsageSample {
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    store
        .record_usage(&usage_event(
            "evt-unknown",
            "unavailable",
            unknown_sample.clone(),
            compute_cost(&unknown_sample, None),
        ))
        .expect("unknown");
    let body = store.honest_usage_read_model().expect("read");
    let events = body["events"].as_array().expect("events");
    let actual = events
        .iter()
        .find(|event| event["event_id"] == "evt-actual")
        .expect("actual event");
    let unknown = events
        .iter()
        .find(|event| event["event_id"] == "evt-unknown")
        .expect("unknown event");
    assert_eq!(actual["cost_label"], "actual");
    assert_eq!(actual["metering_source"], "provider_reported");
    assert!(actual["cost"].is_number());
    assert_ne!(actual["cost"], 0);
    assert_eq!(unknown["cost_label"], "unknown");
    assert_eq!(unknown["cost"], "unknown");
    assert!(unknown["cost_micros"].is_null());
    let serialized = body.to_string();
    assert!(
        !serialized.contains("\"cost_label\":\"estimated\""),
        "must not pretend locally_estimated when that metering_source was unused: {serialized}"
    );
}

#[test]
fn p11_t12_secret_never_lands_in_usage_read_model() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    let body = store.honest_usage_read_model().expect("read");
    let serialized = body.to_string();
    assert!(!serialized.contains("secret_ref"));
    assert!(!serialized.contains("secretref:"));
    assert!(!serialized.contains("sk-"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("Bearer "));
}

#[test]
fn p11_t12_silent_rebind_is_rejected() {
    let (_tmp, store) = store();
    store.insert_account(&account("openai-work")).expect("a");
    store.insert_account(&account("openai-lab")).expect("b");
    store
        .set_binding(&binding("acct-openai-work", "gpt-4o"), 10)
        .expect("first bind");
    let error = store
        .set_binding(&binding("acct-openai-lab", "gpt-4o-mini"), 11)
        .expect_err("silent rebind");
    assert!(matches!(
        error,
        ProviderControlPlaneError::Conflict { detail } if detail.contains("silent rebind")
    ));
    let kept = store
        .get_active_binding("agent://personal/pi")
        .expect("get")
        .expect("present");
    assert_eq!(kept.account_id, "acct-openai-work");
    assert_eq!(kept.model_id, "gpt-4o");
    let replaced = store
        .replace_binding(&binding("acct-openai-lab", "gpt-4o-mini"), 1, 12)
        .expect("explicit rebind");
    assert_eq!(replaced.account_id, "acct-openai-lab");
    assert_eq!(replaced.model_id, "gpt-4o-mini");
}

#[test]
fn p11_t12_binding_explanation_is_durable_and_unbound_at_missing_layers() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    store
        .set_binding(&binding("acct-openai-work", "gpt-4o"), 10)
        .expect("bind");
    let body = store.honest_usage_read_model().expect("read");
    let layers = body["binding_explanation"]["layers"]
        .as_array()
        .expect("layers");
    assert_eq!(layers.len(), 4);
    assert_eq!(layers[0]["layer"], "global");
    assert_eq!(layers[0]["status"], "bound");
    assert_eq!(
        layers[0]["bindings"][0]["agent_instance_id"],
        "agent://personal/pi"
    );
    assert_eq!(layers[1]["layer"], "project");
    assert_eq!(layers[1]["status"], "unbound");
    assert_eq!(layers[2]["layer"], "employee");
    assert_eq!(layers[2]["status"], "unbound");
    assert_eq!(layers[3]["layer"], "task");
    assert_eq!(layers[3]["status"], "unbound");
    for layer in layers.iter().skip(1) {
        assert!(layer.get("cost").is_none(), "{layer}");
        assert_ne!(layer["status"], 0);
    }
    let serialized = body.to_string();
    assert!(!serialized.contains("\"cost\":0"));
    assert!(!serialized.contains("\"cost\":\"0\""));
}

#[test]
fn p11_t12_account_and_quota_fields_are_separated() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    let body = store.honest_usage_read_model().expect("read");
    let row = &body["accounts"][0];
    assert_eq!(row["account"]["id"], "acct-openai-work");
    assert_eq!(row["account"]["display_name"], "openai-work");
    assert!(row["account"].get("allowance").is_none());
    assert!(row["account"].get("remaining").is_none());
    assert!(row["account"].get("secret_ref").is_none());
    assert_eq!(row["quota"]["status"], "unknown");
    assert_eq!(row["quota"]["source"], "unavailable");
    assert!(row["quota"]["allowance"].is_null());
    assert!(row["quota"].get("id").is_none());
}
