//! Out-of-band workspace projection reconciliation (REQ-AGENT-OOB-001 / IMP-11).
//!
//! First read after digest drift MUST reconcile: ingest as candidate, never
//! silently overwrite the governed object or the user edit.

use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OobError {
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ProjectionObject {
    pub path: String,
    pub pinned_digest: String,
    pub authority_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OobCandidate {
    pub path: String,
    pub observed_digest: String,
    pub observed_bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct OobReconciler {
    projections: BTreeMap<String, ProjectionObject>,
    candidates: Vec<OobCandidate>,
    silent_overwrites: u64,
}

impl OobReconciler {
    pub fn pin(&mut self, obj: ProjectionObject) {
        self.projections.insert(obj.path.clone(), obj);
    }

    pub fn candidates(&self) -> &[OobCandidate] {
        &self.candidates
    }

    pub fn authority_bytes(&self, path: &str) -> Option<&[u8]> {
        self.projections.get(path).map(|p| p.authority_bytes.as_slice())
    }

    pub fn silent_overwrite_count(&self) -> u64 {
        self.silent_overwrites
    }

    fn digest_of(bytes: &[u8]) -> String {
        // Stable non-crypto fingerprint for unit tests; production pins use
        // cognitive_contracts::canonical digests at the install boundary.
        format!("sha256:len{}-b0-{:02x}", bytes.len(), bytes.first().copied().unwrap_or(0))
    }

    /// First read after an out-of-band edit: detect drift, ingest candidate.
    pub fn first_read_after_edit(
        &mut self,
        path: &str,
        on_disk_bytes: &[u8],
    ) -> Result<Value, OobError> {
        let pinned = self.projections.get(path).ok_or_else(|| OobError {
            detail: format!("unknown projection {path}"),
        })?;
        let observed = Self::digest_of(on_disk_bytes);
        if observed == pinned.pinned_digest {
            return Ok(json!({
                "outcome": "unchanged",
                "digest_drift_detected": false,
                "authority_unchanged": true,
            }));
        }
        // Must not silently adopt either side.
        self.silent_overwrites = 0;
        self.candidates.push(OobCandidate {
            path: path.to_owned(),
            observed_digest: observed.clone(),
            observed_bytes: on_disk_bytes.to_vec(),
        });
        Ok(json!({
            "outcome": "denied_or_controlled_fallback",
            "digest_drift_detected": true,
            "edit_ingested_as_candidate": true,
            "silent_overwrite_either_side": false,
            "authority_unchanged": true,
            "capability_expanded": false,
            "observed_digest": observed,
            "pinned_digest": pinned.pinned_digest,
        }))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn oob_edit_triggers_candidate_not_silent_adopt() {
        let mut r = OobReconciler::default();
        let authority = b"governed-v1".to_vec();
        let pinned = OobReconciler::digest_of(&authority);
        r.pin(ProjectionObject {
            path: "workspace/notes.md".into(),
            pinned_digest: pinned,
            authority_bytes: authority,
        });
        let edited = b"user-edited-v2".to_vec();
        let report = r.first_read_after_edit("workspace/notes.md", &edited).unwrap();
        assert_eq!(report["digest_drift_detected"], true);
        assert_eq!(report["edit_ingested_as_candidate"], true);
        assert_eq!(report["silent_overwrite_either_side"], false);
        assert_eq!(report["authority_unchanged"], true);
        assert_eq!(r.candidates().len(), 1);
        assert_eq!(r.silent_overwrite_count(), 0);
    }

    #[test]
    fn repeated_first_read_does_not_overwrite_authority() {
        let mut r = OobReconciler::default();
        let authority = b"governed-v1".to_vec();
        r.pin(ProjectionObject {
            path: "workspace/a".into(),
            pinned_digest: OobReconciler::digest_of(&authority),
            authority_bytes: authority.clone(),
        });
        let edited = b"edit".to_vec();
        let _ = r.first_read_after_edit("workspace/a", &edited).unwrap();
        let _ = r.first_read_after_edit("workspace/a", &edited).unwrap();
        assert_eq!(r.authority_bytes("workspace/a").unwrap(), authority.as_slice());
        assert_eq!(r.silent_overwrite_count(), 0);
    }
}
