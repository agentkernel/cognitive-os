# Lane-CTR Ordinary Core AUDIT vector-mapping handoff

- Date: 2026-07-23
- Base: `origin/main@8992473a37c6db3fc6f28349a81709da457b4b5a`
- Scope: unblock only the registry mapping required for Lane-CFR's minimal
  Ordinary Core `status.inspect` AUDIT behavior vector
- Result: **no registry change committed: the required vector does not yet
  exist, and Lane-CTR is prohibited from creating it**

## 1. Read-only diagnosis and exact mapping

The minimal future vector ID is `ORDINARY-CORE-AUDIT-INSPECT-001`. Its scope,
as recorded by the Lane-CFR handoff, is limited to the already registered
formal decision/receipt contract: audit-before-result success release, formal
carrier shape, durable journal readback/digest, receipt identity/request
binding with positive sequence and writer epoch, and mismatch withholding.

It maps to exactly these existing requirements:

| Registry requirement | Existing normative source | Required mapping |
|---|---|---|
| `REQ-AUDIT-001` | `specs/core/ordinary-core-audit.md` decision-record admission/digest rules | add `ORDINARY-CORE-AUDIT-INSPECT-001` to `tests` once the vector exists |
| `REQ-AUDIT-002` | `specs/core/ordinary-core-audit.md` minimal port responsibility | add `ORDINARY-CORE-AUDIT-INSPECT-001` to `tests` once the vector exists |

The matching matrix entries are `docs/traceability/matrix.yaml` entries for
`REQ-AUDIT-001` and `REQ-AUDIT-002`; after the vector exists, each must list
`conformance/vectors/ordinary-core-audit-inspect-001.json` and retain a
non-claim note until the runner executes it.

This is an IMP-01-compatible **correction**: it completes test traceability
for two existing REQs and their already registered formal contract. It adds no
REQ domain, object family, Profile, schema meaning, candidate byte, authority,
or High-Assurance capability.

## 2. Why no change was made

`specs/registry/requirements.yaml` currently maps both REQs only to
`SPEC-CONTRACT-COVERAGE-001`; the matching matrix entries likewise list only
`conformance/vectors/spec-contract-coverage.json`. The intended new vector is
not present under `conformance/vectors/`.

The consistency checker requires every registry test mapping to resolve to an
existing vector. Therefore adding the registry mapping alone would create a
known CI failure, while creating that vector is explicitly outside Lane-CTR's
allowed paths for this task. No temporary red registry state was created.

## 3. Validation and evidence boundary

- Read-only mapping evidence: registry lines 649-660, matrix entries
  1494-1536, and `specs/core/ordinary-core-audit.md` lines 10-33.
- Existing runner evidence remains 84 vectors / 59 pass / 25 not-run / 0 fail;
  it is not Ordinary Core AUDIT behavior evidence.
- No behavior vector, runner, RUN production implementation, schema, generated
  binding, candidate, or golden asset was modified.
- No AUDIT behavior pass, CA-0 GO, High-Assurance, or Profile `implemented`
  claim is made. D-022 remains blocking overall.

## 4. Required atomic next step

Lane-CFR and Lane-CTR must coordinate one atomic compatible batch:

1. Lane-CFR adds `conformance/vectors/ordinary-core-audit-inspect-001.json`
   with the exact two `requirement_ids` above and its failing-first runner case.
2. Lane-CTR adds that vector ID to both REQs' registry `tests` arrays and adds
   its exact vector path to both corresponding matrix entries.
3. Run `pnpm run check:consistency` and `node tools/src/gen-matrix.mjs --check`
   only with both halves present; then Lane-CFR may implement and execute the
   behavior runner.

## 5. Snapshot

- `PROGRESS.md` / `PARALLEL-LANES.md`: already state the CTR-mapping blocker;
  no redundant status edit was made.
- Commit: none. Existing worktree changes were preserved and no incomplete
  registry mapping was staged.
