//! The five execution lifecycle transition tables, consumed from
//! `specs/transitions/*.transitions.json` (REQ-STATE-001..003;
//! `docs/standards/state-and-transition-contract.md`).
//!
//! Single source of truth: the registered JSON tables are embedded verbatim
//! at compile time with `include_str!` and parsed once per process. No
//! state, edge, reason, guard, or terminal declaration is ever hand-copied
//! into Rust constants (`.cursor/rules/10-rust-kernel.mdc`). Consumers pin
//! each table by its canonical digest, computed exactly like the
//! specification-set asset digests in the conformance runner: canonical
//! bytes (RFC 8785) under digest domain `spec-set/0.1`.
//!
//! This module is pure lookup and validation. Guard evaluation, evidence
//! checking, CAS, and commits happen in the deterministic kernel gate.

use crate::ids::StateName;
use cognitive_contracts::bundle::SPEC_SET_DOMAIN;
use cognitive_contracts::canonical;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::OnceLock;

/// The five registered execution lifecycle state domains
/// (`specs/registry/state-domains.yaml`). The registry keeps the domain set
/// open (REQ-STATE-001); these five are the v0.1 execution core and are
/// never merged into one machine (architecture invariant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LifecycleDomain {
    /// `agent-execution.transitions.json` (execution-authority).
    AgentExecution,
    /// `effect.transitions.json` (effect-authority).
    Effect,
    /// `loop.transitions.json` (execution-authority).
    Loop,
    /// `task.transitions.json` (task-acceptance-authority).
    Task,
    /// `verification.transitions.json` (verification-authority).
    Verification,
}

impl LifecycleDomain {
    /// All five registered lifecycle domains, sorted by domain name.
    pub const ALL: [LifecycleDomain; 5] = [
        LifecycleDomain::AgentExecution,
        LifecycleDomain::Effect,
        LifecycleDomain::Loop,
        LifecycleDomain::Task,
        LifecycleDomain::Verification,
    ];

    /// Registered domain name (`state-domains.yaml` `domain` field).
    pub fn as_str(self) -> &'static str {
        match self {
            LifecycleDomain::AgentExecution => "agent-execution",
            LifecycleDomain::Effect => "effect",
            LifecycleDomain::Loop => "loop",
            LifecycleDomain::Task => "task",
            LifecycleDomain::Verification => "verification",
        }
    }

    /// Parse a registered lifecycle domain name.
    pub fn parse(value: &str) -> Result<Self, crate::error::DomainError> {
        match value {
            "agent-execution" => Ok(LifecycleDomain::AgentExecution),
            "effect" => Ok(LifecycleDomain::Effect),
            "loop" => Ok(LifecycleDomain::Loop),
            "task" => Ok(LifecycleDomain::Task),
            "verification" => Ok(LifecycleDomain::Verification),
            other => Err(crate::error::DomainError::UnknownLifecycleDomain(
                other.to_owned(),
            )),
        }
    }
}

impl fmt::Display for LifecycleDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One transition row of a registered table.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionEdge {
    /// Source state.
    pub from: String,
    /// Target state.
    pub to: String,
    /// Structured reasons that may select this row.
    pub reason_codes: Vec<String>,
    /// Guards that must all hold, established deterministically.
    pub guards: Vec<String>,
    /// Evidence items that must accompany an accepted transition.
    pub required_evidence: Vec<String>,
    /// Transition-specific metadata constraining deterministic
    /// interpretation (for example `reconciliation_result`).
    #[serde(default)]
    pub metadata: Option<Value>,
}

/// A registered transition table (`state-transition-table.schema.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionTable {
    /// Registered domain name.
    pub domain: String,
    /// Table version; pinned together with the digest.
    pub version: String,
    /// Registration status (`Draft` for v0.1).
    pub status: String,
    /// State a newly admitted object starts in.
    pub initial_state: String,
    /// Closed state set of this table version.
    pub states: Vec<String>,
    /// States with no legal outgoing transition in this table version.
    pub terminal_states: Vec<String>,
    /// Transition rows.
    pub transitions: Vec<TransitionEdge>,
}

/// A parsed, structurally validated table plus its pinned identity.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedTable {
    /// The typed table.
    pub table: TransitionTable,
    /// Canonical digest of the registered JSON asset: canonical bytes under
    /// domain `spec-set/0.1`, byte-identical to the per-asset digest the
    /// conformance runner records for `specs/transitions/*.json`.
    pub digest: String,
}

/// Failure to parse or validate an embedded registered table asset. These
/// indicate a corrupted specification asset or a broken build, never a
/// caller mistake; the kernel treats them as a fail-closed recovery barrier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TableAssetError {
    /// Asset is not strict I-JSON or violates the table shape.
    #[error("table-parse: {domain}: {reason}")]
    Parse {
        /// Domain whose asset failed.
        domain: &'static str,
        /// Parser message.
        reason: String,
    },
    /// Canonicalization or digest computation failed.
    #[error("table-digest: {domain}: {reason}")]
    Digest {
        /// Domain whose asset failed.
        domain: &'static str,
        /// Digest failure message.
        reason: String,
    },
    /// Structural invariant of the table is violated.
    #[error("table-invalid: {domain}: {reason}")]
    Invalid {
        /// Domain whose asset failed.
        domain: &'static str,
        /// Violated invariant.
        reason: String,
    },
}

/// Outcome of looking up one requested edge in a table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeLookupError {
    /// `from` is not a state of this table version.
    UnknownFromState,
    /// `to` is not a state of this table version.
    UnknownToState,
    /// `from` is declared terminal: no legal outgoing transition exists
    /// (`state-and-transition-contract.md` section 4).
    TerminalFrom,
    /// No transition row matches the `(from, to)` state pair.
    NoMatchingEdge,
    /// Rows match the state pair, but none allows the requested reason.
    ReasonNotAllowed,
}

impl LoadedTable {
    /// True when `state` is in this table's closed state set.
    pub fn has_state(&self, state: &StateName) -> bool {
        self.table.states.iter().any(|s| s == state.as_str())
    }

    /// True when `state` is declared terminal in this table version.
    pub fn is_terminal(&self, state: &StateName) -> bool {
        self.table
            .terminal_states
            .iter()
            .any(|s| s == state.as_str())
    }

    /// Safe available exits from `from`: the sorted, distinct target states
    /// of all legal outgoing rows. Returned with every rejection
    /// (`state-and-transition-contract.md` section 3).
    pub fn legal_exits(&self, from: &StateName) -> Vec<&str> {
        let mut exits: Vec<&str> = self
            .table
            .transitions
            .iter()
            .filter(|edge| edge.from == from.as_str())
            .map(|edge| edge.to.as_str())
            .collect();
        exits.sort_unstable();
        exits.dedup();
        exits
    }

    /// Deterministic row lookup: exactly the row whose `(from, to)` matches
    /// and whose `reason_codes` contains `reason`.
    pub fn find_edge(
        &self,
        from: &StateName,
        to: &StateName,
        reason: &str,
    ) -> Result<&TransitionEdge, EdgeLookupError> {
        if !self.has_state(from) {
            return Err(EdgeLookupError::UnknownFromState);
        }
        if !self.has_state(to) {
            return Err(EdgeLookupError::UnknownToState);
        }
        if self.is_terminal(from) {
            return Err(EdgeLookupError::TerminalFrom);
        }
        let pair_rows: Vec<&TransitionEdge> = self
            .table
            .transitions
            .iter()
            .filter(|edge| edge.from == from.as_str() && edge.to == to.as_str())
            .collect();
        if pair_rows.is_empty() {
            return Err(EdgeLookupError::NoMatchingEdge);
        }
        pair_rows
            .into_iter()
            .find(|edge| edge.reason_codes.iter().any(|code| code == reason))
            .ok_or(EdgeLookupError::ReasonNotAllowed)
    }
}

struct EmbeddedAsset {
    domain: LifecycleDomain,
    text: &'static str,
}

/// Registered table assets embedded at compile time from
/// `specs/transitions/`. The paths are the single source; editing a table
/// recompiles this crate.
const EMBEDDED: [EmbeddedAsset; 5] = [
    EmbeddedAsset {
        domain: LifecycleDomain::AgentExecution,
        text: include_str!("../../../specs/transitions/agent-execution.transitions.json"),
    },
    EmbeddedAsset {
        domain: LifecycleDomain::Effect,
        text: include_str!("../../../specs/transitions/effect.transitions.json"),
    },
    EmbeddedAsset {
        domain: LifecycleDomain::Loop,
        text: include_str!("../../../specs/transitions/loop.transitions.json"),
    },
    EmbeddedAsset {
        domain: LifecycleDomain::Task,
        text: include_str!("../../../specs/transitions/task.transitions.json"),
    },
    EmbeddedAsset {
        domain: LifecycleDomain::Verification,
        text: include_str!("../../../specs/transitions/verification.transitions.json"),
    },
];

/// Parse, digest and structurally validate one table asset text. Public for
/// negative tests; production consumers use [`table`].
pub fn parse_table(domain_name: &'static str, text: &str) -> Result<LoadedTable, TableAssetError> {
    let strict = canonical::parse_strict(text).map_err(|err| TableAssetError::Parse {
        domain: domain_name,
        reason: err.to_string(),
    })?;
    let canonical_bytes =
        canonical::canonical_bytes(&strict).map_err(|err| TableAssetError::Digest {
            domain: domain_name,
            reason: err.to_string(),
        })?;
    let digest = canonical::digest(&canonical_bytes, SPEC_SET_DOMAIN).map_err(|err| {
        TableAssetError::Digest {
            domain: domain_name,
            reason: err.to_string(),
        }
    })?;
    let table: TransitionTable =
        serde_json::from_str(text).map_err(|err| TableAssetError::Parse {
            domain: domain_name,
            reason: err.to_string(),
        })?;
    validate_table(domain_name, &table)?;
    Ok(LoadedTable { table, digest })
}

fn validate_table(
    domain_name: &'static str,
    table: &TransitionTable,
) -> Result<(), TableAssetError> {
    let invalid = |reason: String| TableAssetError::Invalid {
        domain: domain_name,
        reason,
    };
    if table.domain != domain_name {
        return Err(invalid(format!(
            "asset domain {} does not match {}",
            table.domain, domain_name
        )));
    }
    let states: BTreeSet<&str> = table.states.iter().map(String::as_str).collect();
    if states.len() != table.states.len() {
        return Err(invalid("duplicate state names".to_owned()));
    }
    if !states.contains(table.initial_state.as_str()) {
        return Err(invalid(format!(
            "initial_state {} not in states",
            table.initial_state
        )));
    }
    for terminal in &table.terminal_states {
        if !states.contains(terminal.as_str()) {
            return Err(invalid(format!("terminal state {terminal} not in states")));
        }
    }
    if table.transitions.is_empty() {
        return Err(invalid("empty transition set".to_owned()));
    }
    let mut selectors: BTreeSet<(&str, &str, &str)> = BTreeSet::new();
    for edge in &table.transitions {
        if !states.contains(edge.from.as_str()) {
            return Err(invalid(format!("edge from unknown state {}", edge.from)));
        }
        if !states.contains(edge.to.as_str()) {
            return Err(invalid(format!("edge to unknown state {}", edge.to)));
        }
        if table.terminal_states.contains(&edge.from) {
            return Err(invalid(format!(
                "terminal state {} has an outgoing edge",
                edge.from
            )));
        }
        if edge.reason_codes.is_empty() {
            return Err(invalid(format!(
                "edge {} -> {} has no reason codes",
                edge.from, edge.to
            )));
        }
        for reason in &edge.reason_codes {
            // Deterministic row selection requires the (from, to, reason)
            // triple to identify at most one row.
            if !selectors.insert((edge.from.as_str(), edge.to.as_str(), reason.as_str())) {
                return Err(invalid(format!(
                    "ambiguous selector ({}, {}, {reason})",
                    edge.from, edge.to
                )));
            }
        }
    }
    Ok(())
}

type TableSlot = OnceLock<Result<LoadedTable, TableAssetError>>;

static TABLES: [TableSlot; 5] = [
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
];

fn slot_index(domain: LifecycleDomain) -> usize {
    match domain {
        LifecycleDomain::AgentExecution => 0,
        LifecycleDomain::Effect => 1,
        LifecycleDomain::Loop => 2,
        LifecycleDomain::Task => 3,
        LifecycleDomain::Verification => 4,
    }
}

/// The loaded, validated, digest-pinned table of one lifecycle domain.
/// Parsed once per process from the embedded registered asset.
pub fn table(domain: LifecycleDomain) -> Result<&'static LoadedTable, TableAssetError> {
    let asset = &EMBEDDED[slot_index(domain)];
    debug_assert_eq!(asset.domain, domain);
    TABLES[slot_index(domain)]
        .get_or_init(|| parse_table(domain.as_str(), asset.text))
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn state(name: &str) -> StateName {
        StateName::parse(name).unwrap()
    }

    #[test]
    fn all_five_registered_tables_load_and_pin_digests() {
        for domain in LifecycleDomain::ALL {
            let loaded = table(domain).unwrap();
            assert_eq!(loaded.table.domain, domain.as_str());
            assert!(
                loaded.digest.starts_with("sha256:") && loaded.digest.len() == 71,
                "digest form for {domain}"
            );
            assert!(!loaded.table.transitions.is_empty());
        }
    }

    #[test]
    fn embedded_assets_match_the_files_under_specs_transitions() {
        // Belt and braces: include_str! already reads the registered file at
        // compile time; this pins the digest of the on-disk asset to the
        // embedded copy so a stale build environment cannot go unnoticed.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        for domain in LifecycleDomain::ALL {
            let path = root
                .join("specs")
                .join("transitions")
                .join(format!("{}.transitions.json", domain.as_str()));
            let text = std::fs::read_to_string(&path).unwrap();
            let from_disk = parse_table(domain.as_str(), &text).unwrap();
            let embedded = table(domain).unwrap();
            assert_eq!(from_disk.digest, embedded.digest, "{domain}");
        }
    }

    #[test]
    fn initial_states_match_registered_tables() {
        let expected = [
            (LifecycleDomain::AgentExecution, "CREATED"),
            (LifecycleDomain::Effect, "PROPOSED"),
            (LifecycleDomain::Loop, "START"),
            (LifecycleDomain::Task, "DRAFT"),
            (LifecycleDomain::Verification, "NOT_REQUESTED"),
        ];
        for (domain, initial) in expected {
            assert_eq!(table(domain).unwrap().table.initial_state, initial);
        }
    }

    #[test]
    fn terminal_states_have_no_exits_and_reject_lookups() {
        for domain in LifecycleDomain::ALL {
            let loaded = table(domain).unwrap();
            for terminal in &loaded.table.terminal_states {
                let from = state(terminal);
                assert!(loaded.legal_exits(&from).is_empty(), "{domain}/{terminal}");
                let to = state(&loaded.table.initial_state);
                assert_eq!(
                    loaded.find_edge(&from, &to, "ANY_REASON").unwrap_err(),
                    EdgeLookupError::TerminalFrom,
                    "{domain}/{terminal}"
                );
            }
        }
    }

    #[test]
    fn edge_lookup_selects_by_reason_and_reports_reason_mismatch() {
        let effect = table(LifecycleDomain::Effect).unwrap();
        let from = state("OUTCOME_UNKNOWN");
        let to = state("RECONCILED");
        let executed = effect
            .find_edge(&from, &to, "RECONCILIATION_CONFIRMED_EXECUTED")
            .unwrap();
        assert_eq!(
            executed.metadata.as_ref().unwrap()["reconciliation_result"],
            "executed"
        );
        let still_unknown = effect
            .find_edge(&from, &to, "RECONCILIATION_STILL_UNKNOWN")
            .unwrap();
        assert_eq!(
            still_unknown.metadata.as_ref().unwrap()["reconciliation_result"],
            "still_unknown"
        );
        assert_eq!(
            effect.find_edge(&from, &to, "NOT_A_REGISTERED_REASON"),
            Err(EdgeLookupError::ReasonNotAllowed)
        );
    }

    #[test]
    fn outcome_unknown_has_no_direct_commit_or_verify_path() {
        // state-and-transition-contract.md section 4.
        let effect = table(LifecycleDomain::Effect).unwrap();
        let exits = effect.legal_exits(&state("OUTCOME_UNKNOWN"));
        assert_eq!(exits, vec!["RECONCILED"]);
    }

    #[test]
    fn unknown_states_are_rejected_before_edge_search() {
        let task = table(LifecycleDomain::Task).unwrap();
        assert_eq!(
            task.find_edge(&state("NOT_A_STATE"), &state("READY"), "X"),
            Err(EdgeLookupError::UnknownFromState)
        );
        assert_eq!(
            task.find_edge(&state("DRAFT"), &state("NOT_A_STATE"), "X"),
            Err(EdgeLookupError::UnknownToState)
        );
        assert_eq!(
            task.find_edge(&state("COMPLETED"), &state("ACTIVE"), "X"),
            Err(EdgeLookupError::TerminalFrom)
        );
        assert_eq!(
            task.find_edge(&state("DRAFT"), &state("ACTIVE"), "X"),
            Err(EdgeLookupError::NoMatchingEdge)
        );
    }

    #[test]
    fn corrupted_assets_are_rejected() {
        // Not JSON at all.
        assert!(matches!(
            parse_table("task", "{"),
            Err(TableAssetError::Parse { .. })
        ));
        // Terminal state with an outgoing edge.
        let bad = r#"{"domain":"task","version":"0.1","status":"Draft",
            "initial_state":"A","states":["A","B"],"terminal_states":["B"],
            "transitions":[
              {"from":"A","to":"B","reason_codes":["GO"],"guards":[],"required_evidence":[]},
              {"from":"B","to":"A","reason_codes":["BACK"],"guards":[],"required_evidence":[]}
            ]}"#;
        assert!(matches!(
            parse_table("task", bad),
            Err(TableAssetError::Invalid { .. })
        ));
        // Ambiguous (from, to, reason) selector.
        let ambiguous = r#"{"domain":"task","version":"0.1","status":"Draft",
            "initial_state":"A","states":["A","B"],"terminal_states":["B"],
            "transitions":[
              {"from":"A","to":"B","reason_codes":["GO"],"guards":[],"required_evidence":[]},
              {"from":"A","to":"B","reason_codes":["GO"],"guards":["x"],"required_evidence":[]}
            ]}"#;
        assert!(matches!(
            parse_table("task", ambiguous),
            Err(TableAssetError::Invalid { .. })
        ));
        // Domain mismatch between file and registration.
        let wrong_domain = r#"{"domain":"loop","version":"0.1","status":"Draft",
            "initial_state":"A","states":["A"],"terminal_states":[],
            "transitions":[{"from":"A","to":"A","reason_codes":["GO"],"guards":[],"required_evidence":[]}]}"#;
        assert!(matches!(
            parse_table("task", wrong_domain),
            Err(TableAssetError::Invalid { .. })
        ));
    }
}
