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
