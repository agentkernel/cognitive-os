//! Personal bounded daemon surface (P1-T04 / ADR-0019).
//!
//! Local authentication bootstrap, resource bounds, single-instance lifecycle
//! locking, fail-closed HTTP front door, and readiness/status/doctor projection
//! (P1-T05). Not Task scheduling, Memory, MCP, or Provider proxy.

mod auth;
mod bounds;
mod lifecycle;
mod readiness;
mod server;

pub use bounds::PersonalResourceBounds;
pub use server::{PersonalDaemonConfig, serve_personal_loopback};
