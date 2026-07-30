//! Campaign-only payload for validating pre-pointer compensation on Linux.
//!
//! This binary is never selected by the production installer. A signed
//! non-production campaign release packages it as `bin/kernel-server`; the
//! fixed production unit then runs it normally. It exits before binding the
//! fixed health port, forcing the first active-service confirmation to fail.

fn main() {
    // A non-zero status makes systemd's existing `Restart=on-failure` policy
    // behave as it would for a real daemon that exits during startup.
    std::process::exit(1);
}
