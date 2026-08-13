//! Personal bounded daemon surface (P1-T04 / ADR-0019).
//!
//! Local authentication bootstrap, resource bounds, single-instance lifecycle
//! locking, fail-closed HTTP front door, and readiness/status/doctor projection
//! (P1-T05). Not Task scheduling, Memory, MCP, or Provider proxy.

mod auth;
mod bounds;
mod headless_vault_doctor;
mod lifecycle;
mod operability_doctor;
#[cfg(test)]
mod p2_t17_a7_failure_first;
mod pi_runtime;
mod provider_proxy;
mod readiness;
mod resource_api;
mod scheduler_authority;
mod server;
mod six_resource_doctor;
mod skill_package;
mod task_api;
mod tool_executor;
mod verification_executor;

pub use bounds::PersonalResourceBounds;
pub use server::{PersonalDaemonConfig, serve_personal_loopback};
