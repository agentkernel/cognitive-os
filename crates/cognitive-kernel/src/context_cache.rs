//! Governance-bound context caches
//! (`docs/standards/context-resolution-and-cache.md` section 4;
//! REQ-PROFILE-CVM-001, REQ-CAP-005).
//!
//! Every cache on the resolution path keys on the full governance binding:
//! tenant, actor-chain digest, capability set version, revocation epoch,
//! purpose, schema digest, encoding profile — plus the conversation
//! binding (the standard's seven dimensions are a floor, not a ceiling).
//! A hit that ignores any dimension is a correctness defect, not an
//! optimization: revocation and membership changes advance the epoch
//! component and invalidate BY KEY MISMATCH, never by best-effort scans.

use crate::authz::AccessDenial;
use crate::error::CONTEXT_AUTH_DENIED;
use serde::Serialize;
use std::collections::BTreeMap;

/// The full governance binding of one cached resolution artifact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GovernanceBinding {
    /// Tenant of the resolution.
    pub tenant: String,
    /// Actor-chain digest of the requesting chain.
    pub actor_chain_digest: String,
    /// Capability set version in force at resolution time.
    pub capability_set_version: i64,
    /// Revocation epoch in force at resolution time.
    pub revocation_epoch: i64,
    /// Purpose binding.
    pub purpose: String,
    /// Schema digest pin of the consuming payload.
    pub schema_digest: String,
    /// Encoding profile identifier.
    pub encoding_profile: String,
    /// Conversation binding (None = non-conversational activity scope).
    pub conversation: Option<String>,
}

/// Derived artifacts hanging off one cached view; all of them die with the
/// entry on invalidation (vector `context-revocation-cache-reuse.json`
/// `derived_caches_invalidated`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivedCacheKind {
    /// Provider KV cache built over the rendered prefix.
    KvCache,
    /// Provider prompt cache entries.
    PromptCache,
    /// Embedding results computed from loaded bodies.
    EmbeddingResult,
    /// Summaries or compressions derived from the view.
    Summary,
}

/// One cached resolution artifact (digests only — the cache never becomes
/// an alternate body store).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedView {
    /// Render digest of the cached view.
    pub render_digest: String,
    /// Refs loaded by the cached view (for invalidation reporting).
    pub loaded_refs: Vec<String>,
    /// Derived caches attached to this entry.
    pub derived: Vec<DerivedCacheKind>,
}

/// All daemon-controlled facts that make a reusable Context rendering safe.
///
/// This is deliberately more specific than [`GovernanceBinding`]: governance
/// currency alone cannot establish that a Task contract, ContextRequest, or
/// selected source revision has remained unchanged. The cache stores no bodies;
/// callers must still revalidate authorization and freshness before reusing
/// any segment metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ContextCacheKey {
    /// Current authorization and rendering governance facts.
    pub governance: GovernanceBinding,
    /// Durable ContextRequest identity and canonical content digest.
    pub context_request_id: String,
    pub context_request_digest: String,
    /// Immutable TaskContract identity for this Context resolution.
    pub task_ref: String,
    pub task_contract_epoch: i64,
    pub task_contract_digest: String,
    /// Ordered source identity/digest pairs selected after metadata filtering.
    /// Their canonical ordering makes the key deterministic and detects both
    /// source replacement and a source-set change.
    pub ordered_source_digests: Vec<ContextSourceDigest>,
    /// Daemon renderer identity. A renderer change never reuses old segments.
    pub renderer_version: String,
    /// Daemon-validated Tool descriptor digest, when a tool-bound delta is
    /// being cached. Pi-provided candidate values are never accepted here.
    pub validated_tool_descriptor_digest: Option<String>,
}

/// One source identity/digest pair participating in a cache key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ContextSourceDigest {
    pub source_ref: String,
    pub content_digest: String,
}

/// Digest-only stable-prefix and delta metadata retained for a Context key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCacheEntry {
    pub render_digest: String,
    pub stable_prefix_segment_digests: Vec<String>,
    pub delta_segment_digests: Vec<String>,
    pub derived: Vec<DerivedCacheKind>,
}

/// Cache decision for the stricter daemon Context cache key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextCacheLookup {
    Hit(ContextCacheEntry),
    MissResolveFresh,
}

/// A private cache for digest-only stable-prefix and delta metadata.
///
/// It intentionally provides no lookup by partial identity. A caller either
/// proves that every governing fact is current and uses the full key, or
/// resolves a fresh Context view.
#[derive(Debug, Default)]
pub struct GovernedContextCache {
    entries: BTreeMap<ContextCacheKey, ContextCacheEntry>,
}

impl GovernedContextCache {
    pub fn insert(&mut self, key: ContextCacheKey, entry: ContextCacheEntry) {
        self.entries.insert(key, entry);
    }

    pub fn lookup_current(&self, current_key: &ContextCacheKey) -> ContextCacheLookup {
        self.entries
            .get(current_key)
            .cloned()
            .map(ContextCacheLookup::Hit)
            .unwrap_or(ContextCacheLookup::MissResolveFresh)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Removes entries whose governance epoch is provably obsolete. Correct
    /// behavior does not depend on this housekeeping because full-key lookup
    /// can never hit an entry from a prior epoch.
    pub fn evict_stale_epochs(&mut self, current_epoch: i64) -> usize {
        let stale_keys = self
            .entries
            .keys()
            .filter(|key| key.governance.revocation_epoch < current_epoch)
            .cloned()
            .collect::<Vec<_>>();
        let removed_entry_count = stale_keys.len();
        for stale_key in stale_keys {
            self.entries.remove(&stale_key);
        }
        removed_entry_count
    }
}

/// Report of one invalidation (audit-facing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationReport {
    /// The stale binding that was purged.
    pub stale_binding: GovernanceBinding,
    /// Derived caches invalidated with the entry.
    pub derived_caches_invalidated: Vec<DerivedCacheKind>,
}

/// Outcome of a cache consultation under a declared (client-remembered)
/// binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheDecision {
    /// Declared binding equals the current governance binding and an entry
    /// exists: safe hit (authorization dimensions re-checked by key
    /// construction — `authorization_skipped_on_cache_hit: false`).
    Hit(CachedView),
    /// No entry under the current binding: resolve fresh.
    MissResolveFresh,
}

/// Governance-keyed view cache.
#[derive(Debug, Default)]
pub struct ContextViewCache {
    entries: BTreeMap<GovernanceBinding, CachedView>,
}

impl ContextViewCache {
    /// Insert a resolved view under its governance binding.
    pub fn insert(&mut self, binding: GovernanceBinding, view: CachedView) {
        self.entries.insert(binding, view);
    }

    /// Number of live entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the cache has no live entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up strictly by the CURRENT governance binding. Any epoch,
    /// capability-version, purpose, tenant, chain, schema, encoding or
    /// conversation difference is a plain miss — stale entries can never
    /// be reached through this path.
    pub fn lookup_current(&self, current: &GovernanceBinding) -> Option<&CachedView> {
        self.entries.get(current)
    }

    /// Serve a request that DECLARES the binding it remembers (for example
    /// a client replaying `requested_via: cache_lookup` with a stale
    /// revocation version). A declared binding that differs from the
    /// current one is refused with `CONTEXT_AUTH_DENIED` — decision
    /// `revalidate_or_reresolve` — and the stale entry plus every derived
    /// cache is purged by key (REQ-CAP-005: a decision from cached
    /// material after epoch advance is a defect).
    pub fn serve_declared(
        &mut self,
        declared: &GovernanceBinding,
        current: &GovernanceBinding,
    ) -> Result<CacheDecision, (AccessDenial, Option<Box<InvalidationReport>>)> {
        if declared != current {
            let report = self.entries.remove(declared).map(|stale| {
                Box::new(InvalidationReport {
                    stale_binding: declared.clone(),
                    derived_caches_invalidated: stale.derived,
                })
            });
            return Err((
                AccessDenial {
                    code: CONTEXT_AUTH_DENIED.code,
                    category: CONTEXT_AUTH_DENIED.category,
                    retryable: CONTEXT_AUTH_DENIED.retryable,
                    detail: "not available for this principal and purpose",
                },
                report,
            ));
        }
        Ok(match self.entries.get(current) {
            Some(view) => CacheDecision::Hit(view.clone()),
            None => CacheDecision::MissResolveFresh,
        })
    }

    /// Purge every entry whose revocation epoch predates `current_epoch`
    /// (housekeeping; correctness never depends on this because stale keys
    /// can no longer match).
    pub fn evict_stale_epochs(&mut self, current_epoch: i64) -> Vec<InvalidationReport> {
        let stale: Vec<GovernanceBinding> = self
            .entries
            .keys()
            .filter(|binding| binding.revocation_epoch < current_epoch)
            .cloned()
            .collect();
        stale
            .into_iter()
            .filter_map(|binding| {
                self.entries
                    .remove(&binding)
                    .map(|view| InvalidationReport {
                        stale_binding: binding,
                        derived_caches_invalidated: view.derived,
                    })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn governed_cache_key() -> ContextCacheKey {
        ContextCacheKey {
            governance: GovernanceBinding {
                tenant: "tenant-a".to_owned(),
                actor_chain_digest: "sha256:actor-chain".to_owned(),
                capability_set_version: 3,
                revocation_epoch: 7,
                purpose: "task-execution".to_owned(),
                schema_digest: "sha256:context-schema".to_owned(),
                encoding_profile: "canonical-json".to_owned(),
                conversation: Some("conversation-a".to_owned()),
            },
            context_request_id: "request-a".to_owned(),
            context_request_digest: "sha256:request".to_owned(),
            task_ref: "task://tenant-a/1".to_owned(),
            task_contract_epoch: 4,
            task_contract_digest: "sha256:contract".to_owned(),
            ordered_source_digests: vec![ContextSourceDigest {
                source_ref: "source://tenant-a/1".to_owned(),
                content_digest: "sha256:source-one".to_owned(),
            }],
            renderer_version: "personal-context-render/1".to_owned(),
            validated_tool_descriptor_digest: Some("sha256:tool".to_owned()),
        }
    }

    fn cache_entry() -> ContextCacheEntry {
        ContextCacheEntry {
            render_digest: "sha256:render".to_owned(),
            stable_prefix_segment_digests: vec!["sha256:header".to_owned()],
            delta_segment_digests: vec!["sha256:source-one".to_owned()],
            derived: vec![DerivedCacheKind::KvCache],
        }
    }

    #[test]
    fn governed_cache_reuses_only_an_exact_current_key() {
        let key = governed_cache_key();
        let entry = cache_entry();
        let mut cache = GovernedContextCache::default();
        cache.insert(key.clone(), entry.clone());

        assert_eq!(cache.lookup_current(&key), ContextCacheLookup::Hit(entry));

        let changed_source_key = ContextCacheKey {
            ordered_source_digests: vec![ContextSourceDigest {
                source_ref: "source://tenant-a/1".to_owned(),
                content_digest: "sha256:source-one-replaced".to_owned(),
            }],
            ..key.clone()
        };
        assert_eq!(
            cache.lookup_current(&changed_source_key),
            ContextCacheLookup::MissResolveFresh
        );

        let changed_contract_key = ContextCacheKey {
            task_contract_epoch: 5,
            ..key
        };
        assert_eq!(
            cache.lookup_current(&changed_contract_key),
            ContextCacheLookup::MissResolveFresh
        );
    }

    #[test]
    fn governed_cache_does_not_reuse_revoked_or_tool_drifted_material() {
        let key = governed_cache_key();
        let mut cache = GovernedContextCache::default();
        cache.insert(key.clone(), cache_entry());

        let revoked_key = ContextCacheKey {
            governance: GovernanceBinding {
                revocation_epoch: 8,
                ..key.governance.clone()
            },
            ..key.clone()
        };
        assert_eq!(
            cache.lookup_current(&revoked_key),
            ContextCacheLookup::MissResolveFresh
        );
        assert_eq!(cache.evict_stale_epochs(8), 1);
        assert_eq!(cache.len(), 0);

        cache.insert(key.clone(), cache_entry());
        let drifted_tool_key = ContextCacheKey {
            validated_tool_descriptor_digest: Some("sha256:tool-replaced".to_owned()),
            ..key
        };
        assert_eq!(
            cache.lookup_current(&drifted_tool_key),
            ContextCacheLookup::MissResolveFresh
        );
    }
}
