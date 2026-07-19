//! `kernel-server`: single-node composition root of the CognitiveOS
//! reference implementation (M5 delivery; M0 skeleton only).
//!
//! Readiness is graded, never binary (whitepaper section 16 and the
//! readiness case of M6): `MANAGEMENT_READY` (deterministic management and
//! recovery verbs available) before `USER_READY` (task channel accepts
//! intents) before `OPERATIONAL` (full governed execution).

/// Ordered readiness grades of the single-node deployment.
pub const READINESS_GRADES: [&str; 3] = ["MANAGEMENT_READY", "USER_READY", "OPERATIONAL"];

fn main() {
    println!(
        "kernel-server M0 skeleton: no server is started. Readiness grades: {}.",
        READINESS_GRADES.join(" -> ")
    );
    println!(
        "Layers wired: contracts={}, domains={}, ports={}, store={}, runtime={}, mgmt-fallback={:?}, akp={}",
        cognitive_contracts::ENCODING_PROFILE,
        cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.len(),
        cognitive_kernel::KERNEL_PORTS.len(),
        cognitive_store::STORE_BACKEND,
        cognitive_runtime::RUNTIME_ROLE,
        cognitive_management::DETERMINISTIC_FALLBACK_VERBS,
        cognitive_akp::TRANSPORT_PROFILE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_grades_are_ordered_management_first() {
        assert_eq!(READINESS_GRADES[0], "MANAGEMENT_READY");
        assert_eq!(READINESS_GRADES.len(), 3);
    }
}
