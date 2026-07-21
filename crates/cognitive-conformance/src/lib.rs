//! `cognitive-conformance`: conformance runner of the CognitiveOS reference
//! implementation.
//!
//! M3 capability (Lane-CFR, per `docs/plan/DEVELOPMENT-PLAN.md`): enumerate
//! the fifteen test layers of `conformance/README.md` and every declarative
//! vector under `conformance/vectors/`, execute the statically decidable
//! subset against deterministic reference gates grounded in the registered
//! machine assets (schemas, registries, transition tables — see
//! `exec::classify`), execute the M2 kernel-backed vectors behaviorally
//! against `cognitive-kernel` over the `cognitive-store` SQLite WAL
//! authority store, execute the M3 governance/context vectors behaviorally
//! against the `authz`/`context`/`context_cache`/`capability` surface, and
//! report every vector in one of the five states of
//! `docs/standards/conformance-evidence.md` section 2. Vectors whose
//! expectations require runtime behavior of later milestones stay `not-run`
//! with a recorded reason.
//!
//! A schema-valid vector file is never reported as `pass`
//! (`conformance/README.md` "Running"; REQ-CONF-* in the registry). The
//! runner self-check (`exec::self_check`) proves a deliberately wrong,
//! schema-valid implementation is failed.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod exec;
pub use exec::{
    ExecutionMode, ExecutionPlan, ImplementationKind, SelfCheckReport, VectorOutcome, classify,
    execute_all, self_check, self_check_passed,
};

/// Result states used in runner reports. The four executed-outcome states
/// come from `conformance/README.md`; `not-run` is the report-level state for
/// vectors that were enumerated but never executed
/// (see `docs/standards/conformance-evidence.md`).
pub const RESULT_STATES: [&str; 5] = [
    "pass",
    "fail",
    "not-applicable",
    "documented-degradation",
    "not-run",
];

/// The fifteen numbered test layers of `conformance/README.md`, with the
/// vector `layer` slugs currently mapped to each. Layers 7 and 8 have no
/// dedicated slug: their scenarios are cross-slice-hosted under other slugs
/// (`CROSS_SLICE_HOSTED`, D-004 documented disposition).
pub const NUMBERED_LAYERS: [(u8, &str, &[&str]); 15] = [
    (1, "Wire/schema and version negotiation", &["wire-schema"]),
    (
        2,
        "State-machine, CAS and conflict handling",
        &["state-machine"],
    ),
    (
        3,
        "Effect, idempotency and crash recovery",
        &["effect-recovery"],
    ),
    (
        4,
        "Security negatives and information-flow isolation",
        &["security-negative"],
    ),
    (
        5,
        "Context resolution and semantic boundaries",
        &["context-semantic"],
    ),
    (
        6,
        "Harness, Loop, progress and Verification",
        &["harness-loop"],
    ),
    (7, "Knowledge compilation and invalidation", &[]),
    (8, "Performance and reproducibility contracts", &[]),
    (
        9,
        "Privileged management session and deterministic fallback",
        &["management-shell"],
    ),
    (
        10,
        "Agent installation, adapters and sandbox interception",
        &["agent-installation"],
    ),
    (
        11,
        "Governed memory admission and lifecycle",
        &["governed-memory"],
    ),
    (
        12,
        "Cognitive discovery, delta and stagnation",
        &["cognitive-discovery"],
    ),
    (
        13,
        "Operation catalog lifecycle, match and binding",
        &["operation-catalog"],
    ),
    (
        14,
        "Semantic mediation and CRB hard-bound enforcement",
        &["semantic-mediation"],
    ),
    (
        15,
        "User intent, Agent Shell and acceptance semantics",
        &["shell-intent-lifecycle"],
    ),
];

/// D-004 documented disposition (findings ledger): layers 7 and 8 keep no
/// dedicated `layer` slug because their scenarios are genuinely
/// cross-cutting; this mapping pins which vectors host them so the report
/// shows the coverage instead of a misleading zero. Kept in sync with
/// `conformance/README.md` ("Fifteen test layers" note).
pub const CROSS_SLICE_HOSTED: [(u8, &[(&str, &str)]); 2] = [
    (
        7,
        &[
            ("KNOW-INVALIDATION-001", "context-semantic"),
            ("KNOW-POISON-001", "security-negative"),
            ("KNOW-MAINTENANCE-001", "harness-loop"),
        ],
    ),
    (8, &[("PERF-REPORT-CONTRACT-001", "wire-schema")]),
];

/// Cross-cutting vector slugs that map to requirement traceability rather
/// than to a single numbered layer.
pub const CROSS_CUTTING_SLUGS: [&str; 1] = ["contract-traceability"];

/// All thirteen profile keys required by
/// `specs/schemas/profile-manifest.schema.json`.
pub const PROFILE_KEYS: [&str; 13] = [
    "core_digital",
    "distributed",
    "embodied_safety",
    "heterogeneous_cim",
    "controlled_learning",
    "context_virtualization",
    "harnessed_autonomous_execution",
    "intelligent_management_shell",
    "agent_compatibility",
    "governed_memory",
    "cognitive_discovery",
    "operation_catalog",
    "semantic_mediation",
];

#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("i/o error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid vector {path}: {reason}")]
    InvalidVector { path: PathBuf, reason: String },
    #[error("invalid registry {path}: {reason}")]
    InvalidRegistry { path: PathBuf, reason: String },
    #[error("canonicalization failed for {what}: {source}")]
    Canonical {
        what: String,
        source: cognitive_contracts::canonical::CanonicalError,
    },
    #[error("bundle manifest construction failed for {what}: {reason}")]
    Bundle { what: String, reason: String },
    #[error(transparent)]
    Exec(#[from] exec::ExecError),
}

/// One fully loaded vector: metadata plus the declarative `input` and
/// `expected` documents consumed by the execution engine.
#[derive(Debug, Clone)]
pub struct LoadedVector {
    pub id: String,
    pub file: String,
    pub layer_slug: String,
    pub profiles: Vec<String>,
    pub requirement_ids: Vec<String>,
    pub input: serde_json::Value,
    pub expected: serde_json::Value,
}

/// Per-state result counts (report and per-layer aggregation).
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct StateCounts {
    pub pass: usize,
    pub fail: usize,
    #[serde(rename = "not-applicable")]
    pub not_applicable: usize,
    #[serde(rename = "documented-degradation")]
    pub documented_degradation: usize,
    #[serde(rename = "not-run")]
    pub not_run: usize,
}

impl StateCounts {
    fn add(&mut self, result: &str) {
        match result {
            "pass" => self.pass += 1,
            "fail" => self.fail += 1,
            "not-applicable" => self.not_applicable += 1,
            "documented-degradation" => self.documented_degradation += 1,
            _ => self.not_run += 1,
        }
    }

    pub fn total(&self) -> usize {
        self.pass + self.fail + self.not_applicable + self.documented_degradation + self.not_run
    }
}

#[derive(Debug, Serialize)]
pub struct LayerEntry {
    pub layer: u8,
    pub title: &'static str,
    pub vector_slugs: Vec<&'static str>,
    pub vector_count: usize,
    pub results: StateCounts,
    /// D-004: vectors hosting this layer's scenarios under other slugs
    /// (layers 7/8 only; counted in their primary slug's layer, listed here
    /// for coverage visibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_slice_hosted: Option<Vec<CrossSliceRef>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossSliceRef {
    pub vector_id: &'static str,
    pub primary_slug: &'static str,
    pub result: String,
}

#[derive(Debug, Serialize)]
pub struct CrossCuttingEntry {
    pub slug: &'static str,
    pub vector_count: usize,
    pub results: StateCounts,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub total_vectors: usize,
    pub pass: usize,
    pub fail: usize,
    #[serde(rename = "not-applicable")]
    pub not_applicable: usize,
    #[serde(rename = "documented-degradation")]
    pub documented_degradation: usize,
    #[serde(rename = "not-run")]
    pub not_run: usize,
    /// Number of executed vectors (pass + fail): each carries an execution
    /// record with grounding and evidence.
    pub executed: usize,
}

#[derive(Debug, Serialize)]
pub struct ConformanceReport {
    pub report: &'static str,
    pub report_version: &'static str,
    pub runner: RunnerInfo,
    pub note: &'static str,
    pub layers: Vec<LayerEntry>,
    pub cross_cutting: Vec<CrossCuttingEntry>,
    pub vectors: Vec<VectorOutcome>,
    pub summary: ReportSummary,
}

#[derive(Debug, Serialize)]
pub struct RunnerInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub capability: &'static str,
}

/// Enumerate and parse every vector under `<repo_root>/conformance/vectors`.
pub fn enumerate_vectors(repo_root: &Path) -> Result<Vec<LoadedVector>, RunnerError> {
    let dir = repo_root.join("conformance").join("vectors");
    let entries = fs::read_dir(&dir).map_err(|source| RunnerError::Io {
        path: dir.clone(),
        source,
    })?;
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| RunnerError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    files.sort();

    let mut vectors = Vec::with_capacity(files.len());
    for path in files {
        let raw = fs::read_to_string(&path).map_err(|source| RunnerError::Io {
            path: path.clone(),
            source,
        })?;
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|err| RunnerError::InvalidVector {
                path: path.clone(),
                reason: err.to_string(),
            })?;
        let field_str = |name: &str| -> Result<String, RunnerError> {
            value
                .get(name)
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .ok_or_else(|| RunnerError::InvalidVector {
                    path: path.clone(),
                    reason: format!("missing string field `{name}`"),
                })
        };
        let field_str_array = |name: &str| -> Result<Vec<String>, RunnerError> {
            let items = value.get(name).and_then(|v| v.as_array()).ok_or_else(|| {
                RunnerError::InvalidVector {
                    path: path.clone(),
                    reason: format!("missing array field `{name}`"),
                }
            })?;
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| RunnerError::InvalidVector {
                            path: path.clone(),
                            reason: format!("non-string entry in `{name}`"),
                        })
                })
                .collect()
        };
        let expected =
            value
                .get("expected")
                .cloned()
                .ok_or_else(|| RunnerError::InvalidVector {
                    path: path.clone(),
                    reason: "missing `expected` document".to_owned(),
                })?;
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        vectors.push(LoadedVector {
            id: field_str("id")?,
            file: format!("conformance/vectors/{file_name}"),
            layer_slug: field_str("layer")?,
            profiles: field_str_array("profiles")?,
            requirement_ids: field_str_array("requirement_ids")?,
            input: value
                .get("input")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
            expected,
        });
    }
    Ok(vectors)
}

/// Build the full five-state report from executed vector outcomes.
pub fn build_report(outcomes: Vec<VectorOutcome>) -> ConformanceReport {
    let mut per_slug: BTreeMap<&str, StateCounts> = BTreeMap::new();
    let mut by_id: BTreeMap<&str, &str> = BTreeMap::new();
    let mut summary_counts = StateCounts::default();
    for outcome in &outcomes {
        per_slug
            .entry(outcome.layer_slug.as_str())
            .or_default()
            .add(outcome.result);
        by_id.insert(outcome.id.as_str(), outcome.result);
        summary_counts.add(outcome.result);
    }

    let hosted: BTreeMap<u8, &[(&str, &str)]> = CROSS_SLICE_HOSTED.iter().copied().collect();
    let layers = NUMBERED_LAYERS
        .iter()
        .map(|(number, title, slugs)| {
            let mut results = StateCounts::default();
            let mut count = 0usize;
            for slug in *slugs {
                if let Some(counts) = per_slug.get(slug) {
                    count += counts.total();
                    results.pass += counts.pass;
                    results.fail += counts.fail;
                    results.not_applicable += counts.not_applicable;
                    results.documented_degradation += counts.documented_degradation;
                    results.not_run += counts.not_run;
                }
            }
            let cross_slice_hosted = hosted.get(number).map(|pairs| {
                pairs
                    .iter()
                    .map(|(vector_id, primary_slug)| CrossSliceRef {
                        vector_id,
                        primary_slug,
                        result: by_id
                            .get(vector_id)
                            .copied()
                            .unwrap_or("not-run")
                            .to_owned(),
                    })
                    .collect()
            });
            LayerEntry {
                layer: *number,
                title,
                vector_slugs: slugs.to_vec(),
                vector_count: count,
                results,
                cross_slice_hosted,
            }
        })
        .collect();
    let cross_cutting = CROSS_CUTTING_SLUGS
        .iter()
        .map(|slug| {
            let counts = per_slug.get(slug).cloned().unwrap_or_default();
            CrossCuttingEntry {
                slug,
                vector_count: counts.total(),
                results: counts,
            }
        })
        .collect();
    let summary = ReportSummary {
        total_vectors: outcomes.len(),
        pass: summary_counts.pass,
        fail: summary_counts.fail,
        not_applicable: summary_counts.not_applicable,
        documented_degradation: summary_counts.documented_degradation,
        not_run: summary_counts.not_run,
        executed: summary_counts.pass + summary_counts.fail,
    };
    ConformanceReport {
        report: "cognitiveos-conformance-report",
        report_version: "0.2.0",
        runner: RunnerInfo {
            name: "cognitive-conformance",
            version: env!("CARGO_PKG_VERSION"),
            capability: "static-contract + kernel-behavioral execution (M2/M3/M4/M5)",
        },
        note: "M4 runner: statically decidable vectors are executed against deterministic \
               reference gates grounded in registered machine assets; the M2 kernel-backed \
               vectors run behaviorally against cognitive-kernel over the SQLite WAL authority \
               store, the M3 governance/context vectors against the \
               authz/context/context_cache/capability surface, and the M4 effect/recovery \
               vectors through the public fault-injection framework (CrashHarness + \
               ScriptedExecutor). Remaining behavioral layers stay not-run with recorded \
               reasons. A pass is scoped to its execution mode and is never a Profile claim; \
               schema-valid parsing alone is never a pass (conformance/README.md; \
               docs/standards/conformance-evidence.md).",
        layers,
        cross_cutting,
        vectors: outcomes,
        summary,
    }
}

/// Render the human-readable summary of a report.
pub fn human_summary(report: &ConformanceReport) -> String {
    let mut out = String::new();
    out.push_str(
        "CognitiveOS conformance runner (M5: static-contract + kernel-behavioral execution)\n",
    );
    out.push_str(&format!(
        "Vectors enumerated: {} | pass {} | fail {} | not-applicable {} | documented-degradation {} | not-run {}\n",
        report.summary.total_vectors,
        report.summary.pass,
        report.summary.fail,
        report.summary.not_applicable,
        report.summary.documented_degradation,
        report.summary.not_run
    ));
    out.push_str("Layers (conformance/README.md numbering):\n");
    for layer in &report.layers {
        let slugs = if layer.vector_slugs.is_empty() {
            "(cross-slice, D-004)".to_owned()
        } else {
            layer.vector_slugs.join(", ")
        };
        let mut line = format!(
            "  {:>2}. {:<58} vectors: {:>2}  pass {:>2}  not-run {:>2}  [{}]",
            layer.layer,
            layer.title,
            layer.vector_count,
            layer.results.pass,
            layer.results.not_run,
            slugs
        );
        if layer.results.fail > 0 {
            line.push_str(&format!("  FAIL {}", layer.results.fail));
        }
        if let Some(hosted) = &layer.cross_slice_hosted {
            let rendered: Vec<String> = hosted
                .iter()
                .map(|h| format!("{} ({}, {})", h.vector_id, h.primary_slug, h.result))
                .collect();
            line.push_str(&format!(
                "\n      hosted cross-slice: {}",
                rendered.join("; ")
            ));
        }
        line.push('\n');
        out.push_str(&line);
    }
    for cross in &report.cross_cutting {
        out.push_str(&format!(
            "   +. cross-cutting `{}`  vectors: {:>2}  pass {:>2}  fail {:>2}  not-run {:>2}\n",
            cross.slug,
            cross.vector_count,
            cross.results.pass,
            cross.results.fail,
            cross.results.not_run
        ));
    }
    out.push_str(
        "A pass is scoped to its execution mode (static-contract gates grounded in registered \
         machine assets, or M2/M3 behavioral execution against the real kernel surfaces); it \
         is never a Profile conformance claim. Remaining behavioral layers stay not-run with \
         recorded reasons until their owning milestone.\n",
    );
    out
}

/// Compute the registered set/bundle digests pinned by the sample profile
/// manifest (`docs/standards/canonical-encoding-and-digest.md` section 13
/// procedure implemented by `cognitive_contracts::bundle`; recipe documented
/// in `docs/standards/conformance-evidence.md` section 6):
///
/// - `schema_bundle_digest`: manifest over every `specs/schemas/*.json`
///   asset (id = file name == `$id`, suite version, schema media type,
///   per-asset canonical content digest), domain `schema-bundle/0.1`;
/// - `requirement_set_digest`: manifest over the specification set beyond
///   schemas — the three registries (canonical JSON projection of the parsed
///   YAML) and the five transition tables (id = repo-relative path), domain
///   `spec-set/0.1`.
///
/// This replaces the provisional M0 recipe (bare `{id, content_digest}`
/// list; requirements.yaml hashed alone).
pub fn registered_digests(repo_root: &Path) -> Result<(String, String), RunnerError> {
    use cognitive_contracts::bundle::{
        self, BundleAsset, MEDIA_TYPE_JSON, MEDIA_TYPE_SCHEMA_JSON, MEDIA_TYPE_YAML,
        SCHEMA_BUNDLE_DOMAIN, SPEC_SET_DOMAIN, SPEC_SUITE_VERSION,
    };
    use cognitive_contracts::canonical;

    let read = |path: &Path| -> Result<String, RunnerError> {
        fs::read_to_string(path).map_err(|source| RunnerError::Io {
            path: path.to_path_buf(),
            source,
        })
    };
    let bundle_err = |what: &str| {
        let what = what.to_owned();
        move |err: bundle::BundleError| RunnerError::Bundle {
            what,
            reason: err.to_string(),
        }
    };

    // Schema bundle assets: every registered schema, id = file name ($id).
    let schema_dir = repo_root.join("specs").join("schemas");
    let entries = fs::read_dir(&schema_dir).map_err(|source| RunnerError::Io {
        path: schema_dir.clone(),
        source,
    })?;
    let mut schema_files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| RunnerError::Io {
            path: schema_dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            schema_files.push(path);
        }
    }
    schema_files.sort();
    let mut schema_assets = Vec::with_capacity(schema_files.len());
    for path in &schema_files {
        let value =
            canonical::parse_strict(&read(path)?).map_err(|source| RunnerError::Canonical {
                what: path.display().to_string(),
                source,
            })?;
        let bytes =
            canonical::canonical_bytes(&value).map_err(|source| RunnerError::Canonical {
                what: path.display().to_string(),
                source,
            })?;
        let content_digest = canonical::digest(&bytes, SCHEMA_BUNDLE_DOMAIN).map_err(|source| {
            RunnerError::Canonical {
                what: path.display().to_string(),
                source,
            }
        })?;
        schema_assets.push(BundleAsset {
            id: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            version: SPEC_SUITE_VERSION.to_owned(),
            media_type: MEDIA_TYPE_SCHEMA_JSON.to_owned(),
            content_digest,
        });
    }
    let schema_bundle_digest = bundle::manifest_digest(&schema_assets, SCHEMA_BUNDLE_DOMAIN)
        .map_err(bundle_err("schema bundle manifest"))?;

    // Specification-set assets: registries (YAML -> canonical JSON
    // projection) + transition tables, ids = repo-relative paths.
    let mut set_assets: Vec<BundleAsset> = Vec::new();
    for registry in ["requirements.yaml", "errors.yaml", "state-domains.yaml"] {
        let path = repo_root.join("specs").join("registry").join(registry);
        let value: serde_json::Value =
            serde_yaml::from_str(&read(&path)?).map_err(|err| RunnerError::InvalidRegistry {
                path: path.clone(),
                reason: err.to_string(),
            })?;
        let content_digest = bundle::asset_content_digest(&value, SPEC_SET_DOMAIN)
            .map_err(bundle_err(&path.display().to_string()))?;
        set_assets.push(BundleAsset {
            id: format!("specs/registry/{registry}"),
            version: SPEC_SUITE_VERSION.to_owned(),
            media_type: MEDIA_TYPE_YAML.to_owned(),
            content_digest,
        });
    }
    for domain in ["agent-execution", "effect", "loop", "task", "verification"] {
        let path = repo_root
            .join("specs")
            .join("transitions")
            .join(format!("{domain}.transitions.json"));
        let value =
            canonical::parse_strict(&read(&path)?).map_err(|source| RunnerError::Canonical {
                what: path.display().to_string(),
                source,
            })?;
        let bytes =
            canonical::canonical_bytes(&value).map_err(|source| RunnerError::Canonical {
                what: path.display().to_string(),
                source,
            })?;
        let content_digest = canonical::digest(&bytes, SPEC_SET_DOMAIN).map_err(|source| {
            RunnerError::Canonical {
                what: path.display().to_string(),
                source,
            }
        })?;
        set_assets.push(BundleAsset {
            id: format!("specs/transitions/{domain}.transitions.json"),
            version: SPEC_SUITE_VERSION.to_owned(),
            media_type: MEDIA_TYPE_JSON.to_owned(),
            content_digest,
        });
    }
    let requirement_set_digest = bundle::manifest_digest(&set_assets, SPEC_SET_DOMAIN)
        .map_err(bundle_err("specification set manifest"))?;

    Ok((requirement_set_digest, schema_bundle_digest))
}

/// Build the sample profile manifest: every profile `planned`, zero test
/// runs, no conformance claim. Validated against
/// `specs/schemas/profile-manifest.schema.json` by the repo tools in CI.
/// Static-contract vector passes do not move any profile off `planned`:
/// `implemented` requires behavioral evidence for every applicable MUST
/// (`docs/standards/conformance-evidence.md` section 5).
pub fn sample_profile_manifest(
    repo_root: &Path,
    encoding_digest: &str,
) -> Result<serde_json::Value, RunnerError> {
    let (requirement_set_digest, schema_bundle_digest) = registered_digests(repo_root)?;
    let mut profiles = serde_json::Map::new();
    for key in PROFILE_KEYS {
        profiles.insert(
            key.to_owned(),
            serde_json::Value::String("planned".to_owned()),
        );
    }
    Ok(serde_json::json!({
        "cognitiveos_conformance": {
            "spec": {
                "id": "cognitiveos",
                "version": "0.1",
                "requirement_set_digest": requirement_set_digest,
                "schema_bundle_digest": schema_bundle_digest
            },
            "implementation": "cognitiveos-reference/0.0.1 (M1 static-contract runner; sample manifest, no conformance claim)",
            "profiles": profiles,
            "encodings": {
                "cognitiveos.canonical-json/0.1": {
                    "canonicalization": "cognitiveos.canonical-json/0.1",
                    "digest": encoding_digest
                }
            },
            "guarantees": {
                "event_delivery": "at_least_once",
                "state_conflict": "compare_and_swap",
                "effect_recovery": "reconcile_or_quarantine"
            },
            "test_runs": [],
            "known_degradations": [],
            "evidence_refs": ["./conformance-report.json"],
            "performance_reports": [],
            "agent_compatibility": {
                "max_profile": null,
                "max_verified_risk": null,
                "feature_matrix": {}
            },
            "semantic_service": {
                "level": "unsupported",
                "manifest_ref": null
            }
        }
    }))
}

/// Digest of the committed golden fixture file, used as the operational
/// identifier of the encoding profile in the sample manifest.
pub fn golden_fixture_digest(repo_root: &Path) -> Result<String, RunnerError> {
    use cognitive_contracts::canonical;
    let path = repo_root
        .join("tests")
        .join("golden")
        .join("canonical-json-fixtures.json");
    let raw = fs::read_to_string(&path).map_err(|source| RunnerError::Io {
        path: path.clone(),
        source,
    })?;
    let value = canonical::parse_strict(&raw).map_err(|source| RunnerError::Canonical {
        what: path.display().to_string(),
        source,
    })?;
    let bytes = canonical::canonical_bytes(&value).map_err(|source| RunnerError::Canonical {
        what: path.display().to_string(),
        source,
    })?;
    canonical::digest(&bytes, "conformance-fixture/0.1").map_err(|source| RunnerError::Canonical {
        what: path.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn outcome(id: &str, slug: &str, result: &'static str) -> VectorOutcome {
        VectorOutcome {
            id: id.to_owned(),
            file: format!("conformance/vectors/{}.json", id.to_lowercase()),
            layer_slug: slug.to_owned(),
            profiles: vec!["core_digital".to_owned()],
            requirement_ids: vec!["REQ-OBJ-001".to_owned()],
            result,
            execution: None,
            not_run_reason: (result == "not-run").then(|| "test".to_owned()),
            partial_contract_assertions: None,
        }
    }

    #[test]
    fn fifteen_layers_and_five_result_states() {
        assert_eq!(NUMBERED_LAYERS.len(), 15);
        assert_eq!(RESULT_STATES.len(), 5);
        assert!(RESULT_STATES.contains(&"not-run"));
    }

    #[test]
    fn report_counts_five_states_and_hosts_cross_slice() {
        let report = build_report(vec![
            outcome("X-001", "wire-schema", "pass"),
            outcome("X-002", "wire-schema", "not-run"),
            outcome("PERF-REPORT-CONTRACT-001", "wire-schema", "pass"),
        ]);
        assert_eq!(report.summary.total_vectors, 3);
        assert_eq!(report.summary.pass, 2);
        assert_eq!(report.summary.not_run, 1);
        assert_eq!(report.summary.executed, 2);
        let layer1 = &report.layers[0];
        assert_eq!(layer1.vector_count, 3);
        assert_eq!(layer1.results.pass, 2);
        // D-004: layer 8 shows its cross-slice-hosted performance vector.
        let layer8 = &report.layers[7];
        assert_eq!(layer8.vector_count, 0);
        let hosted = layer8
            .cross_slice_hosted
            .as_ref()
            .map(|h| {
                h.iter()
                    .map(|r| (r.vector_id, r.result.as_str()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert_eq!(hosted, vec![("PERF-REPORT-CONTRACT-001", "pass")]);
    }

    #[test]
    fn loaded_vector_expected_is_mandatory() {
        // enumerate_vectors on the real corpus is covered by integration
        // tests; here we pin the report math only.
        let report = build_report(vec![outcome("Y-001", "state-machine", "fail")]);
        assert_eq!(report.summary.fail, 1);
        assert_eq!(report.summary.executed, 1);
        assert_eq!(json!(report.summary.total_vectors), json!(1));
    }
}
