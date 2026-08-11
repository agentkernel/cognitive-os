//! Daemon-only durable scheduler authority reads (P2-T03).
//!
//! This module owns daemon-private scheduler authority reads and one bounded
//! worker-attempt composition boundary. It reloads immutable TaskContract and
//! Effect identities before every durable decision; it never accepts a Task.
//!
//! Production helpers are split into cohesive submodules (P9-T02/D02) without
//! behavior change; this façade preserves crate and test import paths.

#![allow(dead_code, clippy::items_after_test_module)]

mod candidate;
mod context;
mod dispatch;
mod effect;
mod error;
mod policy;
mod types;
mod worker;

pub(crate) use candidate::*;
pub(crate) use context::*;
pub(crate) use dispatch::*;
pub(crate) use effect::*;
pub(crate) use error::*;
pub(crate) use policy::*;
pub(crate) use types::*;
pub(crate) use worker::*;

// Re-export for focused tests that resolve `super::ResolvedContextView`.
#[allow(unused_imports)]
pub(super) use cognitive_kernel::context::ResolvedContextView;

#[cfg(test)]
mod tests;
