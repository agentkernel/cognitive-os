//! `cognitive-conformance`: conformance runner of the CognitiveOS reference
//! implementation.
//!
//! M0 capability (this crate, per `docs/plan/DEVELOPMENT-PLAN.md`): enumerate
//! the fifteen test layers of `conformance/README.md` and every declarative
//! vector under `conformance/vectors/`, report each vector as `not-run`, and
//! emit a machine-readable report plus a sample profile manifest in which
//! every profile is `planned`.
//!
//! Execution and comparison capability is an M1 deliverable (Lane-CFR).
//! A schema-valid vector file is never reported as `pass`
//! (`conformance/README.md` "Running"; REQ-CONF-* in the registry).

use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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
/// dedicated slug today: their scenarios are filed under other slugs
/// (recorded in `docs/traceability/findings-ledger.md`, drift section).
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
}

/// One enumerated vector (parsed metadata only; inputs are not executed).
#[derive(Debug, Clone, Serialize)]
pub struct VectorEntry {
    pub id: String,
    pub file: String,
    pub layer_slug: String,
    pub profiles: Vec<String>,
    pub requirement_ids: Vec<String>,
    /// Always `not-run` in the M0 skeleton.
    pub result: &'static str,
}

#[derive(Debug, Serialize)]
pub struct LayerEntry {
    pub layer: u8,
    pub title: &'static str,
    pub vector_slugs: Vec<&'static str>,
    pub vector_count: usize,
    /// Always `not-run` in the M0 skeleton.
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct CrossCuttingEntry {
    pub slug: &'static str,
    pub vector_count: usize,
    pub status: &'static str,
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
}

#[derive(Debug, Serialize)]
pub struct ConformanceReport {
    pub report: &'static str,
    pub report_version: &'static str,
    pub runner: RunnerInfo,
    pub note: &'static str,
    pub layers: Vec<LayerEntry>,
    pub cross_cutting: Vec<CrossCuttingEntry>,
    pub vectors: Vec<VectorEntry>,
    pub summary: ReportSummary,
}

#[derive(Debug, Serialize)]
pub struct RunnerInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub capability: &'static str,
}

/// Enumerate and parse every vector under `<repo_root>/conformance/vectors`.
pub fn enumerate_vectors(repo_root: &Path) -> Result<Vec<VectorEntry>, RunnerError> {
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
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        vectors.push(VectorEntry {
            id: field_str("id")?,
            file: format!("conformance/vectors/{file_name}"),
            layer_slug: field_str("layer")?,
            profiles: field_str_array("profiles")?,
            requirement_ids: field_str_array("requirement_ids")?,
            result: "not-run",
        });
    }
    Ok(vectors)
}

/// Build the full `not-run` report from enumerated vectors.
pub fn build_report(vectors: Vec<VectorEntry>) -> ConformanceReport {
    let mut per_slug: BTreeMap<&str, usize> = BTreeMap::new();
    for vector in &vectors {
        *per_slug.entry(vector.layer_slug.as_str()).or_insert(0) += 1;
    }
    let layers = NUMBERED_LAYERS
        .iter()
        .map(|(number, title, slugs)| LayerEntry {
            layer: *number,
            title,
            vector_slugs: slugs.to_vec(),
            vector_count: slugs
                .iter()
                .map(|slug| per_slug.get(slug).copied().unwrap_or(0))
                .sum(),
            status: "not-run",
        })
        .collect();
    let cross_cutting = CROSS_CUTTING_SLUGS
        .iter()
        .map(|slug| CrossCuttingEntry {
            slug,
            vector_count: per_slug.get(slug).copied().unwrap_or(0),
            status: "not-run",
        })
        .collect();
    let summary = ReportSummary {
        total_vectors: vectors.len(),
        pass: 0,
        fail: 0,
        not_applicable: 0,
        documented_degradation: 0,
        not_run: vectors.len(),
    };
    ConformanceReport {
        report: "cognitiveos-conformance-report",
        report_version: "0.1.0",
        runner: RunnerInfo {
            name: "cognitive-conformance",
            version: env!("CARGO_PKG_VERSION"),
            capability: "enumerate-only",
        },
        note: "M0 skeleton: no vector was executed; every result is not-run. \
               Schema-valid parsing is never a pass (conformance/README.md).",
        layers,
        cross_cutting,
        vectors,
        summary,
    }
}

/// Render the human-readable summary of a report.
pub fn human_summary(report: &ConformanceReport) -> String {
    let mut out = String::new();
    out.push_str("CognitiveOS conformance runner (M0 skeleton, enumerate-only)\n");
    out.push_str(&format!(
        "Vectors enumerated: {} | pass 0 | fail 0 | not-applicable 0 | documented-degradation 0 | not-run {}\n",
        report.summary.total_vectors, report.summary.not_run
    ));
    out.push_str("Layers (conformance/README.md numbering):\n");
    for layer in &report.layers {
        let slugs = if layer.vector_slugs.is_empty() {
            "(no dedicated vector slug)".to_owned()
        } else {
            layer.vector_slugs.join(", ")
        };
        out.push_str(&format!(
            "  {:>2}. {:<58} vectors: {:>2}  status: not-run  [{}]\n",
            layer.layer, layer.title, layer.vector_count, slugs
        ));
    }
    for cross in &report.cross_cutting {
        out.push_str(&format!(
            "   +. cross-cutting `{}`{:<32} vectors: {:>2}  status: not-run\n",
            cross.slug, "", cross.vector_count
        ));
    }
    out.push_str(
        "No conformance claim is made by this output. Execution capability is an M1 deliverable.\n",
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
            "implementation": "cognitiveos-reference/0.0.1 (M0 skeleton; sample manifest, no conformance claim)",
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

    #[test]
    fn fifteen_layers_and_five_result_states() {
        assert_eq!(NUMBERED_LAYERS.len(), 15);
        assert_eq!(RESULT_STATES.len(), 5);
        assert!(RESULT_STATES.contains(&"not-run"));
    }

    #[test]
    fn report_counts_stay_not_run() {
        let report = build_report(vec![VectorEntry {
            id: "X-001".to_owned(),
            file: "conformance/vectors/x.json".to_owned(),
            layer_slug: "wire-schema".to_owned(),
            profiles: vec!["core_digital".to_owned()],
            requirement_ids: vec!["REQ-OBJ-001".to_owned()],
            result: "not-run",
        }]);
        assert_eq!(report.summary.total_vectors, 1);
        assert_eq!(report.summary.not_run, 1);
        assert_eq!(report.summary.pass, 0);
        let layer1 = &report.layers[0];
        assert_eq!(layer1.vector_count, 1);
    }
}
