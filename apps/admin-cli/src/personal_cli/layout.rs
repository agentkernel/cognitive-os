//! Resolve Personal XDG layout roots for the product CLI.

use std::collections::BTreeMap;
use std::path::PathBuf;

use cognitive_store::{PersonalDataLayout, PersonalLayoutError};

/// Explicit or environment-derived roots for Personal layout resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRoots {
    /// When set, all five XDG-style roots nest under this hermetic directory
    /// (tests and `kernel-server --runtime-root` parity).
    pub runtime_root: Option<PathBuf>,
}

impl LayoutRoots {
    /// Parse shared CLI flags (`--runtime-root`).
    pub fn from_flags(flags: &BTreeMap<String, String>) -> Result<Self, String> {
        Ok(Self {
            runtime_root: flags.get("runtime-root").map(PathBuf::from),
        })
    }
}

/// Build a [`PersonalDataLayout`] from CLI roots or live XDG environment.
pub fn build_layout(roots: &LayoutRoots) -> Result<PersonalDataLayout, PersonalLayoutError> {
    if let Some(runtime_root) = &roots.runtime_root {
        return Ok(PersonalDataLayout::from_xdg_roots(
            runtime_root.join("config"),
            runtime_root.join("data"),
            runtime_root.join("state"),
            runtime_root.join("cache"),
            runtime_root.clone(),
        ));
    }
    PersonalDataLayout::resolve_from_env()
}

/// Resolve layout roots used by integration tests and helpers.
pub fn resolve_layout_roots(runtime_root: Option<PathBuf>) -> LayoutRoots {
    LayoutRoots { runtime_root }
}
