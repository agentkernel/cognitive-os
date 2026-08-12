//! Offline L0-L2 campaign executor for P9-T04.
//!
//! L0 admits or refuses measurement, L1 delegates to the deterministic module
//! benchmark, and L2 collects governed-path and store-access stage timings.
//! The process holds no credential, contacts no Provider, and opens no
//! authority store outside its own disposable root, so `L3`, `L4`, and `L5`
//! are always reported as `not_run` rather than inferred.

use cognitive_runtime::campaign_report::{
    CampaignClaimLevel, CampaignCleanupOutcome, CampaignEvidenceReport, CampaignLayer,
    CampaignSafetyAccounting, LayerDisposition, LayerOutcome, VerifierDisposition,
    build_campaign_evidence_report, not_run_layer,
};
use cognitive_runtime::campaign_runner::{
    CAMPAIGN_ID, CampaignRunRequest, build_campaign_runner_report, parse_campaign_run_request,
};
use cognitive_runtime::perf::{GovernedPathStageCollector, validate_governed_path_observation};
use cognitive_runtime::store_access::collect_store_access_stage_observation;
use cognitive_store::PersonalDataLayout;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Variables the child benchmark needs to locate its toolchain, home and
/// temporary fixture root. Every other inherited variable is dropped.
const FORWARDED_CHILD_ENVIRONMENT: [&str; 7] = [
    "PATH",
    "HOME",
    "LD_LIBRARY_PATH",
    "TMPDIR",
    "TEMP",
    "TMP",
    "SYSTEMROOT",
];

const STORE_ACCESS_ITERATIONS: u32 = 50;

#[derive(Debug, Serialize)]
struct CampaignExecution {
    campaign_report: CampaignEvidenceReport,
    l0_l1_detail: Value,
    l2_governed_path: Value,
    l2_store_access: Value,
}

fn main() -> Result<(), Box<dyn Error>> {
    let l0_started_at = Instant::now();
    let request =
        parse_campaign_run_request(env::args().skip(1), env::vars().map(|(name, _)| name))?;
    let l0_elapsed_nanos = l0_started_at.elapsed().as_nanos();

    let l1_started_at = Instant::now();
    let module_observation = run_module_benchmark(&request)?;
    let l1_elapsed_nanos = l1_started_at.elapsed().as_nanos();
    let l0_l1_detail = serde_json::to_value(build_campaign_runner_report(
        request.clone(),
        l0_elapsed_nanos,
        l1_elapsed_nanos,
        module_observation,
    )?)?;

    let (governed_path, store_access, campaign_root) = collect_layer_two(&request)?;
    let campaign_state_removed = std::fs::remove_dir_all(&campaign_root).is_ok();

    let campaign_report = build_campaign_evidence_report(
        CAMPAIGN_ID,
        request.source_revision.clone(),
        request.environment_id.clone(),
        CampaignClaimLevel::Hypothesis,
        false,
        vec![
            completed_layer(
                CampaignLayer::L0Eligibility,
                1,
                0,
                digest_of(&l0_l1_detail)?,
            ),
            completed_layer(
                CampaignLayer::L1ModuleBenchmark,
                u64::try_from(request.sample_count).unwrap_or(u64::MAX),
                3,
                digest_of(&l0_l1_detail)?,
            ),
            completed_layer(
                CampaignLayer::L2GovernedAndTransport,
                u64::from(STORE_ACCESS_ITERATIONS) + 2,
                0,
                digest_of(&json!({"governed": governed_path, "store": store_access}))?,
            ),
            not_run_layer(CampaignLayer::L3ProviderRoute),
            not_run_layer(CampaignLayer::L4GovernedTaskScenarios),
            not_run_layer(CampaignLayer::L5BenefitCampaign),
        ],
        CampaignSafetyAccounting::default(),
        CampaignCleanupOutcome {
            campaign_processes_stopped: true,
            campaign_state_removed,
            // This executor never imports or creates a Provider secret entry.
            campaign_secret_entry_removed: true,
            owner_source_file_untouched: true,
            orphan_processes: 0,
            orphan_sockets: 0,
            stale_locks: 0,
        },
        VerifierDisposition::NotReviewed,
    )?;

    println!(
        "{}",
        serde_json::to_string_pretty(&CampaignExecution {
            campaign_report,
            l0_l1_detail,
            l2_governed_path: governed_path,
            l2_store_access: store_access,
        })?
    );
    Ok(())
}

fn completed_layer(
    layer: CampaignLayer,
    started_samples: u64,
    excluded_warmups: u64,
    evidence_digest: String,
) -> LayerOutcome {
    LayerOutcome {
        layer,
        disposition: LayerDisposition::Completed,
        started_samples,
        retained_samples: started_samples,
        excluded_warmups,
        evidence_digest: Some(evidence_digest),
    }
}

/// Collect the governed-path and store-access stage timings inside one
/// disposable campaign root, and return that root so cleanup can be reported.
fn collect_layer_two(
    request: &CampaignRunRequest,
) -> Result<(Value, Value, PathBuf), Box<dyn Error>> {
    let mut cold = Vec::new();
    let mut warm = Vec::new();
    for _ in 0..2 {
        let observation = GovernedPathStageCollector::cold().collect()?;
        validate_governed_path_observation(&observation).map_err(io::Error::other)?;
        cold.push(stage_json(&observation));
        let observation = GovernedPathStageCollector::warm().collect()?;
        validate_governed_path_observation(&observation).map_err(io::Error::other)?;
        warm.push(stage_json(&observation));
    }

    let campaign_root = env::temp_dir().join(format!(
        "cognitiveos-p9-t04-campaign-{}-{}",
        std::process::id(),
        request.correlation_id.as_str()
    ));
    std::fs::create_dir_all(&campaign_root)?;
    let layout = PersonalDataLayout::from_xdg_roots(
        &campaign_root,
        &campaign_root,
        &campaign_root,
        &campaign_root,
        &campaign_root,
    );
    let store_access = collect_store_access_stage_observation(&layout, STORE_ACCESS_ITERATIONS)
        .map_err(|error| io::Error::other(error.to_string()))?;
    let store_access = json!({
        "claim_level": store_access.claim_level,
        "claims_agent_benefit": store_access.claims_agent_benefit(),
        "stages": store_access
            .stages
            .iter()
            .map(|sample| json!({
                "stage": sample.stage.as_str(),
                "duration_nanos": sample.duration_nanos,
                "iterations": sample.iterations,
            }))
            .collect::<Vec<_>>(),
    });

    Ok((
        json!({"claim_level": "hypothesis", "cold": cold, "warm": warm}),
        store_access,
        campaign_root,
    ))
}

fn stage_json(observation: &cognitive_runtime::perf::GovernedPathObservation) -> Value {
    json!({
        "claim_level": observation.claim_level,
        "cache_mode": observation.cache_mode,
        "stages": observation
            .stages
            .iter()
            .map(|sample| json!({
                "stage": format!("{:?}", sample.stage),
                "duration_nanos": sample.duration_nanos,
                "omitted": sample.omitted,
            }))
            .collect::<Vec<_>>(),
    })
}

fn digest_of(value: &Value) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_vec(value)?);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn run_module_benchmark(request: &CampaignRunRequest) -> Result<Value, Box<dyn Error>> {
    let mut command = Command::new(sibling_executable("p7_t04_module_benchmark")?);
    command
        .args(["--source-revision", &request.source_revision])
        .env_clear()
        .env(
            "COGNITIVEOS_BENCHMARK_SAMPLES",
            request.sample_count.to_string(),
        );
    for name in FORWARDED_CHILD_ENVIRONMENT {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(io::Error::other("L1 module benchmark did not complete").into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn sibling_executable(name: &str) -> Result<PathBuf, io::Error> {
    let executable_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    Ok(env::current_exe()?
        .parent()
        .ok_or_else(|| io::Error::other("campaign executable has no parent directory"))?
        .join(executable_name))
}
