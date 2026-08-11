//! Non-claim store-access stage timing for P9-T03/D03.
//!
//! Compares per-call `SqliteAuthorityStore::open` against a long-lived single
//! writer handle. Observations stay hypothesis-only: they never become Gate,
//! release, Profile, or Agent-benefit claims.

use std::time::Instant;

use cognitive_kernel::ProtocolStore;
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore, prepare_personal_databases};

const CLAIM_LEVEL: &str = "hypothesis-only";

/// Named stages recorded by the store-access comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreAccessStage {
    PerOpenRead,
    LongLivedRead,
}

impl StoreAccessStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PerOpenRead => "store_access_per_open_read",
            Self::LongLivedRead => "store_access_long_lived_read",
        }
    }
}

/// One raw stage sample. Tail percentiles are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAccessStageSample {
    pub stage: StoreAccessStage,
    pub duration_nanos: u128,
    pub iterations: u32,
}

/// Hypothesis-only before/after comparison for store access modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAccessStageObservation {
    pub claim_level: &'static str,
    pub stages: Vec<StoreAccessStageSample>,
}

impl StoreAccessStageObservation {
    pub fn claims_agent_benefit(&self) -> bool {
        false
    }
}

/// Errors while collecting the non-claim store-access comparison.
#[derive(Debug, thiserror::Error)]
pub enum StoreAccessCollectionError {
    #[error("{0}")]
    Failed(String),
}

/// Collect per-open vs long-lived store read timings against one Personal layout.
///
/// The collector opens real SQLite databases and issues `current_contract_epoch`
/// reads. It records raw stage nanos only and never fabricates percentiles or
/// Agent-benefit claims.
pub fn collect_store_access_stage_observation(
    layout: &PersonalDataLayout,
    iterations: u32,
) -> Result<StoreAccessStageObservation, StoreAccessCollectionError> {
    if iterations == 0 {
        return Err(StoreAccessCollectionError::Failed(
            "store-access comparison requires a positive iteration count".to_owned(),
        ));
    }
    prepare_personal_databases(layout).map_err(|error| {
        StoreAccessCollectionError::Failed(format!("prepare Personal databases: {error}"))
    })?;
    let database_path = layout.authority_database_path();

    let per_open_started = Instant::now();
    for _ in 0..iterations {
        let store = SqliteAuthorityStore::open(&database_path).map_err(|error| {
            StoreAccessCollectionError::Failed(format!("open authority store: {error}"))
        })?;
        let _ = store
            .current_contract_epoch("task://store-access/compare")
            .map_err(|error| {
                StoreAccessCollectionError::Failed(format!("per-open contract epoch read: {error}"))
            })?;
    }
    let per_open_nanos = per_open_started.elapsed().as_nanos();

    let long_lived = SqliteAuthorityStore::open(&database_path).map_err(|error| {
        StoreAccessCollectionError::Failed(format!("open long-lived authority store: {error}"))
    })?;
    let long_lived_started = Instant::now();
    for _ in 0..iterations {
        let _ = long_lived
            .current_contract_epoch("task://store-access/compare")
            .map_err(|error| {
                StoreAccessCollectionError::Failed(format!(
                    "long-lived contract epoch read: {error}"
                ))
            })?;
    }
    let long_lived_nanos = long_lived_started.elapsed().as_nanos();

    Ok(StoreAccessStageObservation {
        claim_level: CLAIM_LEVEL,
        stages: vec![
            StoreAccessStageSample {
                stage: StoreAccessStage::PerOpenRead,
                duration_nanos: per_open_nanos,
                iterations,
            },
            StoreAccessStageSample {
                stage: StoreAccessStage::LongLivedRead,
                duration_nanos: long_lived_nanos,
                iterations,
            },
        ],
    })
}

/// Reject authority-shaped or incomplete store-access observations.
pub fn validate_store_access_stage_observation(
    observation: &StoreAccessStageObservation,
) -> Result<(), String> {
    if observation.claim_level != CLAIM_LEVEL {
        return Err("store-access observations must remain hypothesis-only".to_owned());
    }
    if observation.claims_agent_benefit() {
        return Err("store-access observations must not claim Agent benefit".to_owned());
    }
    if observation.stages.len() != 2 {
        return Err("store-access observation is missing both access-mode stages".to_owned());
    }
    let per_open = observation
        .stages
        .iter()
        .find(|sample| sample.stage == StoreAccessStage::PerOpenRead)
        .ok_or_else(|| "store-access observation is missing per-open stage".to_owned())?;
    let long_lived = observation
        .stages
        .iter()
        .find(|sample| sample.stage == StoreAccessStage::LongLivedRead)
        .ok_or_else(|| "store-access observation is missing long-lived stage".to_owned())?;
    if per_open.iterations == 0 || long_lived.iterations == 0 {
        return Err("store-access stages must record a positive iteration count".to_owned());
    }
    if per_open.iterations != long_lived.iterations {
        return Err("store-access stages must share the same iteration denominator".to_owned());
    }
    if per_open.duration_nanos == 0 || long_lived.duration_nanos == 0 {
        return Err("store-access stages must record positive raw durations".to_owned());
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use cognitive_store::PersonalDataLayout;

    fn temporary_layout(label: &str) -> (std::path::PathBuf, PersonalDataLayout) {
        let root = std::env::temp_dir().join(format!(
            "cos-store-access-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        let layout = PersonalDataLayout::from_xdg_roots(&root, &root, &root, &root, &root);
        layout.ensure_directories().unwrap();
        (root, layout)
    }

    #[test]
    fn store_access_comparison_records_both_modes_without_agent_claim() {
        let (root, layout) = temporary_layout("compare");
        let observation = collect_store_access_stage_observation(&layout, 3).unwrap();
        validate_store_access_stage_observation(&observation).unwrap();
        assert!(!observation.claims_agent_benefit());
        assert_eq!(observation.claim_level, "hypothesis-only");
        assert_eq!(
            observation.stages[0].stage.as_str(),
            "store_access_per_open_read"
        );
        assert_eq!(
            observation.stages[1].stage.as_str(),
            "store_access_long_lived_read"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn zero_iteration_comparison_fails_closed() {
        let (root, layout) = temporary_layout("zero");
        let error = collect_store_access_stage_observation(&layout, 0).unwrap_err();
        assert!(error.to_string().contains("positive iteration count"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn forged_agent_claim_level_is_rejected() {
        let observation = StoreAccessStageObservation {
            claim_level: "agent-benefit",
            stages: vec![
                StoreAccessStageSample {
                    stage: StoreAccessStage::PerOpenRead,
                    duration_nanos: 10,
                    iterations: 1,
                },
                StoreAccessStageSample {
                    stage: StoreAccessStage::LongLivedRead,
                    duration_nanos: 5,
                    iterations: 1,
                },
            ],
        };
        let error = validate_store_access_stage_observation(&observation).unwrap_err();
        assert!(error.contains("hypothesis-only"));
    }
}
