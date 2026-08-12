//! Daemon-private native Tool execution admission and bounded request shape.
//!
//! This module does not grant authority and does not replace Intent/Effect
//! persistence; it converts an already daemon-bound Tool descriptor into a
//! request the persistent protocol can safely execute.
//!
//! Production helpers are split into cohesive submodules (P9-T02/D03) without
//! behavior change; this façade preserves crate and test import paths.

#![allow(unused)]

mod process;
mod search;
mod types;
mod validate;
mod workspace;

pub(crate) use process::*;
pub(crate) use search::*;
pub(crate) use types::*;
pub(crate) use validate::*;
pub(crate) use workspace::*;

/// Operation families this daemon actually assembled an executor for.
///
/// P2-T09/D01: registry availability is a descriptor fact and says nothing
/// about whether this binary can dispatch. Every other registered family
/// currently fails staging with `UnsupportedExecutionFamily`, so the projection
/// must derive readiness from this list rather than from availability.
pub const ASSEMBLED_EXECUTOR_FAMILIES: [cognitive_kernel::tool_registry::NativeOperationFamily; 2] = [
    cognitive_kernel::tool_registry::NativeOperationFamily::WorkspaceRead,
    cognitive_kernel::tool_registry::NativeOperationFamily::ProcessCheck,
];

#[cfg(test)]
mod tests;
