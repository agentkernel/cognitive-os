//! Pure capability constraint model: monotone attenuation and
//! intersection (REQ-CAP-001/002; `docs/standards/authn-authz-capability.md`
//! sections 2-3).
//!
//! This module is arithmetic over deterministic data. It never consults a
//! store, a clock, or a model; the kernel authorization gate supplies the
//! decision instant and the current revocation epoch. Two laws hold
//! everywhere and are property-tested:
//!
//! - **Attenuation is monotone**: a derived capability may only narrow its
//!   parent — scope, actions, parameters, lease, delegation depth. Any
//!   widening is a violation (`AUTH_CAPABILITY_ATTENUATION_VIOLATION`,
//!   vector `capability-attenuation.json`).
//! - **Intersection only shrinks**: the effective right of a chain is the
//!   intersection of its links, never the union (decision order step 3).

use crate::ids::WallTimestamp;
use std::collections::{BTreeMap, BTreeSet};

/// Lease validity window (`authorization-capability.schema.json` `lease`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseWindow {
    /// Not valid before this instant.
    pub not_before: WallTimestamp,
    /// Not valid at or after this instant.
    pub expires: WallTimestamp,
}

impl LeaseWindow {
    /// True when `at` lies inside `[not_before, expires)`.
    pub fn contains(&self, at: &WallTimestamp) -> bool {
        let at = at.instant_key();
        self.not_before.instant_key() <= at && at < self.expires.instant_key()
    }

    /// The overlap of two windows, or `None` when they are disjoint.
    pub fn intersect(&self, other: &LeaseWindow) -> Option<LeaseWindow> {
        let not_before = if self.not_before.instant_key() >= other.not_before.instant_key() {
            self.not_before.clone()
        } else {
            other.not_before.clone()
        };
        let expires = if self.expires.instant_key() <= other.expires.instant_key() {
            self.expires.clone()
        } else {
            other.expires.clone()
        };
        if not_before.instant_key() < expires.instant_key() {
            Some(LeaseWindow {
                not_before,
                expires,
            })
        } else {
            None
        }
    }
}

/// One declared parameter bound of a capability's `parameter_binding`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterBound {
    /// Numeric upper bound (for example `max_amount_minor`).
    NumericMax(i64),
    /// Closed set of allowed values.
    OneOf(BTreeSet<String>),
}

impl ParameterBound {
    /// True when `self` is equal to or tighter than `parent`.
    fn narrows(&self, parent: &ParameterBound) -> bool {
        match (self, parent) {
            (ParameterBound::NumericMax(child), ParameterBound::NumericMax(parent)) => {
                child <= parent
            }
            (ParameterBound::OneOf(child), ParameterBound::OneOf(parent)) => {
                child.is_subset(parent)
            }
            // A bound that changes kind cannot be shown monotone: fail closed.
            _ => false,
        }
    }

    /// Tightest bound admitted by both, or `None` when incompatible/empty.
    fn intersect(&self, other: &ParameterBound) -> Option<ParameterBound> {
        match (self, other) {
            (ParameterBound::NumericMax(a), ParameterBound::NumericMax(b)) => {
                Some(ParameterBound::NumericMax(*a.min(b)))
            }
            (ParameterBound::OneOf(a), ParameterBound::OneOf(b)) => {
                let joined: BTreeSet<String> = a.intersection(b).cloned().collect();
                if joined.is_empty() {
                    None
                } else {
                    Some(ParameterBound::OneOf(joined))
                }
            }
            _ => None,
        }
    }
}

/// Deterministic constraint content of one AuthorizationCapability link
/// (the governance header, signature and issuer verification live with the
/// contracts/schema layer; this is the decision arithmetic surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityConstraints {
    /// Subject the capability was granted to.
    pub subject: String,
    /// Audience (service/authority) the capability may be presented to.
    pub audience: String,
    /// Resource scope URI this capability is bound to. A child resource
    /// narrows by exact match or by descending the `/`-separated path.
    pub resource: String,
    /// Purpose binding (exact label).
    pub purpose: String,
    /// Allowed actions.
    pub actions: BTreeSet<String>,
    /// Parameter bounds by parameter name.
    pub parameter_bounds: BTreeMap<String, ParameterBound>,
    /// Lease validity window.
    pub lease: LeaseWindow,
    /// Remaining delegation depth (0 = may not be derived from).
    pub depth_remaining: i64,
    /// Revocation epoch this capability was issued under.
    pub issued_epoch: i64,
}

/// True when `child` equals `parent` or descends its `/` path.
pub fn resource_within(child: &str, parent: &str) -> bool {
    child == parent || child.starts_with(&format!("{parent}/"))
}

/// Monotone-attenuation check: every violated dimension of `derived`
/// against `parent`, as stable field paths (empty = valid attenuation).
/// Field-path spelling follows vector `capability-attenuation.json`
/// (`parameter_binding.<name>`).
pub fn attenuation_violations(
    parent: &CapabilityConstraints,
    derived: &CapabilityConstraints,
) -> Vec<String> {
    let mut violations = Vec::new();
    if derived.audience != parent.audience {
        violations.push("audience".to_owned());
    }
    if !resource_within(&derived.resource, &parent.resource) {
        violations.push("resource".to_owned());
    }
    if derived.purpose != parent.purpose {
        violations.push("purpose".to_owned());
    }
    if !derived.actions.is_subset(&parent.actions) {
        violations.push("actions".to_owned());
    }
    for (name, parent_bound) in &parent.parameter_bounds {
        match derived.parameter_bounds.get(name) {
            // Dropping a parent bound would widen the right: fail closed.
            None => violations.push(format!("parameter_binding.{name}")),
            Some(child_bound) if !child_bound.narrows(parent_bound) => {
                violations.push(format!("parameter_binding.{name}"));
            }
            Some(_) => {}
        }
    }
    if derived.lease.not_before.instant_key() < parent.lease.not_before.instant_key() {
        violations.push("lease.not_before".to_owned());
    }
    if derived.lease.expires.instant_key() > parent.lease.expires.instant_key() {
        violations.push("lease.expires".to_owned());
    }
    if parent.depth_remaining < 1 {
        violations.push("delegation.depth_exhausted".to_owned());
    } else if derived.depth_remaining >= parent.depth_remaining || derived.depth_remaining < 0 {
        violations.push("delegation.depth_remaining".to_owned());
    }
    violations.sort_unstable();
    violations
}

/// The effective right left after intersecting capability chain links.
/// Empty `actions` (or an impossible window/resource) means no right at
/// all — the caller falls through to default deny.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveRights {
    /// Actions every link allows.
    pub actions: BTreeSet<String>,
    /// Narrowest resource among the links (the descendant path).
    pub resource: Option<String>,
    /// Purpose all links agree on.
    pub purpose: Option<String>,
    /// Tightest parameter bound per name (union of names: every declared
    /// bound of every link applies).
    pub parameter_bounds: BTreeMap<String, ParameterBound>,
    /// Overlap of all lease windows.
    pub lease: Option<LeaseWindow>,
    /// Oldest issue epoch across the links (the chain is only as current
    /// as its stalest link).
    pub oldest_issued_epoch: i64,
}

impl EffectiveRights {
    /// The identity for intersection: everything the single link allows.
    fn from_link(link: &CapabilityConstraints) -> EffectiveRights {
        EffectiveRights {
            actions: link.actions.clone(),
            resource: Some(link.resource.clone()),
            purpose: Some(link.purpose.clone()),
            parameter_bounds: link.parameter_bounds.clone(),
            lease: Some(link.lease.clone()),
            oldest_issued_epoch: link.issued_epoch,
        }
    }

    fn no_rights(&self) -> EffectiveRights {
        EffectiveRights {
            actions: BTreeSet::new(),
            resource: None,
            purpose: None,
            parameter_bounds: self.parameter_bounds.clone(),
            lease: None,
            oldest_issued_epoch: self.oldest_issued_epoch,
        }
    }

    fn intersect_link(&self, link: &CapabilityConstraints) -> EffectiveRights {
        let resource = match &self.resource {
            Some(current) if resource_within(current, &link.resource) => Some(current.clone()),
            Some(current) if resource_within(&link.resource, current) => {
                Some(link.resource.clone())
            }
            _ => None,
        };
        let purpose = match &self.purpose {
            Some(current) if *current == link.purpose => Some(current.clone()),
            _ => None,
        };
        let lease = self
            .lease
            .as_ref()
            .and_then(|window| window.intersect(&link.lease));
        let mut parameter_bounds = self.parameter_bounds.clone();
        let mut impossible_bound = false;
        for (name, bound) in &link.parameter_bounds {
            match parameter_bounds.get(name) {
                None => {
                    parameter_bounds.insert(name.clone(), bound.clone());
                }
                Some(existing) => match existing.intersect(bound) {
                    Some(tighter) => {
                        parameter_bounds.insert(name.clone(), tighter);
                    }
                    None => impossible_bound = true,
                },
            }
        }
        let oldest_issued_epoch = self.oldest_issued_epoch.min(link.issued_epoch);
        if resource.is_none() || purpose.is_none() || lease.is_none() || impossible_bound {
            return EffectiveRights {
                actions: BTreeSet::new(),
                resource,
                purpose,
                parameter_bounds,
                lease,
                oldest_issued_epoch,
            };
        }
        EffectiveRights {
            actions: self.actions.intersection(&link.actions).cloned().collect(),
            resource,
            purpose,
            parameter_bounds,
            lease,
            oldest_issued_epoch,
        }
    }

    /// True when no action right is left.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

/// Intersect all links of a capability chain (decision order step 3): the
/// effective right is the intersection, never the union. An empty chain
/// yields no rights (default deny).
pub fn intersect_chain(links: &[CapabilityConstraints]) -> EffectiveRights {
    let mut iter = links.iter();
    let Some(first) = iter.next() else {
        return EffectiveRights {
            actions: BTreeSet::new(),
            resource: None,
            purpose: None,
            parameter_bounds: BTreeMap::new(),
            lease: None,
            oldest_issued_epoch: 0,
        };
    };
    let mut effective = EffectiveRights::from_link(first);
    for link in iter {
        effective = effective.intersect_link(link);
        if effective.is_empty() {
            // Keep folding epochs/bounds but rights cannot come back.
            let drained = effective.no_rights();
            return links.iter().fold(drained, |acc, l| EffectiveRights {
                oldest_issued_epoch: acc.oldest_issued_epoch.min(l.issued_epoch),
                ..acc
            });
        }
    }
    effective
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn ts(text: &str) -> WallTimestamp {
        WallTimestamp::parse(text).unwrap()
    }

    fn base() -> CapabilityConstraints {
        CapabilityConstraints {
            subject: "principal://tenant-a/agent-1".to_owned(),
            audience: "service://tenant-a/payments".to_owned(),
            resource: "scope://tenant-a/payments".to_owned(),
            purpose: "refund_processing".to_owned(),
            actions: ["refund".to_owned(), "read".to_owned()].into(),
            parameter_bounds: [(
                "max_amount_minor".to_owned(),
                ParameterBound::NumericMax(5000),
            )]
            .into(),
            lease: LeaseWindow {
                not_before: ts("2026-07-18T12:00:00Z"),
                expires: ts("2026-07-18T12:05:00Z"),
            },
            depth_remaining: 1,
            issued_epoch: 41,
        }
    }

    /// Vector `capability-attenuation.json` (CAP-ATTEN-004) arithmetic:
    /// raising max_amount_minor 5000 -> 7500 is amplification.
    #[test]
    fn amplified_parameter_bound_is_a_violation_with_the_vector_field_path() {
        let parent = base();
        let mut derived = base();
        derived.depth_remaining = 0;
        derived.parameter_bounds.insert(
            "max_amount_minor".to_owned(),
            ParameterBound::NumericMax(7500),
        );
        assert_eq!(
            attenuation_violations(&parent, &derived),
            vec!["parameter_binding.max_amount_minor".to_owned()]
        );
    }

    #[test]
    fn valid_attenuation_narrows_every_dimension() {
        let parent = base();
        let mut derived = base();
        derived.depth_remaining = 0;
        derived.actions = ["refund".to_owned()].into();
        derived.resource = "scope://tenant-a/payments/refunds".to_owned();
        derived.parameter_bounds.insert(
            "max_amount_minor".to_owned(),
            ParameterBound::NumericMax(1000),
        );
        derived.lease.expires = ts("2026-07-18T12:04:00Z");
        assert!(attenuation_violations(&parent, &derived).is_empty());
    }

    #[test]
    fn widening_any_dimension_is_rejected() {
        let parent = base();
        let widen = |mutate: &dyn Fn(&mut CapabilityConstraints), field: &str| {
            let mut derived = base();
            derived.depth_remaining = 0;
            mutate(&mut derived);
            let violations = attenuation_violations(&parent, &derived);
            assert!(
                violations.iter().any(|v| v == field),
                "expected {field} in {violations:?}"
            );
        };
        widen(
            &|d| {
                d.actions.insert("transfer".to_owned());
            },
            "actions",
        );
        widen(&|d| d.resource = "scope://tenant-a".to_owned(), "resource");
        widen(&|d| d.purpose = "anything_else".to_owned(), "purpose");
        widen(
            &|d| d.audience = "service://tenant-a/other".to_owned(),
            "audience",
        );
        widen(
            &|d| d.lease.expires = ts("2026-07-18T13:00:00Z"),
            "lease.expires",
        );
        widen(
            &|d| d.lease.not_before = ts("2026-07-18T11:00:00Z"),
            "lease.not_before",
        );
        widen(
            &|d| {
                d.parameter_bounds.clear();
            },
            "parameter_binding.max_amount_minor",
        );
        widen(&|d| d.depth_remaining = 1, "delegation.depth_remaining");
    }

    #[test]
    fn exhausted_delegation_depth_fails_closed() {
        let mut parent = base();
        parent.depth_remaining = 0;
        let mut derived = base();
        derived.depth_remaining = 0;
        assert!(
            attenuation_violations(&parent, &derived)
                .contains(&"delegation.depth_exhausted".to_owned())
        );
    }

    /// F-007 arithmetic law: intersection only ever narrows (REQ-CAP-002,
    /// decision order step 3).
    #[test]
    fn intersection_only_narrows_and_is_commutative() {
        let a = base();
        let mut b = base();
        b.actions = ["refund".to_owned(), "transfer".to_owned()].into();
        b.parameter_bounds.insert(
            "max_amount_minor".to_owned(),
            ParameterBound::NumericMax(900),
        );
        b.lease.not_before = ts("2026-07-18T12:01:00Z");
        b.issued_epoch = 40;

        let ab = intersect_chain(&[a.clone(), b.clone()]);
        let ba = intersect_chain(&[b.clone(), a.clone()]);
        assert_eq!(ab, ba, "intersection is order-independent");
        assert!(ab.actions.is_subset(&a.actions) && ab.actions.is_subset(&b.actions));
        assert_eq!(ab.actions, ["refund".to_owned()].into());
        let lease = ab.lease.expect("windows overlap");
        assert!(lease.not_before.instant_key() >= a.lease.not_before.instant_key());
        assert!(lease.expires.instant_key() <= b.lease.expires.instant_key());
        match ab.parameter_bounds.get("max_amount_minor").unwrap() {
            ParameterBound::NumericMax(max) => assert_eq!(*max, 900, "tightest bound wins"),
            other => panic!("unexpected bound {other:?}"),
        }
        assert_eq!(ab.oldest_issued_epoch, 40, "stalest link epoch governs");

        // Intersecting a chain with itself changes nothing (idempotent).
        let aa = intersect_chain(&[a.clone(), a.clone()]);
        assert_eq!(aa.actions, a.actions);
    }

    #[test]
    fn disjoint_links_leave_no_rights() {
        let a = base();
        let mut other_purpose = base();
        other_purpose.purpose = "reporting".to_owned();
        assert!(intersect_chain(&[a.clone(), other_purpose]).is_empty());

        let mut disjoint_window = base();
        disjoint_window.lease = LeaseWindow {
            not_before: ts("2026-07-18T13:00:00Z"),
            expires: ts("2026-07-18T14:00:00Z"),
        };
        assert!(intersect_chain(&[a.clone(), disjoint_window]).is_empty());

        let mut sibling_resource = base();
        sibling_resource.resource = "scope://tenant-a/billing".to_owned();
        assert!(intersect_chain(&[a, sibling_resource]).is_empty());

        assert!(
            intersect_chain(&[]).is_empty(),
            "empty chain = default deny"
        );
    }

    #[test]
    fn lease_window_contains_is_half_open() {
        let window = LeaseWindow {
            not_before: ts("2026-07-18T12:00:00Z"),
            expires: ts("2026-07-18T12:05:00Z"),
        };
        assert!(window.contains(&ts("2026-07-18T12:00:00Z")));
        assert!(window.contains(&ts("2026-07-18T12:04:59.999Z")));
        assert!(!window.contains(&ts("2026-07-18T12:05:00Z")));
        assert!(!window.contains(&ts("2026-07-18T11:59:59.999999999Z")));
    }
}
