# ADR-0001: Rust Kernel and TypeScript Clients

- Status: Accepted for the reference implementation baseline
- Date: 2026-07-20
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: reference implementation decision. This ADR binds this
  repository's implementation only; it is NOT a CognitiveOS specification
  requirement. A conforming implementation may use any technology.

## Context

The registered contracts demand determinism the specification cannot get
from goodwill: authorization, CAS, state transitions, budgets, idempotency,
fencing and commits must be deterministic code (whitepaper section 4;
`.cursor/rules/00-architecture-invariants.mdc`), Rust and TypeScript must
produce byte-identical canonical bytes and digests (ADR-0004, golden
fixtures), and clients must be non-authority consumers of projections. The
project needs one systems language for the durable governed core and one
client language for Shell/SDK/Console surfaces, with a small, auditable
toolchain on Windows and Linux.

## Decision

The kernel side (`crates/*`, `apps/kernel-server`, `apps/admin-cli`) is Rust
stable, pinned by `rust-toolchain.toml` (1.97.1 at adoption; updated
deliberately). Tokio is the async runtime for I/O layers when they land
(M2+); `cognitive-domain` and `cognitive-kernel` stay runtime-agnostic and
free of HTTP, SQLite and model SDK types. Workspace lints deny
`unwrap/expect/panic` in library code.

The client side (`packages/contracts-ts`, `packages/sdk-ts`,
`apps/agent-shell`, later Console) is TypeScript with pnpm workspaces
(Node >= 22, tsc strict). TypeScript consumes generated contracts
(ADR-0006) and the `@cognitiveos/contracts-ts` canonical encoding; it never
reimplements governance decisions.

Dependency direction is fixed: contracts → domain → ports (traits) →
adapters → applications. Crate-level enforcement lives in each `Cargo.toml`
and `.cursor/rules/10-rust-kernel.mdc`.

## Alternatives considered

### Single-language TypeScript/Node for kernel and clients

Rejected: the kernel needs deterministic control over memory, panics,
concurrency and fencing on the commit path; a GC runtime plus dynamic
typing weakens exactly the guarantees the architecture sells.

### Go for the kernel

Workable, but rejected for this reference: Rust's type system carries the
newtype ID discipline, exhaustive state-machine matching, and
`Result`-only error flow that the transition tables and error contract
assume; panics-as-values is a poorer fit for fail-closed proofs.

### Sharing one implementation via WASM instead of twin implementations

Deferred: twin implementations with shared golden fixtures are themselves
the cross-language conformance evidence the standard requires
(`canonical-encoding-and-digest.md` section 14). WASM distribution of the
Rust canonicalizer may later reduce client duplication but does not replace
the fixture gate.

## Consequences

Two toolchains (rustup + pnpm) in CI on Windows and Linux; contributors need
both. Cross-language drift is caught by the golden digest gate, not code
review. Rust compile times are paid on every CI run; workspace layering
keeps incremental builds small. The Rust edition (2024) and toolchain pin
are upgraded explicitly with an ADR note, never implicitly.

## Compliance checks

CI matrix runs `cargo build/test/clippy` and `pnpm -r build/test` on
windows-latest and ubuntu-latest, plus the cross-language golden digest
diff. A change that makes either language's digests diverge fails CI.
