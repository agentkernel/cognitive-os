//! Library surface for the `admin-cli` package.
//!
//! Hosts the Personal product CLI (`cognitive` bin, P1-T06) as a reusable
//! module so integration tests can exercise command handlers without shelling
//! every path. The deterministic management binary (`admin-cli`) keeps its
//! own `main.rs` entry and does not depend on this Personal surface for
//! authority verbs.

pub mod personal_cli;

pub use personal_cli::{CognitiveCommand, run_cognitive_command};
