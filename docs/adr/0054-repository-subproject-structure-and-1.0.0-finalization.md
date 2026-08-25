# ADR-0054: Repository Subproject Structure and 1.0.0 Boundary Finalization

- Status: Accepted (owner-directed, 2026-08-25)
- Date: 2026-08-25
- Decision owners: repository owner; executed under
  `lease/personal/P0-T08/subproject-restructure`
- Change class: **structural** (path moves and path-literal rewrites only; no
  product, contract, negative-test, or runtime semantic change)
- Task anchor: `P0-T08` (owner-directed repository restructure and 1.0.0
  finalization)

## Context

The repository historically mixed three conceptual layers in one flat tree:
the CognitiveOS architecture/contract layer, the `cognitiveos-personal`
product implementation, and client work that had been moved to the external
`cognitiveos-clients` repository. Version boundaries were also implicit:
Personal `1.0.0` was defined across ADR-0034/0035/0037/0038 and promoted by
the `GMVP-LINUX` Gate (MVP pass under ADR-0049), but no directory, tag, or
finalization document made the 1.0.0 boundary inspectable.

The candidate analysis
`docs/research/agent-work-system/11-repository-governance-and-topology-recommendation.md`
recommended against an immediate multi-repository split and for internal
modularization first. On 2026-08-25 the owner directed a stronger, in-repo
form of that modularization: physically regroup the monorepo into subproject
directories, fold the external clients repository back in, and finalize the
1.0.0 boundaries. This ADR records that owner decision. Where this ADR and
the candidate analysis differ (clients fold-in; immediate physical directory
moves), this ADR is the accepted source; the analysis remains a candidate
research document.

## Decision

### 1. Four subproject directories in one repository

The repository keeps a single Git history, a single Rust workspace, a single
pnpm workspace, and a single atomic CI, but its tree is regrouped into four
subproject roots plus shared governance:

| Root | Subproject | Content |
|---|---|---|
| `core/` | `cognitiveos-core` | Machine contracts and the product-neutral authority substrate: `core/specs/`, `core/conformance/` (vectors), `core/crates/` (`cognitive-contracts`, `cognitive-domain`, `cognitive-kernel`, `cognitive-akp`), `core/packages/contracts-ts/`, `core/tests/golden/`, CognitiveOS architecture whitepapers/RFCs under `core/docs/` |
| `personal/` | `cognitiveos-personal` | The sole active product implementation: `personal/crates/` (`cognitive-store`, `cognitive-runtime`, `cognitive-management`, `cognitive-conformance`, `cognitive-secret`, `cognitive-provider-transport`), `personal/apps/` (`kernel-server`, `admin-cli`, `pi-agent-adapter`, `agent-shell`), `personal/packages/` (`sdk-ts`, `pi-cognitiveos`, `dsh-akp-adapter`), `personal/deploy/`, `personal/handbook/`, `personal/tests/`, Personal product/architecture docs under `personal/docs/` |
| `enterprise/` | `cognitiveos-enterprise` | Design layer only: candidate enterprise product/interaction/architecture documents and the 1.0.0 boundary definition under `enterprise/docs/`. No implementation exists and none is authorized by this ADR |
| `clients/` | clients | The former external `cognitiveos-clients` repository imported at its current `main` (history preserved via subtree merge), plus the Control Plane design corpus (`clients/docs/design/`) and the legacy in-repo client stubs |

Shared, cross-subproject material stays at the repository root: `docs/`
(governance, ADRs, plan, standards, checkpoints, evaluation, traceability,
legal, research), `tools/`, `scripts/`, `.github/`, `.githooks/`, `.cursor/`,
and the root workspace manifests.

### 2. Dependency direction

```text
core  ->  personal  ->  clients (Personal API consumer)
core  ->  enterprise (design-time contract consumer)
core  ->  clients (generated contract consumer)
```

- `core/` packages must not depend on `personal/`, `enterprise/`, or
  `clients/` code.
- `personal/` consumes `core/` crates/packages through the shared workspace.
- Clients never own authority transitions; enterprise central services (when
  ever authorized) send requests and the node daemon remains the sole local
  authority writer.
- Known impurity, accepted and registered as follow-up work rather than
  blocking this restructure: `cognitive-store`, `cognitive-runtime`, and
  `cognitive-management` mix reusable adapter code with Personal product
  code; they move to `personal/` whole. `cognitive-conformance` (the runner
  that executes vectors against the reference implementation) also lives in
  `personal/` as the reference-IUT harness, while the normative vectors stay
  in `core/conformance/`. Splitting the mixed crates into pure core parts and
  Personal parts is a registered future refactor task, not part of this
  structural change.

### 3. 1.0.0 boundaries and finalization

- **`cognitiveos-core` 1.0.0 is development-complete and finalized.** Its
  boundary and acceptance mapping live in
  [`core/docs/VERSION-1.0.0.md`](../../core/docs/VERSION-1.0.0.md). The
  finalization is anchored by annotated tag `core-v1.0.0` on the merge
  revision of this restructure.
- **`cognitiveos-personal` 1.0.0 is development-complete and finalized.** The
  boundary is the existing ADR-0034/0035/0037/0038 definition (Linux x86_64
  single-service, six-resource minimal real slices, Pi + sidecar only),
  promoted by the already-passed Gates B01, B02, B03, B04, B05, B08, B09,
  B12, and `GMVP-LINUX` (MVP, ADR-0049). Boundary, evidence mapping, and the
  post-1.0 roadmap live in
  [`personal/docs/VERSION-1.0.0.md`](../../personal/docs/VERSION-1.0.0.md).
  Finalization tag: `personal-v1.0.0` on the same merge revision. The
  finalization records the MVP claim ceiling of those Gates verbatim; it does
  not upgrade any Gate evidence, Profile, or distribution claim, and the
  clean-VM RC/release-claim work stays open as P7-T06.
- **`cognitiveos-enterprise` 1.0.0 is defined but not started.** Boundary,
  acceptance criteria, and the activation gate live in
  [`enterprise/docs/VERSION-1.0.0.md`](../../enterprise/docs/VERSION-1.0.0.md).
  This ADR authorizes no enterprise implementation.

### 4. Clients fold-in

The external repository `agentkernel/cognitiveos-clients` is imported at its
current `main` into `clients/` with history preserved (subtree merge). After
this restructure merges, the external repository should be archived
(read-only) by the owner with a pointer to `clients/` in this repository;
that archive action is an owner GitHub operation outside this change set.
ADR-0053's stack, serving, session, and threat decisions remain in force; its
"only implementation path is `cognitiveos-clients/pc/web/`" location clause
is superseded by `clients/pc/web/` in this repository. The 2026-07-26
migration note in `PARALLEL-LANES.md` §2.1 is likewise superseded for
location only.

### 5. Tooling and governance migration

All path-bearing infrastructure is rewritten in the same change set: root
`Cargo.toml` and `pnpm-workspace.yaml`, `.github/workflows/`, `tools/src/`
checkers and generators, `handbook/_meta/source-map.json` routing,
`.cursor/rules/`, `AGENTS.md`, `PROJECT-IDENTITY.md`, and documentation
links/indexes. The docs-sync contract and its gate remain in force with
updated paths. No canonical fact changes owner; old paths become new paths,
not second sources.

## Consequences

- The directory tree now states the product boundary that previously lived
  only in governance prose.
- Wire identity, crate names, npm package names, schema IDs, REQ IDs, task
  IDs, Gate IDs, and all recorded evidence are unchanged; historical
  checkpoint/evaluation documents keep their original path references as
  historical facts and are not rewritten.
- A future physical repository split (the candidate analysis's Option D/F)
  becomes a per-directory extraction if its objective gates are ever met;
  nothing in this ADR commits to one.
- Follow-up registered work: internal split of the mixed
  store/runtime/management crates; P7-T06 RC/clean-VM release evidence;
  remaining post-1.0 trains (P6 Multi-Agent, Control Plane W6+, P7-T07
  Windows/B01-W).

## Non-claims

This ADR and the restructure it authorizes create no new Gate, release,
Profile, benchmark, or Agent-benefit claim; the 1.0.0 finalization tags
designate already-recorded MVP Gate outcomes and change no evidence. No
secret, artifact payload, or campaign guest is touched.
