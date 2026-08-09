# P4-T05 Memory/Skill API and unified projection closure

## Task boundary

- Task: `P4-T05`
- Slices: `D01-D05`
- Branch: `personal/P4-T05-resource-api`
- Lease: `lease/personal/P4-T05/resource-api`
- PR: `#176`
- Scope: daemon-private Personal projection and application boundary only

## Delivered authority boundary

- Task-channel resource projection routes require a Task bearer and a nonempty
  `task_ref`; management bearers cannot cross that boundary.
- Management projection routes remain separately authenticated.
- Memory explanation loads immutable Memory objects from `MemoryStore`.
- Skill explanation loads binding/package/revision and revocation facts from
  `SkillStore`; revoked bindings remain explainable but do not become eligible.
- Memory forget appends the existing immutable lifecycle tombstone port.
- Skill revoke appends the existing immutable binding revocation port.
- Memory remember delegates to the existing Context-revalidating admission
  service before the store transaction.
- Skill import and binding delegate to immutable SkillStore authority ports.
- All daemon authority writes remain behind the Rust daemon; no client writes
  SQLite directly.

## Failure-first coverage

- missing task reference;
- Task bearer against management projection;
- management bearer against Task projection;
- missing authority object identifier;
- Task bearer against Memory explanation;
- malformed Memory forget payload;
- Task bearer against Skill revoke;
- Memory admission source revalidation and no-partial-object behavior through
  the existing focused admission tests;
- immutable Skill digest/provenance/scope/binding failure behavior through the
  existing focused Skill store tests;
- duplicate lifecycle/import/binding conflicts fail closed through the
  existing authority store contracts.

## Validation

- `cargo fmt --all`: passed locally.
- `git diff --check`: passed locally.
- `pnpm run check:consistency`: passed locally.
- Required Ubuntu and Windows CI run `31335218082`: passed.
- Local Windows Rust build/test/Clippy were not run because the registered GNU
  linker failure makes that environment unsupported for Rust validation.

## Explicit non-claims

This closure does not claim a public API contract, public DTO/schema, B08,
Gate, release, Profile, embedding/vector retrieval, script execution,
capability grants, or completed Context/Task consumption. The projection is a
private versioned daemon observation and the Memory/Skill authority facts retain
their separate stores and lifecycles.
