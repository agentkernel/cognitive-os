//! P9-T01 deterministic decision runner.
//!
//! This runner consumes the existing daemon-owned governed-path collector. It
//! reports raw hypothesis observations and applies the preregistered rule
//! without changing any authority path or making a performance claim.

use cognitive_runtime::{
    GovernedPathStage, GovernedPathStageCollector, GovernedStageSample,
    validate_governed_path_observation,
};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::io;

const DEFAULT_RUN_COUNT: usize = 2;
const DEFAULT_SAMPLES_PER_RUN: usize = 5;

#[derive(Debug, Serialize)]
struct DecisionReport {
    report_kind: &'static str,
    claim_level: &'static str,
    source_revision: String,
    run_count: usize,
    samples_per_run: usize,
    runs: Vec<RunReport>,
    decision: Decision,
}

#[derive(Debug, Serialize)]
struct RunReport {
    run_index: usize,
    cold: StageSummary,
    warm: StageSummary,
}

#[derive(Debug, Serialize)]
struct StageSummary {
    cache_mode: &'static str,
    stages: Vec<StageTiming>,
}

#[derive(Debug, Serialize)]
struct StageTiming {
    stage: &'static str,
    p50_nanos: u128,
    p95_nanos: u128,
    p99_nanos: u128,
    samples: Vec<u128>,
}

#[derive(Debug, Serialize)]
struct Decision {
    outcome: &'static str,
    dominant_stage: Option<&'static str>,
    dominant_share_of_cold_p95: Option<f64>,
    rationale: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_revision = parse_source_revision(env::args().skip(1))?;
    let run_count = parse_positive_setting("P9_T01_RUNS", DEFAULT_RUN_COUNT)?;
    let samples_per_run = parse_positive_setting("P9_T01_SAMPLES", DEFAULT_SAMPLES_PER_RUN)?;
    let mut runs = Vec::with_capacity(run_count);
    let mut cold_effect_dominant = true;

    for run_index in 0..run_count {
        let cold_summary = summarize("cold", collect_samples(false, samples_per_run)?)?;
        let warm_summary = summarize("warm", collect_samples(true, samples_per_run)?)?;
        cold_effect_dominant &= is_effect_dominant(&cold_summary);
        runs.push(RunReport {
            run_index: run_index + 1,
            cold: cold_summary,
            warm: warm_summary,
        });
    }

    let (dominant_stage, dominant_share) = cold_dominant_stage(&runs);
    let should_migrate = cold_effect_dominant
        && dominant_stage == Some("effect_persistence")
        && dominant_share >= 0.5;
    let decision = if should_migrate {
        Decision {
            outcome: "stream-only-async-migration-candidate",
            dominant_stage,
            dominant_share_of_cold_p95: Some(dominant_share),
            rationale: "effect persistence dominates cold p95 in every native run; only the stream layer may be staged for async migration",
        }
    } else {
        Decision {
            outcome: "conservative-no-migration",
            dominant_stage,
            dominant_share_of_cold_p95: Some(dominant_share),
            rationale: "the preregistered reproducible I/O dominance rule was not satisfied; retain the synchronous path",
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&DecisionReport {
            report_kind: "p9-t01-async-decision/0.1",
            claim_level: "hypothesis",
            source_revision,
            run_count,
            samples_per_run,
            runs,
            decision,
        })?
    );
    Ok(())
}

fn collect_samples(
    warm_cache: bool,
    sample_count: usize,
) -> Result<Vec<Vec<GovernedStageSample>>, Box<dyn Error>> {
    let mut stage_samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        let observation = if warm_cache {
            GovernedPathStageCollector::warm().collect()?
        } else {
            GovernedPathStageCollector::cold().collect()?
        };
        validate_governed_path_observation(&observation).map_err(io::Error::other)?;
        stage_samples.push(observation.stages);
    }
    Ok(stage_samples)
}

fn summarize(
    cache_mode: &'static str,
    samples: Vec<Vec<GovernedStageSample>>,
) -> Result<StageSummary, Box<dyn Error>> {
    let stage_order = [
        GovernedPathStage::Authorization,
        GovernedPathStage::ContextResolution,
        GovernedPathStage::CacheReuse,
        GovernedPathStage::EffectPersistence,
    ];
    let mut stages = Vec::with_capacity(stage_order.len());
    for stage in stage_order {
        let durations = samples
            .iter()
            .filter_map(|run| run.iter().find(|sample| sample.stage == stage))
            .filter(|sample| !sample.omitted)
            .map(|sample| sample.duration_nanos)
            .collect::<Vec<_>>();
        if durations.is_empty() {
            return Err(io::Error::other("decision runner received no stage samples").into());
        }
        stages.push(StageTiming {
            stage: stage_name(stage),
            p50_nanos: percentile(&durations, 0.50),
            p95_nanos: percentile(&durations, 0.95),
            p99_nanos: percentile(&durations, 0.99),
            samples: durations,
        });
    }
    Ok(StageSummary { cache_mode, stages })
}

fn percentile(values: &[u128], quantile: f64) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len() as f64 * quantile).ceil() as usize).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn is_effect_dominant(summary: &StageSummary) -> bool {
    let total_p95 = summary
        .stages
        .iter()
        .map(|stage| stage.p95_nanos)
        .sum::<u128>();
    let effect_p95 = summary
        .stages
        .iter()
        .find(|stage| stage.stage == "effect_persistence")
        .map(|stage| stage.p95_nanos)
        .unwrap_or_default();
    effect_p95.saturating_mul(2) >= total_p95
}

fn cold_dominant_stage(runs: &[RunReport]) -> (Option<&'static str>, f64) {
    let Some(first_run) = runs.first() else {
        return (None, 0.0);
    };
    let total_p95 = first_run
        .cold
        .stages
        .iter()
        .map(|stage| stage.p95_nanos)
        .sum::<u128>();
    let dominant = first_run
        .cold
        .stages
        .iter()
        .max_by_key(|stage| stage.p95_nanos);
    let Some(dominant) = dominant else {
        return (None, 0.0);
    };
    let share = if total_p95 == 0 {
        0.0
    } else {
        dominant.p95_nanos as f64 / total_p95 as f64
    };
    (Some(dominant.stage), share)
}

fn stage_name(stage: GovernedPathStage) -> &'static str {
    match stage {
        GovernedPathStage::Authorization => "authorization",
        GovernedPathStage::ContextResolution => "context_resolution",
        GovernedPathStage::CacheReuse => "cache_reuse",
        GovernedPathStage::EffectPersistence => "effect_persistence",
    }
}

fn parse_positive_setting(name: &str, default: usize) -> Result<usize, io::Error> {
    let value = match env::var(name) {
        Ok(value) => value.parse::<usize>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{name} must be positive"),
            )
        })?,
        Err(env::VarError::NotPresent) => default,
        Err(error) => return Err(io::Error::other(error)),
    };
    if value == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} must be positive"),
        ));
    }
    Ok(value)
}

fn parse_source_revision(arguments: impl Iterator<Item = String>) -> Result<String, io::Error> {
    let arguments = arguments.collect::<Vec<_>>();
    let revision = arguments
        .windows(2)
        .find(|pair| pair[0] == "--source-revision")
        .map(|pair| pair[1].clone())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--source-revision is required")
        })?;
    if (revision.len() != 40 && revision.len() != 64)
        || !revision
            .bytes()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--source-revision must be a 40 or 64 character hexadecimal revision",
        ));
    }
    Ok(revision)
}
