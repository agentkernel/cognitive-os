//! `admin-cli`: deterministic management CLI of the CognitiveOS reference
//! implementation (M5 delivery; M0 skeleton only).
//!
//! Hard rule: this binary must never depend on a model SDK. It is the
//! fallback path that keeps inspect / stop / revoke / reconcile available
//! when no model is reachable (`management-deterministic-fallback.json`).

fn main() {
    println!(
        "admin-cli M0 skeleton. Deterministic verbs (implemented in M5): {}",
        cognitive_management::DETERMINISTIC_FALLBACK_VERBS.join(", ")
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn verbs_come_from_management_crate() {
        assert!(cognitive_management::DETERMINISTIC_FALLBACK_VERBS.contains(&"reconcile"));
    }
}
