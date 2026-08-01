//! P2-T03 failure-first coverage for scheduler ceiling admission.
//!
//! The scheduler must refuse dispatch before leasing when a durable authority
//! snapshot says that any configured deadline, retry, step, or cost ceiling
//! has already been reached.

#![allow(clippy::expect_used)]

use cognitive_runtime::{SchedulerCeilingFacts, SchedulerService, SchedulerStopReason};

fn facts() -> SchedulerCeilingFacts {
    SchedulerCeilingFacts {
        deadline: Some("2026-08-01T12:00:00Z".to_owned()),
        retry_count: 1,
        retry_ceiling: 2,
        completed_steps: 3,
        step_ceiling: 4,
        spent_cost_microunits: 9,
        cost_ceiling_microunits: 10,
    }
}

#[test]
fn reached_authority_ceiling_refuses_dispatch_before_a_lease_is_created() {
    let mut scheduler = SchedulerService::new("worker-1", 60).expect("valid scheduler");
    let mut ceiling_facts = facts();

    ceiling_facts.retry_count = ceiling_facts.retry_ceiling;
    let stop = scheduler
        .evaluate_authority_ceilings(&ceiling_facts, "2026-08-01T11:59:59Z")
        .expect("valid authority facts")
        .expect("reached retry ceiling must stop dispatch");

    assert_eq!(stop, SchedulerStopReason::RetryCeilingReached);
}

#[test]
fn each_authority_ceiling_stops_at_its_inclusive_boundary() {
    let mut scheduler = SchedulerService::new("worker-1", 60).expect("valid scheduler");

    let deadline_stop = scheduler
        .evaluate_authority_ceilings(&facts(), "2026-08-01T12:00:00Z")
        .expect("valid authority facts");
    assert_eq!(deadline_stop, Some(SchedulerStopReason::DeadlineReached));

    let mut step_facts = facts();
    step_facts.deadline = None;
    step_facts.completed_steps = step_facts.step_ceiling;
    assert_eq!(
        scheduler
            .evaluate_authority_ceilings(&step_facts, "2026-08-01T11:59:59Z")
            .expect("valid authority facts"),
        Some(SchedulerStopReason::StepCeilingReached)
    );

    let mut cost_facts = facts();
    cost_facts.deadline = None;
    cost_facts.spent_cost_microunits = cost_facts.cost_ceiling_microunits;
    assert_eq!(
        scheduler
            .evaluate_authority_ceilings(&cost_facts, "2026-08-01T11:59:59Z")
            .expect("valid authority facts"),
        Some(SchedulerStopReason::CostCeilingReached)
    );
}
