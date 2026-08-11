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
mod types;
mod validate;
mod workspace;

pub(crate) use process::*;
pub(crate) use types::*;
pub(crate) use validate::*;
pub(crate) use workspace::*;

#[cfg(test)]
mod tests;
