//! Deterministic module benchmark entry point for P7-T04/D01.
//!
//! This binary emits raw local timing observations for production module APIs.
//! It does not establish a release threshold, performance non-inferiority, or
//! an Agent-benefit claim. A controlled D04/D05 campaign consumes such
//! observations only after it records environment, revision, and comparison
//! prerequisites separately.

use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{UriRef, WallTimestamp};
use cognitive_kernel::authz::{
    ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance, PrincipalFacts,
};
use cognitive_kernel::context::{
    ArrivalOrderRanker, CandidateObject, ContextBudget, RenderSpec, ResolutionRequest, resolve,
};
use cognitive_kernel::context_cache::{
    ContextCacheEntry, ContextCacheKey, ContextCacheLookup, ContextSourceDigest, DerivedCacheKind,
    GovernanceBinding, GovernedContextCache,
};
use cognitive_runtime::GovernanceOverheadSample;
use cognitive_store::ArtifactStore;
use cognitive_store::scheduler::{
    SchedulerRepository, SchedulerRow, SchedulerState, SchedulerWorkKey,
};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

const DEFAULT_SAMPLE_COUNT: usize = 25;
const WARMUP_ITERATIONS: usize = 3;

#[derive(Serialize)]
struct BenchmarkReport {
    report_kind: &'static str,
    claim_level: &'static str,
    source_revision: String,
    fixture_digest: String,
    warmup_iterations: usize,
    samples_per_benchmark: usize,
    benchmarks: Vec<BenchmarkObservation>,
}

#[derive(Serialize)]
struct BenchmarkObservation {
    benchmark_id: &'static str,
    unit: &'static str,
    sample_count: usize,
    p50: u128,
    p95: u128,
    p99: u128,
    minimum: u128,
    maximum: u128,
    raw_samples: Vec<u128>,
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create(label: &str) -> Result<Self, io::Error> {
        let directory_name = format!(
            "cognitiveos-p7-t04-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos(),
        );
        let path = env::temp_dir().join(directory_name);
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_revision = parse_source_revision(env::args().skip(1))?;
    let sample_count = parse_sample_count()?;
    let fixture_digest = calculate_fixture_digest()?;

    let benchmarks = vec![
        measure_context_resolution(sample_count)?,
        measure_context_cache_hit(sample_count)?,
        measure_artifact_cas_publish(sample_count)?,
        measure_scheduler_eligible_cas(sample_count)?,
        measure_canonical_report_serialization(sample_count)?,
    ];

    let report = BenchmarkReport {
        report_kind: "p7-t04-d01-module-observation/0.1",
        claim_level: "hypothesis",
        source_revision,
        fixture_digest,
        warmup_iterations: WARMUP_ITERATIONS,
        samples_per_benchmark: sample_count,
        benchmarks,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn parse_source_revision(arguments: impl Iterator<Item = String>) -> Result<String, io::Error> {
    let arguments = arguments.collect::<Vec<_>>();
    let source_revision = arguments
        .windows(2)
        .find(|argument_pair| argument_pair[0] == "--source-revision")
        .map(|argument_pair| argument_pair[1].as_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--source-revision <immutable-git-revision> is required",
            )
        })?;
    let is_exact_revision_length = source_revision.len() == 40 || source_revision.len() == 64;
    let is_hex_revision = is_exact_revision_length
        && source_revision
            .bytes()
            .all(|character| character.is_ascii_hexdigit());
    if !is_hex_revision {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--source-revision must be a 40 or 64 character hexadecimal revision",
        ));
    }
    Ok(source_revision.to_owned())
}

fn parse_sample_count() -> Result<usize, io::Error> {
    match env::var("COGNITIVEOS_BENCHMARK_SAMPLES") {
        Ok(value) => value.parse::<usize>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "COGNITIVEOS_BENCHMARK_SAMPLES must be a positive integer",
            )
        }),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_SAMPLE_COUNT),
        Err(error) => Err(io::Error::other(error)),
    }
    .and_then(|sample_count| {
        if sample_count == 0 {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "COGNITIVEOS_BENCHMARK_SAMPLES must be a positive integer",
            ))
        } else {
            Ok(sample_count)
        }
    })
}

fn calculate_fixture_digest() -> Result<String, serde_json::Error> {
    let fixture = json!({
        "context_candidates": ["authorized", "wrong_scope", "wrong_tenant"],
        "cache_key": "complete-governance-binding",
        "artifact_payload_prefix": "p7-t04-artifact",
        "scheduler_state": "runnable",
        "report": "governance-overhead-schema-shape"
    });
    let encoded_fixture = serde_json::to_vec(&fixture)?;
    Ok(format!("sha256:{:x}", Sha256::digest(encoded_fixture)))
}

fn measure_context_resolution(sample_count: usize) -> Result<BenchmarkObservation, Box<dyn Error>> {
    let request = context_request()?;
    let candidates = vec![
        context_candidate(
            "knowledge://tenant-a/authorized",
            "tenant-a",
            "scope://tenant-a/kb",
        ),
        context_candidate(
            "knowledge://tenant-a/wrong-scope",
            "tenant-a",
            "scope://tenant-a/private",
        ),
        context_candidate(
            "knowledge://tenant-b/wrong-tenant",
            "tenant-b",
            "scope://tenant-b/kb",
        ),
    ];
    let ranker = ArrivalOrderRanker;
    measure("context-resolution-filter-builder", sample_count, |_| {
        let view = resolve(&request, &candidates, &ranker)
            .map_err(|error| io::Error::other(error.to_string()))?;
        if view.loaded.len() != 1 {
            return Err(Box::new(io::Error::other(
                "fixture no longer yields one authorized Context item",
            )));
        }
        Ok(())
    })
}

fn measure_context_cache_hit(sample_count: usize) -> Result<BenchmarkObservation, Box<dyn Error>> {
    let cache_key = context_cache_key();
    let cache_entry = ContextCacheEntry {
        render_digest: format!("sha256:{}", "4d".repeat(32)),
        stable_prefix_segment_digests: vec![format!("sha256:{}", "5e".repeat(32))],
        delta_segment_digests: vec![format!("sha256:{}", "6f".repeat(32))],
        derived: vec![DerivedCacheKind::KvCache],
    };
    let mut cache = GovernedContextCache::default();
    cache.insert(cache_key.clone(), cache_entry);

    measure(
        "context-cache-full-key-hit",
        sample_count,
        |_| match cache.lookup_current(&cache_key) {
            ContextCacheLookup::Hit(_) => Ok(()),
            ContextCacheLookup::MissResolveFresh => Err(Box::new(io::Error::other(
                "fixture unexpectedly missed the full Context cache key",
            ))),
        },
    )
}

fn measure_artifact_cas_publish(
    sample_count: usize,
) -> Result<BenchmarkObservation, Box<dyn Error>> {
    let directory = TemporaryDirectory::create("artifact-cas")?;
    let store = ArtifactStore::open(directory.path(), 2_000_000)?;

    measure(
        "artifact-cas-immutable-publish-readback",
        sample_count,
        |sample_index| {
            let payload = format!("p7-t04-artifact-{sample_index:04}");
            let reference = store.put(payload.as_bytes())?;
            let restored = store.get(&reference)?;
            if restored.as_deref() != Some(payload.as_bytes()) {
                return Err(Box::new(io::Error::other(
                    "published Artifact CAS bytes did not round-trip",
                )));
            }
            Ok(())
        },
    )
}

fn measure_scheduler_eligible_cas(
    sample_count: usize,
) -> Result<BenchmarkObservation, Box<dyn Error>> {
    let directory = TemporaryDirectory::create("scheduler")?;
    let mut repository = SchedulerRepository::open(&directory.path().join("scheduler.sqlite"))?;
    let total_operation_count = sample_count + WARMUP_ITERATIONS;
    let work_keys =
        (0..total_operation_count)
            .map(
                |sample_index| -> Result<
                    SchedulerWorkKey,
                    cognitive_store::scheduler::SchedulerRepositoryError,
                > {
                    let task_reference = format!("task://tenant-a/p7-t04/{sample_index}");
                    repository.upsert(&SchedulerRow {
                        task_ref: task_reference.clone(),
                        contract_epoch: 1,
                        state: SchedulerState::Runnable.as_str().to_owned(),
                        lease_owner: None,
                        lease_epoch: 0,
                        lease_expires: None,
                        next_eligible: "2026-08-10T00:00:00Z".to_owned(),
                        attempt_count: 0,
                        cancel_requested: false,
                    })?;
                    Ok(SchedulerWorkKey {
                        task_ref: task_reference,
                        contract_epoch: 1,
                    })
                },
            )
            .collect::<Result<Vec<_>, cognitive_store::scheduler::SchedulerRepositoryError>>()?;

    measure(
        "scheduler-eligible-lease-cas",
        sample_count,
        |sample_index| {
            let leased = repository.acquire_eligible_lease(
                &work_keys[sample_index],
                "p7-t04-benchmark-worker",
                1,
                "2026-08-10T00:00:00Z",
                "2026-08-10T00:01:00Z",
            )?;
            if leased.lease_owner.as_deref() != Some("p7-t04-benchmark-worker") {
                return Err(Box::new(io::Error::other(
                    "scheduler CAS did not persist benchmark owner",
                )));
            }
            Ok(())
        },
    )
}

fn measure_canonical_report_serialization(
    sample_count: usize,
) -> Result<BenchmarkObservation, Box<dyn Error>> {
    // This builder sample only supplies a schema-shaped payload to serialize.
    // Its values are never emitted as measured governance-overhead evidence.
    let report_sample = GovernanceOverheadSample::documented_builder_sample();
    measure(
        "canonical-performance-report-serialization",
        sample_count,
        |_| {
            let report = report_sample.to_report_json();
            let digest = report_sample.report_digest().map_err(io::Error::other)?;
            if !digest.starts_with("sha256:") || report.get("comparison").is_some() {
                return Err(Box::new(io::Error::other(
                    "serialization fixture violated its non-claim boundary",
                )));
            }
            Ok(())
        },
    )
}

fn measure<F>(
    benchmark_id: &'static str,
    sample_count: usize,
    mut operation: F,
) -> Result<BenchmarkObservation, Box<dyn Error>>
where
    F: FnMut(usize) -> Result<(), Box<dyn Error>>,
{
    for warmup_index in 0..WARMUP_ITERATIONS {
        operation(warmup_index)?;
    }

    let mut samples = Vec::with_capacity(sample_count);
    for sample_index in 0..sample_count {
        let started_at = Instant::now();
        operation(sample_index + WARMUP_ITERATIONS)?;
        samples.push(started_at.elapsed().as_nanos());
    }

    let mut sorted_samples = samples.clone();
    sorted_samples.sort_unstable();
    Ok(BenchmarkObservation {
        benchmark_id,
        unit: "nanoseconds",
        sample_count,
        p50: nearest_rank(&sorted_samples, 50),
        p95: nearest_rank(&sorted_samples, 95),
        p99: nearest_rank(&sorted_samples, 99),
        minimum: sorted_samples[0],
        maximum: sorted_samples[sorted_samples.len() - 1],
        raw_samples: samples,
    })
}

fn nearest_rank(sorted_samples: &[u128], percentile: usize) -> u128 {
    let rank = (percentile * sorted_samples.len()).div_ceil(100);
    sorted_samples[rank.saturating_sub(1)]
}

fn timestamp(value: &str) -> Result<WallTimestamp, Box<dyn Error>> {
    Ok(WallTimestamp::parse(value)?)
}

fn uri(value: &str) -> Result<UriRef, Box<dyn Error>> {
    Ok(UriRef::parse(value)?)
}

fn context_request() -> Result<ResolutionRequest, Box<dyn Error>> {
    Ok(ResolutionRequest {
        snapshot: AuthzSnapshot {
            tenant_id: "tenant-a".to_owned(),
            principal: PrincipalFacts {
                principal_ref: uri("principal://tenant-a/agent-1")?,
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
                    not_before: timestamp("2026-08-10T00:00:00Z")?,
                    expires: timestamp("2026-08-10T01:00:00Z")?,
                },
                depth_remaining: 1,
                issued_epoch: 7,
            }],
            capability_set_version: 7,
            explicit_denies: Vec::new(),
            revocation_epoch: 9,
            decided_at: timestamp("2026-08-10T00:30:00Z")?,
        },
        purpose: "task_execution".to_owned(),
        conversation_ref: None,
        required: Vec::new(),
        allow_partial: false,
        budget: ContextBudget {
            context_bytes: Some(4096),
            input_tokens: Some(512),
        },
        render: RenderSpec {
            renderer_version: "p7-t04-benchmark-renderer/1".to_owned(),
            target_profile: "structured/v1".to_owned(),
        },
        schema_digest: format!("sha256:{}", "b2".repeat(32)),
    })
}

fn context_candidate(object_ref: &str, tenant_id: &str, resource_scope: &str) -> CandidateObject {
    CandidateObject {
        object_ref: object_ref.to_owned(),
        object_version: 1,
        content_digest: format!("sha256:{}", "c3".repeat(32)),
        governance: ObjectGovernance {
            object_ref: object_ref.to_owned(),
            tenant_id: Some(tenant_id.to_owned()),
            owner_ref: "principal://tenant-a/librarian".to_owned(),
            resource_scope: resource_scope.to_owned(),
            conversation_ref: None,
        },
        role: LoadedContextItemRole::Evidence,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        body: json!({"ref": object_ref, "text": "deterministic benchmark body"}),
        cost_bytes: 48,
        cost_tokens: 12,
    }
}

fn context_cache_key() -> ContextCacheKey {
    ContextCacheKey {
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
        context_request_id: "context-request://tenant-a/p7-t04".to_owned(),
        context_request_digest: format!("sha256:{}", "f6".repeat(32)),
        task_ref: "task://tenant-a/p7-t04".to_owned(),
        task_contract_epoch: 1,
        task_contract_digest: format!("sha256:{}", "17".repeat(32)),
        ordered_source_digests: vec![ContextSourceDigest {
            source_ref: "knowledge://tenant-a/benchmark-source".to_owned(),
            content_digest: format!("sha256:{}", "28".repeat(32)),
        }],
        renderer_version: "p7-t04-benchmark-renderer/1".to_owned(),
        validated_tool_descriptor_digest: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_fixture_digest, parse_source_revision};

    #[test]
    fn requires_an_exact_hex_source_revision() {
        let short_revision = parse_source_revision(
            ["--source-revision", "abcdef012345"]
                .map(str::to_owned)
                .into_iter(),
        );
        assert!(short_revision.is_err());

        let exact_revision = "a".repeat(40);
        assert!(matches!(
            parse_source_revision(
                ["--source-revision".to_owned(), exact_revision.clone()].into_iter()
            ),
            Ok(revision) if revision == exact_revision
        ));
    }

    #[test]
    fn fixture_digest_is_a_sha256_identifier() {
        assert!(matches!(
            calculate_fixture_digest().as_deref(),
            Ok(digest) if digest.len() == 71 && digest.starts_with("sha256:")
        ));
    }
}
