//! Personal bounded daemon surface (P1-T04 / ADR-0019).
//!
//! Local authentication bootstrap, resource bounds, single-instance lifecycle
//! locking, and a fail-closed HTTP front door for loopback Personal operation.
//! Not Task scheduling, Memory, MCP, full readiness (P1-T05), or Provider proxy.

mod auth;
mod bounds;
mod lifecycle;
mod server;

pub use auth::{
    ChannelClass, LocalAuthError, LocalSessionAuthority, SessionIssueRequest, SessionTokenView,
};
pub use bounds::{
    PersonalResourceBounds, RequestBoundError, validate_body_length, validate_header_block,
};
pub use lifecycle::{DaemonLifecycleError, DaemonSingleInstanceLock};
pub use server::{PersonalDaemonConfig, PersonalDaemonError, serve_personal_loopback};