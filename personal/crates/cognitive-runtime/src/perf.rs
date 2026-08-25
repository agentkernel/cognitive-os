//! Governance overhead baseline reporting (REQ-PERF-004 / IMP-04).
//!
//! Declares an ungoverned baseline. Does **not** emit REQ-PERF-005 agent
//! benefit claims.

use cognitive_contracts::canonical;
use serde_json::{Value, json};

#[derive(Debug, Clone, Default)]
pub struct StageLatencyMs {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone)]
pub struct GovernanceOverheadSample {
    pub ungoverned_baseline: String,
    pub authorization: StageLatencyMs,
    pub context_resolution: StageLatencyMs,
    pub effect_protocol: StageLatencyMs,
    pub cache_hit_preservation_ratio: f64,
    pub extra_writes: f64,
    pub extra_bytes: f64,
    pub approval_latency: StageLatencyMs,
    pub rubber_stamp_rate: f64,
    pub retry_after_deny_rate: f64,
    pub overhead_latency_percent_r1: f64,
    pub overhead_cost_percent_r1: f64,
}

impl GovernanceOverheadSample {
    /// Fixed report-builder sample used to validate report plumbing only.
    /// These values are never measurements or performance claims.
    pub fn documented_builder_sample() -> Self {
        Self {
            ungoverned_baseline: "ungoverned-local-v1".into(),
            authorization: StageLatencyMs {
                p50: 0.1,
                p95: 0.4,
                p99: 0.9,
            },
            context_resolution: StageLatencyMs {
                p50: 1.0,
                p95: 3.0,
                p99: 5.0,
            },
            effect_protocol: StageLatencyMs {
                p50: 0.5,
                p95: 1.2,
                p99: 2.0,
            },
            cache_hit_preservation_ratio: 0.9,
            extra_writes: 2.0,
            extra_bytes: 1024.0,
            approval_latency: StageLatencyMs {
                p50: 10.0,
                p95: 50.0,
                p99: 100.0,
            },
            rubber_stamp_rate: 0.01,
            retry_after_deny_rate: 0.02,
            overhead_latency_percent_r1: 3.0,
            overhead_cost_percent_r1: 2.0,
        }
    }

    /// Build a schema-shaped performance report fragment for governance overhead.
    /// Callers must supply measured numbers — never copy vector fixture values.
    pub fn to_report_json(&self) -> Value {
        json!({
            "schema_version": "cognitiveos.performance-report/0.1",
            "benchmark_manifest": {
                "workload": {"name": "m6-governance-overhead"},
                "model": {
                    "provider": "n/a",
                    "model": "deterministic-gates",
                    "revision": "sha256:0000000000000000000000000000000000000000000000000000000000000001",
                    "sampling": {"temperature": 0}
                },
                "hardware_topology": {"nodes": 1},
                "concurrency": 1,
                "datasets": [{"id": "m6-overhead", "version": "1"}],
                "fault_profile": {"name": "none"},
                "risk_class": "R1",
                "samples": 1,
                "confidence_interval": {"level": 0.95, "method": "none"},
                "baselines": [self.ungoverned_baseline],
                "ablations": [],
                "latency_boundaries": {
                    "mechanism": "kernel gates only",
                    "model_tool_network": "separately measured"
                },
                "execution_state": "warm"
            },
            "slo_profile": {
                "id": "m6-overhead",
                "version": "1",
                "window": "measured"
            },
            "metrics": [{
                "name": "governance_overhead_share",
                "category": "governance_overhead",
                "unit": "percent",
                "numerator": self.overhead_latency_percent_r1,
                "denominator": 100.0,
                "window": {"start": "measured", "end": "measured"},
                "p50": self.overhead_latency_percent_r1,
                "p95": self.overhead_latency_percent_r1,
                "p99": self.overhead_latency_percent_r1,
                "sample_count": 1,
                "confidence_interval": {
                    "level": 0.95,
                    "low": self.overhead_latency_percent_r1,
                    "high": self.overhead_latency_percent_r1,
                    "method": "none"
                }
            }],
            "safety_failures": [],
            "governance_overhead": {
                "ungoverned_baseline": self.ungoverned_baseline,
                "gate_latency_ms": {
                    "authorization": {
                        "p50": self.authorization.p50,
                        "p95": self.authorization.p95,
                        "p99": self.authorization.p99
                    },
                    "context_resolution": {
                        "p50": self.context_resolution.p50,
                        "p95": self.context_resolution.p95,
                        "p99": self.context_resolution.p99
                    },
                    "effect_protocol": {
                        "p50": self.effect_protocol.p50,
                        "p95": self.effect_protocol.p95,
                        "p99": self.effect_protocol.p99
                    }
                },
                "cache_hit_preservation_ratio": self.cache_hit_preservation_ratio,
                "extra_persistence_per_governed_call": {
                    "writes": self.extra_writes,
                    "bytes": self.extra_bytes
                },
                "approval": {
                    "latency_ms": {
                        "p50": self.approval_latency.p50,
                        "p95": self.approval_latency.p95,
                        "p99": self.approval_latency.p99
                    },
                    "rubber_stamp_rate": self.rubber_stamp_rate,
                    "retry_after_deny_rate": self.retry_after_deny_rate
                },
                "overhead_share_by_risk_class": [{
                    "risk_class": "R1",
                    "latency_percent": self.overhead_latency_percent_r1,
                    "cost_percent": self.overhead_cost_percent_r1
                }]
            },
            "tail_latency_disclosed": true
        })
    }

    pub fn report_digest(&self) -> Result<String, String> {
        let report = self.to_report_json();
        let bytes = canonical::canonical_bytes_of_value(&report).map_err(|e| e.to_string())?;
        canonical::digest(&bytes, "performance-report/0.1").map_err(|e| e.to_string())
    }

    pub fn declares_ungoverned_baseline(&self) -> bool {
        !self.ungoverned_baseline.is_empty()
    }

    pub fn claims_agent_benefit(&self) -> bool {
        false
    }
}

/// Named stages on the single daemon-owned governed path measured by D02.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernedPathStage {
    Authorization,
    ContextResolution,
    CacheReuse,
    EffectPersistence,
}

impl GovernedPathStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Authorization => "authorization",
            Self::ContextResolution => "context_resolution",
            Self::CacheReuse => "cache_reuse",
            Self::EffectPersistence => "effect_persistence",
        }
    }
}

/// One raw stage sample. Tail percentiles are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernedStageSample {
    pub stage: GovernedPathStage,
    pub duration_nanos: u128,
    pub omitted: bool,
}

/// Bounded counters for one governed-path observation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GovernedPathCounters {
    pub authorization_grants: u64,
    pub context_items_loaded: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub intents_persisted: u64,
    pub omitted_stages: u64,
}

/// Hypothesis-only observation from one real governed path execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernedPathObservation {
    pub claim_level: &'static str,
    pub cache_mode: &'static str,
    pub stages: Vec<GovernedStageSample>,
    pub counters: GovernedPathCounters,
}

/// Executes one daemon-owned authorize→Context→cache→Intent path with
/// monotonic stage timing. It never fabricates p95/p99 or Agent benefit.
#[derive(Debug, Clone, Copy)]
pub struct GovernedPathStageCollector {
    pub warm_cache: bool,
    pub omit_effect_persistence: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum GovernedPathCollectionError {
    #[error("{0}")]
    Failed(String),
}

impl From<Box<dyn std::error::Error>> for GovernedPathCollectionError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Failed(error.to_string())
    }
}

impl From<String> for GovernedPathCollectionError {
    fn from(error: String) -> Self {
        Self::Failed(error)
    }
}

impl GovernedPathStageCollector {
    pub fn cold() -> Self {
        Self {
            warm_cache: false,
            omit_effect_persistence: false,
        }
    }

    pub fn warm() -> Self {
        Self {
            warm_cache: true,
            omit_effect_persistence: false,
        }
    }

    pub fn with_omitted_effect_persistence(mut self) -> Self {
        self.omit_effect_persistence = true;
        self
    }

    pub fn collect(self) -> Result<GovernedPathObservation, GovernedPathCollectionError> {
        use cognitive_contracts::generated::context_view::{
            LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
        };
        use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
        use cognitive_domain::{LifecycleDomain, ObjectId, UriRef, Version, WallTimestamp};
        use cognitive_kernel::authz::{
            AccessRequest, ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance,
            PrincipalFacts, authorize,
        };
        use cognitive_kernel::context::{
            ArrivalOrderRanker, CandidateObject, ContextBudget, RenderSpec, ResolutionRequest,
            resolve,
        };
        use cognitive_kernel::context_cache::{
            ContextCacheEntry, ContextCacheKey, ContextCacheLookup, ContextSourceDigest,
            DerivedCacheKind, GovernanceBinding, GovernedContextCache,
        };
        use cognitive_kernel::effects::{
            EffectClass, IntentCommand, MintedIntent, OperationDescriptor, WriterLease, mint_intent,
        };
        use cognitive_kernel::executor::ExecutorCapabilities;
        use cognitive_kernel::ports::{Clock, IdGenerator, PortFailure, ProtocolStore};
        use cognitive_kernel::{AdmitCommand, TransitionEngine};
        use cognitive_store::SqliteAuthorityStore;
        use serde_json::json;
        use std::path::PathBuf;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Instant;

        struct FixedClock(WallTimestamp);
        impl Clock for FixedClock {
            fn now(&self) -> Result<WallTimestamp, PortFailure> {
                Ok(self.0.clone())
            }
        }

        struct SequenceIds(AtomicU64);
        impl IdGenerator for SequenceIds {
            fn next_uuid_v7(&self) -> Result<String, PortFailure> {
                let sequence = self.0.fetch_add(1, Ordering::SeqCst);
                Ok(format!("00000000-0000-7000-8000-{sequence:012x}"))
            }
        }

        fn parse_uri(value: &str) -> Result<UriRef, GovernedPathCollectionError> {
            UriRef::parse(value)
                .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))
        }

        fn parse_timestamp(value: &str) -> Result<WallTimestamp, GovernedPathCollectionError> {
            WallTimestamp::parse(value)
                .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))
        }

        fn object_id(sequence: u64) -> Result<ObjectId, GovernedPathCollectionError> {
            ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}"))
                .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))
        }

        let decided_at = parse_timestamp("2026-08-10T00:30:00Z")?;
        let snapshot = AuthzSnapshot {
            tenant_id: "tenant-a".to_owned(),
            principal: PrincipalFacts {
                principal_ref: parse_uri("principal://tenant-a/agent-1")?,
                authenticated: true,
                active: true,
                tenant_id: Some("tenant-a".to_owned()),
            },
            actor_chain: ActorChainFacts {
                chain_digest: format!("sha256:{}", "a1".repeat(32)),
                resolved: true,
            },
            membership: Some(MembershipFacts {
                valid: true,
                roles: ["member".to_owned()].into(),
            }),
            capability_links: vec![CapabilityConstraints {
                subject: "principal://tenant-a/agent-1".to_owned(),
                audience: "service://tenant-a/context".to_owned(),
                resource: "scope://tenant-a/kb".to_owned(),
                purpose: "task_execution".to_owned(),
                actions: ["read_body".to_owned()].into(),
                parameter_bounds: Default::default(),
                lease: LeaseWindow {
                    not_before: parse_timestamp("2026-08-10T00:00:00Z")?,
                    expires: parse_timestamp("2026-08-10T01:00:00Z")?,
                },
                depth_remaining: 1,
                issued_epoch: 10,
            }],
            capability_set_version: 7,
            explicit_denies: Vec::new(),
            revocation_epoch: 9,
            decided_at: decided_at.clone(),
        };
        let governance = ObjectGovernance {
            object_ref: "knowledge://tenant-a/authorized".to_owned(),
            tenant_id: Some("tenant-a".to_owned()),
            owner_ref: "principal://tenant-a/librarian".to_owned(),
            resource_scope: "scope://tenant-a/kb".to_owned(),
            conversation_ref: None,
        };
        let request = ResolutionRequest {
            snapshot: snapshot.clone(),
            purpose: "task_execution".to_owned(),
            conversation_ref: None,
            required: Vec::new(),
            allow_partial: false,
            budget: ContextBudget {
                context_bytes: Some(4096),
                input_tokens: Some(512),
            },
            render: RenderSpec {
                renderer_version: "p7-t04-d02-renderer/1".to_owned(),
                target_profile: "structured/v1".to_owned(),
            },
            schema_digest: format!("sha256:{}", "b2".repeat(32)),
        };
        let candidates = vec![CandidateObject {
            object_ref: governance.object_ref.clone(),
            object_version: 1,
            content_digest: format!("sha256:{}", "c3".repeat(32)),
            governance: governance.clone(),
            role: LoadedContextItemRole::Evidence,
            trust_level: LoadedContextItemTrustLevel::Verified,
            representation: LoadedContextItemRepresentation::Text,
            body: json!({"text": "governed-path stage body"}),
            cost_bytes: 48,
            cost_tokens: 12,
        }];
        let cache_key = ContextCacheKey {
            governance: GovernanceBinding {
                tenant: "tenant-a".to_owned(),
                actor_chain_digest: format!("sha256:{}", "d4".repeat(32)),
                capability_set_version: 7,
                revocation_epoch: 9,
                purpose: "task_execution".to_owned(),
                schema_digest: format!("sha256:{}", "e5".repeat(32)),
                encoding_profile: "structured/v1".to_owned(),
                conversation: None,
            },
            context_request_id: "context-request://tenant-a/p7-t04-d02".to_owned(),
            context_request_digest: format!("sha256:{}", "f6".repeat(32)),
            task_ref: "task://tenant-a/p7-t04-d02".to_owned(),
            task_contract_epoch: 1,
            task_contract_digest: format!("sha256:{}", "17".repeat(32)),
            ordered_source_digests: vec![ContextSourceDigest {
                source_ref: "knowledge://tenant-a/authorized".to_owned(),
                content_digest: format!("sha256:{}", "28".repeat(32)),
            }],
            renderer_version: "p7-t04-d02-renderer/1".to_owned(),
            validated_tool_descriptor_digest: None,
        };

        let mut stages = Vec::with_capacity(4);
        let mut counters = GovernedPathCounters::default();

        let authorization_started = Instant::now();
        authorize(
            &snapshot,
            &governance,
            &AccessRequest {
                action: "read_body".to_owned(),
                purpose: "task_execution".to_owned(),
            },
        )
        .map_err(|error| GovernedPathCollectionError::Failed(error.denial.code.to_owned()))?;
        counters.authorization_grants = 1;
        stages.push(GovernedStageSample {
            stage: GovernedPathStage::Authorization,
            duration_nanos: authorization_started.elapsed().as_nanos(),
            omitted: false,
        });

        let context_started = Instant::now();
        let view = resolve(&request, &candidates, &ArrivalOrderRanker).map_err(|error| {
            GovernedPathCollectionError::Failed(format!("context resolution failed: {error}"))
        })?;
        counters.context_items_loaded = u64::try_from(view.loaded.len()).unwrap_or(u64::MAX);
        if counters.context_items_loaded != 1 {
            return Err(GovernedPathCollectionError::Failed(
                "governed path fixture must load exactly one Context item".to_owned(),
            ));
        }
        stages.push(GovernedStageSample {
            stage: GovernedPathStage::ContextResolution,
            duration_nanos: context_started.elapsed().as_nanos(),
            omitted: false,
        });

        let mut cache = GovernedContextCache::default();
        if self.warm_cache {
            cache.insert(
                cache_key.clone(),
                ContextCacheEntry {
                    render_digest: format!("sha256:{}", "4d".repeat(32)),
                    stable_prefix_segment_digests: vec![format!("sha256:{}", "5e".repeat(32))],
                    delta_segment_digests: vec![format!("sha256:{}", "6f".repeat(32))],
                    derived: vec![DerivedCacheKind::KvCache],
                },
            );
        }
        let cache_started = Instant::now();
        match cache.lookup_current(&cache_key) {
            ContextCacheLookup::Hit(_) => counters.cache_hits = 1,
            ContextCacheLookup::MissResolveFresh => counters.cache_misses = 1,
        }
        stages.push(GovernedStageSample {
            stage: GovernedPathStage::CacheReuse,
            duration_nanos: cache_started.elapsed().as_nanos(),
            omitted: false,
        });

        if self.omit_effect_persistence {
            counters.omitted_stages = 1;
            stages.push(GovernedStageSample {
                stage: GovernedPathStage::EffectPersistence,
                duration_nanos: 0,
                omitted: true,
            });
        } else {
            let temporary_root = std::env::temp_dir().join(format!(
                "cognitiveos-p7-t04-d02-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))?
                    .as_nanos()
            ));
            std::fs::create_dir_all(&temporary_root).map_err(|error| {
                GovernedPathCollectionError::Failed(format!("create temp dir: {error}"))
            })?;
            let authority_path: PathBuf = temporary_root.join("authority.sqlite");
            let effect_started = Instant::now();
            let collect_effect = (|| -> Result<(), GovernedPathCollectionError> {
                let store = SqliteAuthorityStore::open(&authority_path)
                    .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))?;
                let clock = FixedClock(parse_timestamp("2026-08-10T00:00:00Z")?);
                let identifiers = SequenceIds(AtomicU64::new(40_000));
                let effect_id = object_id(30_001)?;
                let engine = TransitionEngine::new(&store, &clock, &identifiers);
                engine
                    .admit_object(&AdmitCommand {
                        object_id: effect_id.clone(),
                        domain: LifecycleDomain::Effect,
                        subject_ref: parse_uri("effect://tenant-a/p7-t04-d02")?,
                        body: json!({"benchmark": "p7-t04-d02"}),
                        actor_ref: parse_uri("actor://tenant-a/benchmark")?,
                        authority_ref: parse_uri("authority://tenant-a/benchmark")?,
                        correlation_id: parse_uri("correlation://tenant-a/p7-t04-d02")?,
                        outbox_destinations: Vec::new(),
                        fencing_epoch: None,
                    })
                    .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))?;
                let minted = mint_intent(
                    &store,
                    &clock,
                    &identifiers,
                    &WriterLease { epoch: 1 },
                    &IntentCommand {
                        intent_id: object_id(30_002)?,
                        effect_object_id: effect_id.clone(),
                        descriptor: OperationDescriptor {
                            operation_id: "operation://tenant-a/p7-t04/d02".to_owned(),
                            action: "benchmark.persist".to_owned(),
                            effect_class: EffectClass::GovernedExternal,
                            executor: "executor://tenant-a/p7-t04".to_owned(),
                            capabilities: ExecutorCapabilities {
                                queryable: true,
                                idempotent: false,
                            },
                            descriptor_version: 1,
                        },
                        target: "https://benchmark.invalid/p7-t04-d02".to_owned(),
                        parameters: json!({"path": "governed-stage"}),
                        idempotency_key: "p7-t04-d02-persist".to_owned(),
                        expected_state_version: Version::INITIAL,
                        grant_epoch: 1,
                        capability_set_version: 1,
                        actor_ref: parse_uri("actor://tenant-a/benchmark")?,
                        authority_ref: parse_uri("authority://tenant-a/benchmark")?,
                        correlation_id: parse_uri("correlation://tenant-a/p7-t04-d02")?,
                        task_binding: None,
                    },
                )
                .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))?;
                let MintedIntent::Persisted(intent) = minted else {
                    return Err(GovernedPathCollectionError::Failed(
                        "governed path unexpectedly replayed Intent".to_owned(),
                    ));
                };
                let restored = store
                    .load_intent_for_effect(&effect_id)
                    .map_err(|error| GovernedPathCollectionError::Failed(error.to_string()))?;
                if restored.as_ref().map(|row| &row.intent_id) != Some(&intent.intent_id) {
                    return Err(GovernedPathCollectionError::Failed(
                        "persisted Intent could not be reloaded".to_owned(),
                    ));
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(&temporary_root);
            collect_effect?;
            counters.intents_persisted = 1;
            stages.push(GovernedStageSample {
                stage: GovernedPathStage::EffectPersistence,
                duration_nanos: effect_started.elapsed().as_nanos(),
                omitted: false,
            });
        }

        Ok(GovernedPathObservation {
            claim_level: "hypothesis",
            cache_mode: if self.warm_cache { "warm" } else { "cold" },
            stages,
            counters,
        })
    }
}

/// Reject observations that invent release-tail statistics or benefit claims.
pub fn validate_governed_path_observation(
    observation: &GovernedPathObservation,
) -> Result<(), String> {
    if observation.claim_level != "hypothesis" {
        return Err("governed-path observations must remain hypothesis-only".to_owned());
    }
    if observation.stages.is_empty() {
        return Err("governed-path observation is missing stage samples".to_owned());
    }
    for sample in &observation.stages {
        if sample.omitted && sample.duration_nanos != 0 {
            return Err(format!(
                "omitted stage {} must not invent a measured duration",
                sample.stage.as_str()
            ));
        }
    }
    let effect = observation
        .stages
        .iter()
        .find(|sample| sample.stage == GovernedPathStage::EffectPersistence);
    if let Some(sample) = effect
        && sample.omitted
        && observation.counters.intents_persisted != 0
    {
        return Err(
            "omitted Effect persistence cannot claim a persisted Intent counter".to_owned(),
        );
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn overhead_report_requires_ungoverned_baseline_and_forbids_benefit() {
        let sample = GovernanceOverheadSample::documented_builder_sample();
        assert!(sample.declares_ungoverned_baseline());
        assert!(!sample.claims_agent_benefit());
        let report = sample.to_report_json();
        assert_eq!(
            report["governance_overhead"]["ungoverned_baseline"],
            "ungoverned-local-v1"
        );
        assert!(report.get("comparison").is_none());
        let digest = sample.report_digest().unwrap();
        assert!(digest.starts_with("sha256:"));
    }

    #[test]
    fn cold_governed_path_records_cache_miss_and_effect_persistence() {
        let observation = GovernedPathStageCollector::cold().collect().unwrap();
        validate_governed_path_observation(&observation).unwrap();
        assert_eq!(observation.cache_mode, "cold");
        assert_eq!(observation.counters.cache_misses, 1);
        assert_eq!(observation.counters.cache_hits, 0);
        assert_eq!(observation.counters.intents_persisted, 1);
        assert_eq!(observation.counters.omitted_stages, 0);
        assert!(observation.stages.iter().any(|sample| sample.stage
            == GovernedPathStage::EffectPersistence
            && !sample.omitted
            && sample.duration_nanos > 0));
    }

    #[test]
    fn warm_governed_path_records_cache_hit_without_fabricating_tails() {
        let observation = GovernedPathStageCollector::warm().collect().unwrap();
        validate_governed_path_observation(&observation).unwrap();
        assert_eq!(observation.cache_mode, "warm");
        assert_eq!(observation.counters.cache_hits, 1);
        assert_eq!(observation.counters.cache_misses, 0);
        assert_eq!(observation.claim_level, "hypothesis");
    }

    #[test]
    fn omitted_effect_stage_stays_zero_duration_and_unpersisted() {
        let observation = GovernedPathStageCollector::cold()
            .with_omitted_effect_persistence()
            .collect()
            .unwrap();
        validate_governed_path_observation(&observation).unwrap();
        assert_eq!(observation.counters.omitted_stages, 1);
        assert_eq!(observation.counters.intents_persisted, 0);
        let effect = observation
            .stages
            .iter()
            .find(|sample| sample.stage == GovernedPathStage::EffectPersistence)
            .unwrap();
        assert!(effect.omitted);
        assert_eq!(effect.duration_nanos, 0);
    }

    #[test]
    fn omitted_stage_validator_rejects_invented_duration() {
        let mut observation = GovernedPathStageCollector::cold()
            .with_omitted_effect_persistence()
            .collect()
            .unwrap();
        observation.stages.last_mut().unwrap().duration_nanos = 42;
        let error = validate_governed_path_observation(&observation).unwrap_err();
        assert!(error.contains("must not invent a measured duration"));
    }
}
