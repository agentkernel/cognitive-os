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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn overhead_report_requires_ungoverned_baseline_and_forbids_benefit() {
        let sample = GovernanceOverheadSample {
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
        };
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
}
