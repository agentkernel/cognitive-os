#![allow(unused, unused_imports)]

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{NativeOperationFamily, NativeToolDescriptor, ToolRisk};
use cognitive_kernel::{
    authz::AuthorizationGrant,
    effects::{EffectError, EffectProtocol, GovernanceCurrency, WriterLease},
    engine::CommittedTransition,
    executor::{
        DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    },
    ports::PortFailure,
};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;

use super::*;

/// Bounded daemon-private input for one native Tool operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeToolExecutionRequest {
    pub descriptor: NativeToolDescriptor,
    pub target: String,
    pub input: Vec<u8>,
    pub workspace_root: Option<PathBuf>,
}

/// Validated execution input. The validated target is still an observation
/// boundary; authority, budget, fencing, and Effect state remain server-side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedNativeToolRequest {
    pub descriptor: NativeToolDescriptor,
    pub target: String,
    pub input: Vec<u8>,
    pub approved_workspace_root: Option<PathBuf>,
    pub resolved_workspace_path: Option<PathBuf>,
}

/// Monotonic cursor over bounded output chunks. A cursor never exposes more
/// than the descriptor ceiling and cannot replay an already acknowledged
/// chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundedOutputCursor {
    output_limit_bytes: usize,
    next_offset: usize,
}

impl BoundedOutputCursor {
    pub(crate) fn new(output_limit_bytes: usize) -> Self {
        Self {
            output_limit_bytes,
            next_offset: 0,
        }
    }

    pub(crate) fn next_chunk(
        &mut self,
        output: &[u8],
        requested_offset: usize,
        maximum_chunk_bytes: usize,
    ) -> Result<Option<(usize, Vec<u8>)>, NativeToolExecutionError> {
        if requested_offset != self.next_offset {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "output cursor is stale or out of order".to_owned(),
            ));
        }
        let bounded_output = &output[..output.len().min(self.output_limit_bytes)];
        if self.next_offset >= bounded_output.len() {
            return Ok(None);
        }
        let chunk_end = (self.next_offset + maximum_chunk_bytes.max(1)).min(bounded_output.len());
        let chunk = bounded_output[self.next_offset..chunk_end].to_vec();
        let chunk_offset = self.next_offset;
        self.next_offset = chunk_end;
        Ok(Some((chunk_offset, chunk)))
    }
}

/// Redact values that must never enter ordinary Tool output or evidence.
pub(crate) fn redact_sensitive_output(output: &str) -> String {
    let mut redacted_output = output.to_owned();
    for sensitive_marker in ["api_key=", "API_KEY=", "token=", "TOKEN="] {
        let mut search_start = 0;
        while let Some(relative_marker_start) =
            redacted_output[search_start..].find(sensitive_marker)
        {
            let marker_start = search_start + relative_marker_start;
            let value_start = marker_start + sensitive_marker.len();
            let value_end = redacted_output[value_start..]
                .find([' ', '\n', '\r', '&'])
                .map_or(redacted_output.len(), |relative_end| {
                    value_start + relative_end
                });
            redacted_output.replace_range(value_start..value_end, "[REDACTED]");
            // Continue after the replacement. Searching from the beginning
            // would rediscover the marker we intentionally retain for context.
            search_start = value_start + "[REDACTED]".len();
        }
    }
    redacted_output
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum NativeToolExecutionError {
    #[error("descriptor binding is invalid: {0}")]
    InvalidDescriptor(String),
    #[error("tool input exceeds descriptor limit: {observed} > {limit}")]
    InputTooLarge { observed: usize, limit: usize },
    #[error("workspace target is required for this operation")]
    WorkspaceTargetRequired,
    #[error("workspace target escapes the daemon-approved workspace root")]
    WorkspaceTargetEscapesRoot,
    #[error("workspace mutation requires a non-empty input")]
    MutationInputRequired,
    #[error("network target must use HTTPS")]
    NetworkTargetMustUseHttps,
    #[error("network target contains credentials")]
    NetworkTargetContainsCredentials,
    #[error("process target is not a bounded process identifier")]
    InvalidProcessTarget,
    #[error("this executor only accepts its validated native operation family")]
    UnsupportedExecutionFamily,
    #[error("idempotency key is already bound to a different native operation")]
    IdempotencyBindingConflict,
    #[error("native Tool executor is unavailable: {0}")]
    ExecutorUnavailable(String),
}
