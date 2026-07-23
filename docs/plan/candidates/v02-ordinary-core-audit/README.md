# Ordinary Core AUDIT candidate freeze (review-only)

Status: **review-only candidate; non-registered; non-published; non-selected;
non-conformance; no Profile claim.**

This directory freezes the smallest candidate contract needed by the implemented
Ordinary Core `status.inspect` audit-before-result path. It is deliberately
outside `specs/`, registries, transitions, and conformance vectors. Its JSON
objects are review inputs only; they create no generated bindings, machine
registration, CA-0 GO, behavior-pass, or Profile-implemented claim.

The physical canonical JSONL journal is an adapter detail. Only the safe record,
receipt, operation responsibility, and their stated digest rules are candidates.

Run `cargo test -p cognitive-contracts --test ordinary_core_audit_candidate` to
recompute every listed raw byte hash and canonical digest. The test uses the
repository's `cognitive_contracts::canonical` implementation.
