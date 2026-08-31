//! Personal bounded daemon surface (P1-T04 / ADR-0019).
//!
//! Local authentication bootstrap, resource bounds, single-instance lifecycle
//! locking, fail-closed HTTP front door, and readiness/status/doctor projection
//! (P1-T05). Not Task scheduling, Memory, MCP, or Provider proxy.

mod auth;
mod bounds;
#[cfg_attr(not(test), allow(dead_code))]
mod campaign_observation;
#[cfg_attr(not(test), allow(dead_code))]
mod capability_truth;
mod fault_profile;
mod headless_vault_doctor;
mod lifecycle;
mod memory_skill_consumer;
mod observation;
mod operability_doctor;
#[cfg(test)]
mod p2_t17_a7_failure_first;
mod pi_runtime;
mod pinned_https;
mod project_aggregate;
mod provider_control_plane;
mod provider_proxy;
mod readiness;
mod registered_check;
mod resource_api;
mod resource_manager;
mod route_observation;
mod scheduler_authority;
mod server;
mod six_resource_doctor;
mod skill_package;
mod task_api;
mod tool_executor;
mod tool_lifecycle;
mod user_backup;
mod verification_executor;
mod windows_host;
mod x_connector;

pub use bounds::PersonalResourceBounds;
pub(crate) use registered_check::run_registered_check_worker;
pub use server::{PersonalDaemonConfig, serve_personal_loopback};
