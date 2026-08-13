//! Daemon-private native Tool execution admission and bounded request shape.
//!
//! This module does not grant authority and does not replace Intent/Effect
//! persistence; it converts an already daemon-bound Tool descriptor into a
//! request the persistent protocol can safely execute.
//!
//! Production helpers are split into cohesive submodules (P9-T02/D03) without
//! behavior change; this façade preserves crate and test import paths.

#![allow(unused)]

mod http_fetch;
mod mutate;
mod process;
mod search;
mod secure_fs;
mod state;
mod types;
mod validate;
mod workspace;

pub(crate) use http_fetch::*;
pub(crate) use mutate::*;
pub(crate) use process::*;
pub(crate) use search::*;
pub(crate) use secure_fs::*;
pub(crate) use state::*;
pub(crate) use types::*;
pub(crate) use validate::*;
pub(crate) use workspace::*;

/// Operation families this daemon actually assembled an executor for.
///
/// P2-T09/D01: registry availability is a descriptor fact and says nothing
/// about whether this binary can dispatch. A family absent from this list fails
/// staging with `UnsupportedExecutionFamily`, so the projection must derive
/// readiness from this list rather than from availability.
///
/// P2-T10/D04: all six registered families now have a sink, and
/// `every_assembled_family_has_a_sink_that_accepts_it` fails if this list ever
/// names a family no executor will stage.
///
/// This says one thing only: **this binary contains an executor for the
/// family**. It is not a claim that an Agent can reach it. The production call
/// chain from an admitted Task to one of these sinks does not exist yet — see
/// gaps 1, 2 and 4 on `handbook/*/developer/execution-chain-status.md` — and
/// `execution_ready` must not be read as "an Agent can use this tool".
pub const ASSEMBLED_EXECUTOR_FAMILIES: [cognitive_kernel::tool_registry::NativeOperationFamily; 6] = [
    cognitive_kernel::tool_registry::NativeOperationFamily::WorkspaceRead,
    cognitive_kernel::tool_registry::NativeOperationFamily::WorkspaceSearch,
    cognitive_kernel::tool_registry::NativeOperationFamily::WorkspaceWrite,
    cognitive_kernel::tool_registry::NativeOperationFamily::WorkspacePatch,
    cognitive_kernel::tool_registry::NativeOperationFamily::ProcessCheck,
    cognitive_kernel::tool_registry::NativeOperationFamily::HttpFetchReadOnly,
];

#[cfg(test)]
mod tests;
