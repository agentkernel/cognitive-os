#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P8-T13 Provider Control Plane store negatives: secrets, cost, retention, alerts.

use cognitive_store::{
    AgentProviderBindingRecord, NewUsageEvent, PersonalDataLayout, ProviderAccountRecord,
    ProviderControlPlaneError, ProviderControlPlaneStore, ProviderModelRecord, UsageSample,
    apply_builtin_prices, compute_cost, prepare_personal_databases, usage_from_anthropic_json,
    usage_from_openai_json,
};
use serde_json::json;
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

#[test]
fn secret_material_never_lands_in_sqlite_and_audit_rejects_key_shaped_detail() {
    let (_tmp, store) = store();
    let mut record = account("openai-work");
    let fixture_key = "sk-p8t13-fixture-not-a-real-key";
    record.secret_ref = Some("secretref:abc".to_owned());
    store.insert_account(&record).expect("insert");
    assert!(
        !store.leak_scan_contains(fixture_key).expect("scan"),
        "API key fixture must not appear in control-plane tables"
    );
    assert!(
        !store.leak_scan_contains("sk-p8t13").expect("scan"),
        "key prefix must not appear in control-plane tables"
    );
    assert!(matches!(
        store.append_audit(
            "aud-1",
            1,
            "key.set",
            Some("acct-openai-work"),
            None,
            "ok",
            fixture_key
        ),
        Err(ProviderControlPlaneError::Invalid { .. })
    ));
}

#[test]
fn active_binding_blocks_account_delete_and_discovery_failure_preserves_catalog() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    store
        .upsert_manual_model(&ProviderModelRecord {
            account_id: "acct-openai-work".to_owned(),
            model_id: "gpt-4o".to_owned(),
            source: "manually_configured".to_owned(),
            pricing_version: Some("manual-1".to_owned()),
            price_input_per_million: Some("2.50".to_owned()),
            price_output_per_million: Some("10.00".to_owned()),
            price_cache_read_per_million: Some("1.25".to_owned()),
            price_cache_write_per_million: Some("2.50".to_owned()),
            catalog_revision: 1,
        })
        .expect("manual model");
    store
        .set_binding(
            &AgentProviderBindingRecord {
                agent_instance_id: "agent://personal/pi".to_owned(),
                account_id: "acct-openai-work".to_owned(),
                model_id: "gpt-4o".to_owned(),
                revision: 1,
                status: "active".to_owned(),
            },
            10,
        )
        .expect("bind");
    assert!(matches!(
        store.delete_account("acct-openai-work"),
        Err(ProviderControlPlaneError::Conflict { .. })
    ));
    store
        .mark_discovery_outcome("acct-openai-work", "degraded", 1, Some("upstream 401"), 11)
        .expect("degraded");
    let models = store.list_models("acct-openai-work").expect("list");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].source, "manually_configured");
    let binding = store
        .get_active_binding("agent://personal/pi")
        .expect("get")
        .expect("present");
    assert_eq!(binding.model_id, "gpt-4o");
    store
        .replace_discovered_models("acct-openai-work", 2, &[])
        .expect("failed refresh does not delete manuals");
    assert_eq!(
        store.list_models("acct-openai-work").expect("list").len(),
        1
    );
}

#[test]
fn unknown_usage_and_missing_price_are_cost_unavailable_not_zero() {
    let sample = UsageSample {
        input_tokens: Some(100),
        output_tokens: Some(20),
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let unpriced = compute_cost(&sample, None);
    assert_eq!(unpriced.cost_status, "cost_unavailable");
    assert!(unpriced.cost_micros.is_none());

    let unknown_input = UsageSample {
        input_tokens: None,
        output_tokens: Some(0),
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let mut model = ProviderModelRecord {
        account_id: "acct".to_owned(),
        model_id: "gpt-4o".to_owned(),
        source: "provider_discovered".to_owned(),
        pricing_version: None,
        price_input_per_million: None,
        price_output_per_million: None,
        price_cache_read_per_million: None,
        price_cache_write_per_million: None,
        catalog_revision: 1,
    };
    apply_builtin_prices("openai_official", &mut model);
    let unknown_cost = compute_cost(&unknown_input, Some(&model));
    assert_eq!(unknown_cost.cost_status, "cost_unavailable");
    assert!(unknown_cost.cost_micros.is_none());
    assert_ne!(unknown_cost.cost_micros, Some(0));

    let priced = compute_cost(&sample, Some(&model));
    assert_eq!(priced.cost_status, "priced");
    assert!(priced.cost_micros.is_some_and(|value| value > 0));
    assert!(priced.cache_hit_rate_unknown);

    let with_cache = UsageSample {
        input_tokens: Some(100),
        output_tokens: Some(10),
        cache_read_tokens: Some(40),
        cache_write_tokens: Some(5),
    };
    let cache_cost = compute_cost(&with_cache, Some(&model));
    assert!(!cache_cost.cache_hit_rate_unknown);
    assert_eq!(cache_cost.cost_status, "priced");
}

#[test]
fn openai_and_anthropic_usage_mapping_keeps_missing_cache_unknown() {
    let openai = usage_from_openai_json(&json!({
        "usage": { "prompt_tokens": 12, "completion_tokens": 3 }
    }));
    assert_eq!(openai.input_tokens, Some(12));
    assert_eq!(openai.output_tokens, Some(3));
    assert_eq!(openai.cache_read_tokens, None);

    let openai_cached = usage_from_openai_json(&json!({
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 3,
            "prompt_tokens_details": { "cached_tokens": 4 }
        }
    }));
    assert_eq!(openai_cached.cache_read_tokens, Some(4));

    let anthropic = usage_from_anthropic_json(&json!({
        "usage": {
            "input_tokens": 9,
            "output_tokens": 2,
            "cache_read_input_tokens": 1,
            "cache_creation_input_tokens": 3
        }
    }));
    assert_eq!(anthropic.cache_read_tokens, Some(1));
    assert_eq!(anthropic.cache_write_tokens, Some(3));
}

#[test]
fn duplicate_usage_is_idempotent_and_historical_cost_stays_after_price_update() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    let mut model = ProviderModelRecord {
        account_id: "acct-openai-work".to_owned(),
        model_id: "gpt-4o".to_owned(),
        source: "provider_discovered".to_owned(),
        pricing_version: Some("builtin-2026-08".to_owned()),
        price_input_per_million: Some("2.50".to_owned()),
        price_output_per_million: Some("10.00".to_owned()),
        price_cache_read_per_million: Some("1.25".to_owned()),
        price_cache_write_per_million: Some("2.50".to_owned()),
        catalog_revision: 1,
    };
    store.upsert_manual_model(&model).expect("model");
    let sample = UsageSample {
        input_tokens: Some(1_000_000),
        output_tokens: Some(0),
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let cost = compute_cost(&sample, Some(&model));
    let event = NewUsageEvent {
        event_id: "evt-1".to_owned(),
        idempotency_key: "idem-1".to_owned(),
        recorded_at_ms: 1_700_000_000_000,
        account_id: "acct-openai-work".to_owned(),
        provider_kind: "openai_official".to_owned(),
        model_id: "gpt-4o".to_owned(),
        agent_instance_id: "agent://personal/pi".to_owned(),
        sample,
        duration_ms: Some(12),
        outcome: "ok".to_owned(),
        metering_source: "provider_reported".to_owned(),
        estimation_method: None,
        cost: cost.clone(),
    };
    assert!(store.record_usage(&event).expect("first"));
    assert!(!store.record_usage(&event).expect("dup"));
    store
        .set_model_prices(
            "acct-openai-work",
            "gpt-4o",
            "manual-later",
            Some("9.99"),
            Some("9.99"),
            Some("9.99"),
            Some("9.99"),
        )
        .expect("price update");
    let events = store.list_usage_events(0).expect("events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].2, cost.cost_micros);
    assert_eq!(events[0].3, "priced");
    model.price_input_per_million = Some("9.99".to_owned());
    let later = compute_cost(
        &UsageSample {
            input_tokens: Some(1_000_000),
            output_tokens: Some(0),
            cache_read_tokens: None,
            cache_write_tokens: None,
        },
        Some(&model),
    );
    assert_ne!(later.cost_micros, cost.cost_micros);
}

#[test]
fn retention_drops_old_events_and_keeps_recent_aggregates() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    let now = 2_000_000_000_000;
    let old = now - (40 * 24 * 60 * 60 * 1000);
    let sample = UsageSample {
        input_tokens: Some(1),
        output_tokens: Some(1),
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let unavailable = compute_cost(&sample, None);
    store
        .record_usage(&NewUsageEvent {
            event_id: "old".to_owned(),
            idempotency_key: "old".to_owned(),
            recorded_at_ms: old,
            account_id: "acct-openai-work".to_owned(),
            provider_kind: "openai_official".to_owned(),
            model_id: "gpt-4o".to_owned(),
            agent_instance_id: "agent://personal/pi".to_owned(),
            sample: sample.clone(),
            duration_ms: Some(1),
            outcome: "ok".to_owned(),
            metering_source: "unavailable".to_owned(),
            estimation_method: None,
            cost: unavailable.clone(),
        })
        .expect("old");
    store
        .record_usage(&NewUsageEvent {
            event_id: "new".to_owned(),
            idempotency_key: "new".to_owned(),
            recorded_at_ms: now,
            account_id: "acct-openai-work".to_owned(),
            provider_kind: "openai_official".to_owned(),
            model_id: "gpt-4o".to_owned(),
            agent_instance_id: "agent://personal/pi".to_owned(),
            sample,
            duration_ms: Some(1),
            outcome: "ok".to_owned(),
            metering_source: "unavailable".to_owned(),
            estimation_method: None,
            cost: unavailable,
        })
        .expect("new");
    let (dropped_events, _) = store.apply_retention(now).expect("retain");
    assert_eq!(dropped_events, 1);
    assert_eq!(store.list_usage_events(0).expect("left").len(), 1);
}

#[test]
fn budget_alerts_dedupe_at_80_and_100_and_ignore_unavailable_cost_as_zero() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    store
        .upsert_budget("bud-1", "account", "acct-openai-work", Some(100), None, 1)
        .expect("budget");
    let sample = UsageSample {
        input_tokens: Some(80),
        output_tokens: Some(0),
        cache_read_tokens: None,
        cache_write_tokens: None,
    };
    let cost = compute_cost(&sample, None);
    assert_eq!(cost.cost_status, "cost_unavailable");
    store
        .record_usage(&NewUsageEvent {
            event_id: "u1".to_owned(),
            idempotency_key: "u1".to_owned(),
            recorded_at_ms: 1_800_000_000_000,
            account_id: "acct-openai-work".to_owned(),
            provider_kind: "openai_official".to_owned(),
            model_id: "gpt-4o".to_owned(),
            agent_instance_id: "agent://personal/pi".to_owned(),
            sample,
            duration_ms: Some(1),
            outcome: "ok".to_owned(),
            metering_source: "unavailable".to_owned(),
            estimation_method: None,
            cost,
        })
        .expect("usage");
    let first = store
        .maybe_issue_budget_alerts(1_800_000_000_000)
        .expect("alerts");
    assert_eq!(first, vec![("bud-1".to_owned(), "warning_80".to_owned())]);
    let again = store
        .maybe_issue_budget_alerts(1_800_000_000_000)
        .expect("dedupe");
    assert!(again.is_empty());

    store
        .upsert_budget(
            "bud-amt",
            "agent",
            "agent://personal/dsh",
            None,
            Some(1_000),
            2,
        )
        .expect("amount budget");
    let amount_alerts = store
        .maybe_issue_budget_alerts(1_800_000_000_000)
        .expect("amount");
    assert!(
        !amount_alerts.iter().any(|item| item.0 == "bud-amt"),
        "cost_unavailable must not count as zero against an amount budget"
    );
}

#[test]
fn pi_and_dsh_bindings_are_independent() {
    let (_tmp, store) = store();
    store
        .insert_account(&account("openai-work"))
        .expect("insert");
    store
        .insert_account(&account("anthropic-personal"))
        .expect("insert");
    store
        .set_binding(
            &AgentProviderBindingRecord {
                agent_instance_id: "agent://personal/pi".to_owned(),
                account_id: "acct-openai-work".to_owned(),
                model_id: "gpt-4o".to_owned(),
                revision: 1,
                status: "active".to_owned(),
            },
            1,
        )
        .expect("pi");
    store
        .set_binding(
            &AgentProviderBindingRecord {
                agent_instance_id: "agent://personal/dsh".to_owned(),
                account_id: "acct-anthropic-personal".to_owned(),
                model_id: "claude-sonnet-4-20250514".to_owned(),
                revision: 1,
                status: "active".to_owned(),
            },
            2,
        )
        .expect("dsh");
    let pi = store
        .get_active_binding("agent://personal/pi")
        .expect("pi")
        .expect("present");
    let dsh = store
        .get_active_binding("agent://personal/dsh")
        .expect("dsh")
        .expect("present");
    assert_ne!(pi.account_id, dsh.account_id);
    assert_ne!(pi.model_id, dsh.model_id);
}
