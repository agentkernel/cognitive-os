# PERSONAL-PERF-EVAL-002 preregistration and freeze record

- Status: **frozen; execution started**
- Campaign ID: `PERSONAL-PERF-EVAL-002`
- Kind: owner-directed evaluation campaign
  ([Operating Model §2.5](../governance/DEVELOPMENT-OPERATING-MODEL.md)); not a
  `P*-T*` task, not a Delivery Slice, not a product Gate
- Execution plan (execution contract): [personal-performance-benchmark-execution-plan.md](../evaluation/personal-performance-benchmark-execution-plan.md)
  v1.1
- Campaign lease: `lease/personal/EVAL-20260812/performance-evaluation-002`
- Branch: `main` (documentation-only writes under the evaluation lease)
- Operator: standing owner-authorized campaign operator (single session)
- Independent reviewer: **not available** in this session; verifier disposition
  is `not_reviewed` and the claim ceiling stays `hypothesis`
- Claim ceiling: `hypothesis` / non-claim. No Gate, release, Profile, B01,
  B01-W, or Agent-benefit promotion may cite this campaign.
- Final report target: `docs/evaluation/personal-performance-assessment-20260812.md`

This document is plan §9 step 1 (freeze) and step 2 (review disposition). It is
written **before** any campaign measurement executes on the target guest, and
every later execution record is appended to it in the section
[§10 Execution record](#10-execution-record).

## 1. Frozen source revision

| Fact | Value |
|---|---|
| Campaign source revision | `4cbec8470bc7a19f23f978e8754ed20133122eb1` |
| Ref | `main`, pushed; `HEAD == origin/main` at freeze time |
| Worktree | clean (`git status --porcelain` empty) |
| Plan implementation reading baseline | `d514e8ac6aa539864a0a889b9f0a58be009521ef` |
| Product drift since the reading baseline | **none** |

The only paths that changed between the plan's reading baseline and the frozen
campaign revision are `tools/package.json`, `tools/src/check-consistency.mjs`,
`tools/src/docs-sync-gate.mjs`, `tools/test/check.test.mjs` and
`tools/test/docs-sync-gate.test.mjs` — the P8-T08 docs-sync gate and its tests.
No path under `crates/`, `apps/`, `packages/`, `specs/` or `conformance/`
differs. The measured product surface is therefore identical to the surface the
plan was written against, and the plan's capability truth table (§1.1) applies
unchanged.

## 2. Environments

| Role | Environment ID | Use |
|---|---|---|
| Measurement target | `B01-DESKTOP-002`, guest `B01-Desktop-Linux-002` | every campaign cell |
| Build/staging host | `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, `hal9000`) | exact-revision clone and release build only |

### 2.1 Build host facts at freeze

Ubuntu 22.04 class, kernel `6.8.0-83-generic`, x86_64, 36 logical CPUs,
31 GiB RAM, glibc 2.35, Rust `cargo 1.97.1`, Node `v22.19.0`, pnpm `11.18.0`,
Git `2.34.1`. The frozen revision was cloned from
`git@github.com:agentkernel/cognitive-os.git` into the disposable root
`~/perfeval002/build` and verified equal to the campaign revision with a clean
worktree. GitHub HTTPS transport was measured unusable from this host at freeze
time (`Operation too slow`, under 1000 B/s); the SSH transport succeeded. This
is a recorded environment fact, not a product finding.

### 2.2 Target guest facts at freeze

| Fact | Observed value |
|---|---|
| Hostname | `hal9001-Standard-PC-Q35-ICH9-2009` |
| OS | Ubuntu 24.04.4 LTS |
| Kernel | `7.0.0-28-generic` |
| glibc | 2.39 |
| vCPU / RAM | 2 / 3911 MiB |
| Node / npm present | `v22.23.2` / `10.9.8` |
| `git`, `cargo`, `pnpm` present | no (clean of developer toolchain) |
| libvirt power state | running |
| Snapshot present | `b01-platform-qualified-baseline` (2026-08-09), untouched |

**The guest is not in its pristine qualified baseline.** It carries residual
state from the P9-T04 campaign: a populated `~/p9t04` root (product binaries,
Pi package, runtime root with `authority.sqlite`, `provider.json`,
`selected-model.json`, `pi.json`) and a still-running
`kernel-server --personal --bind 127.0.0.1:48181 --runtime-root ~/p9t04/runtime`
process. This campaign **does not revert the snapshot** and **does not modify or
stop the P9-T04 residue**, because reverting would destroy another campaign's
recorded state and is outside this campaign's allowlist.

Consequences, recorded now rather than discovered later:

1. no clean-install, first-install, or B01-class claim is available from this
   campaign at all;
2. an idle second daemon is present as background load for every measurement;
   resource and tail figures are reported against that stated background, not
   against an idle machine;
3. this campaign runs in its own campaign root and its own loopback port so
   that neither campaign's authority store can influence the other.

Cross-build direction is `glibc 2.35 → 2.39` (forward compatible), the same
direction P9-T04 used and verified.

## 3. Arms and what is actually executable

Plan §2.1 defines four arms. Their disposition under the measurement-only
boundary is fixed here, before execution.

| Arm | Definition | Disposition | Reason |
|---|---|---|---|
| `D` daemon diagnostic | client → daemon → DeepSeek, no Pi | **executable** | driven by the existing frozen runner `tools/personal/p9-t04-l3-provider-route-runner.mjs` |
| `O` OS Pi | Pi → CognitiveOS Extension → daemon proxy → DeepSeek | **executable, single-marker only** | driven by the existing `tools/personal/p1-t09-product-route-smoke.sh`; no multi-task corpus runner exists |
| `P` pure Pi | Pi → approved baseline credential broker → DeepSeek | **`not-run`** | see §3.1 |
| `G` governed Task | admission → Context → candidate → Tool/Effect → verifier → acceptance | **`not-run`** | production call chain absent (plan §1.1); admission half only, via `UJ4` |

### 3.1 The `P` arm is `not-run`, and why that is a boundary decision

Plan §2.2 requires the `P` arm to use either a Pi-supported approved OS
SecretStore path, or a campaign-only loopback credential broker. Plan §2.5
additionally requires a preregistered, digest-frozen campaign-only **paired
runner** before `B1`/`B2` may execute. Neither the broker nor the paired runner
exists in the repository at the frozen revision.

Operating Model §2.5 item 3 states that a missing capability, **runner**, or
**credential path** is recorded as `not-run`/`not_available` and is **never
implemented mid-campaign**. Building the broker or the paired runner is
therefore not available to this campaign. The consequence is decided here and
not renegotiated later:

- the `P` arm is `not-run`;
- every `O vs P` paired comparison is `not-run`, which is the entire
  confirmatory composition of plan §4.8 (`G1/G2/G3/G4/G6/G9-C0`, `A1-C0`,
  `A4-C0`, `A5-C0`, 270 paired task-seeds);
- `B1` pilot and `B2` confirmatory are `not-run` as batches, not as failures;
- no non-inferiority threshold (plan §7.2) and no benefit claim (plan §7.3) is
  evaluated, met, or failed. Reading any single-arm number in this campaign as
  a threshold outcome is a category error.

This matches the execution plan's own §0 statement that the formal `P/O`
campaign is initially `not-run` and that "design is feasible" must not be
written as "already executable".

### 3.2 Measurement instrument boundary applied in this campaign

To keep the previous paragraph auditable rather than elastic, this campaign
applies one explicit line and records it in the final report:

- **Permitted:** running the repository's already-existing runners at the
  frozen revision, unmodified; and directly invoking the product's own shipped
  public surface (its CLI binaries and its loopback HTTP endpoints) with
  standard system utilities, timing the calls and retaining outcomes.
- **Not permitted:** authoring any new campaign runner, credential broker,
  task-corpus generator, oracle, scorer, or paired-arm coordinator; and
  modifying product code, contracts, negatives, tests, or generated
  documentation sources for any reason.

Cells reachable only through the second list are `not-run`. Cells executed via
direct public-surface invocation are labelled **public-surface observation** in
the final report — they carry full denominators but are not frozen preregistered
runners, and they are never presented as a paired campaign.

## 4. Guest change allowlist

Preregistered and permitted without further owner confirmation:

1. create, populate and later remove the campaign root `~/perfeval002/` inside
   the guest (transferred binaries, Pi package, campaign runtime root, evidence);
2. start and stop **campaign-owned** `kernel-server` processes bound to
   loopback `127.0.0.1:48282` with the campaign runtime root;
3. start and stop **campaign-owned** Pi processes under campaign-scoped XDG
   roots;
4. install Pi `0.81.1` from the official npm origin into the campaign root with
   integrity verification;
5. read-only observation of guest platform, process, resource and product
   surface facts;
6. delete campaign-created state during cleanup.

Requires stopping and asking the owner first:

1. any libvirt snapshot create, revert or delete, including
   `b01-platform-qualified-baseline`;
2. any guest power-state change (start, shutdown, reset, destroy);
3. any modification to, or shutdown of, the residual `~/p9t04` state or its
   running daemon;
4. system-wide changes: `apt`, systemd units, users, network configuration;
5. any credential entry into the guest keyring, any change to an existing
   keyring entry, and any read, copy, hash or relocation of owner Provider key
   material.

## 5. Secret boundary

Unchanged from the axioms and plan §8.2. Specifically for this campaign:

- the campaign performs **no** credential entry and **no** credential import.
  Whether a usable Provider credential already exists in the guest's approved
  SecretStore is determined in `B0` by product-surface behaviour only
  (does a Provider request succeed or fail closed), never by reading, listing
  values from, or hashing the keyring;
- no key material may appear in argv, environment, configuration, SQLite, logs,
  evidence, this document, or chat;
- if the Provider path is not configured, Provider-dependent cells are recorded
  `not-run` with reason `credential path not available to a non-interactive
  session`, and the campaign continues with non-Provider cells. Requesting
  owner credential entry is an owner-only boundary and is not attempted
  silently.

## 6. Denominators, retention and retry

- every started sample is retained, including timeout, denial, refusal,
  rate-limit, unknown outcome and manual intervention;
- warmups are excluded before a cell begins and are never reclassified
  afterwards;
- `retry = 0` for every Provider-touching request;
- a cell's sample count is fixed before it starts; no optional stopping, no
  outlier deletion, no "close to significant" top-up;
- each cell is recorded `pass` / `fail` / `partial` / `not-run` /
  `not_available` with its denominator, in [§10](#10-execution-record).

Because no paired arm exists, no clustered bootstrap, McNemar test, or Holm
correction from plan §7.1 is applicable. Percentile reporting follows plan §7.1
tail discipline: median/MAD/min/max always; p95 only where N >= 100; no p99
claim anywhere in this campaign.

## 7. Preregistered cell register

Order follows plan §9, skipping batches whose arm is `not-run`.

| # | Cell | Instrument | Planned denominator | Preregistered status |
|---|---|---|---|---|
| 1 | `B0` qualification | staging + public surface | n/a | to execute |
| 2 | `D` Provider route marker | existing `p9-t04-l3-provider-route-runner.mjs` | 30 | to execute if Provider configured |
| 3 | `D` warm repeated route | same runner | 50 | to execute if Provider configured |
| 4 | `O` Pi route first response | existing `p1-t09-product-route-smoke.sh` | 30 | to execute if Provider configured |
| 5 | `UJ4` / `O1` Task admission truth | existing `p9-t04-l4-t1-scenario-runner.mjs` | 30 | to execute |
| 6 | `UJ3` daily operations | public-surface observation | plan §5.3 counts | to execute |
| 7 | `T-GOV` Tool projection + lifecycle | public-surface observation | 1 projection + 1 lifecycle round | to execute |
| 8 | `MS-AUTH` Memory/Skill authority smoke | public-surface observation | 10 + 10 + negatives | to execute |
| 9 | `B3` model mismatch | existing route runner `--model-override` | 10 | to execute if Provider configured |
| 10 | `B3` bounded client deadline | existing route runner `--request-timeout-ms` | 10 | to execute |
| 11 | `B3` daemon restart / cleanup residue | public-surface observation | 10 cycles | to execute |
| 12 | `B3` Pi process kill | public-surface observation | 10 | to execute if Pi installed |
| 13 | `B4` concurrency 1/8/16 + bounded overload | public-surface observation | 100 local reads per profile | to execute |
| 14 | `B5` 1 h soak | public-surface observation | 60 min | conditional on budget and prior cells |
| 15 | `B1` pilot, `B2` confirmatory paired | — | — | **`not-run`** (no `P` arm, no paired runner) |
| 16 | `A3`/`A6`/`A7`, `G*-C1/C2`, `A1-C1` | — | — | **`not-run`** (no OS product path) |
| 17 | `S4`/`S8`, `T4`–`T9` | — | — | **`not-run`** (no governed consumer / no production caller) |
| 18 | `O2`/`O3` Context correctness and cache | — | — | **`not_available`** (no public observation surface) |
| 19 | `O4`/`O5`/`O6` scheduler, Effect, verifier | — | — | **`not-run`** (production path absent) |
| 20 | `O14` backup/restore | — | — | **`not_available`** (no user CLI/archive wiring) |
| 21 | `B5` 8 h, `B5` 24 h, `B6` replay | — | — | **`not-run`** (gated on prior exits / post-optimization) |

## 8. Safety hard conditions

Plan §6.8's eight counters are collected for every cell with an explicit
evidence disposition (`observed_zero`, `not_applicable`, `not_available`,
`observed_nonzero`). A structure default or a runner-hardcoded `0` never
produces `observed_zero`. In a campaign with no mutation path and no verifier
caller, most of these are expected to be `not_applicable` or `not_available`,
and saying so is the honest result.

## 9. Evidence, abort and cleanup

Raw payloads live only in the ignored root
`artifacts/performance/PERSONAL-PERF-EVAL-002/` on the operator host and in the
guest campaign root; Git receives only redacted aggregates, digests and this
record. Abort conditions are plan §6.8 non-zero counters, manifest mismatch,
sensitive evidence capture, or any guest change outside §4 — on any of these the
campaign preserves the samples, records the fact, and stops claim promotion
rather than deleting and rerunning.

Cleanup removes the campaign root and campaign processes, checks for orphan
processes, sockets, locks and residue, and leaves the guest exactly as found in
§2.2, including the untouched snapshot and the untouched P9-T04 residue.

## 10. Execution record

Appended per cell as the campaign proceeds. Each entry states the cell, the
instrument, the exact revision, the started/retained denominator, the outcome
classes, the measurement, and the safety dispositions.

### 10.0 Freeze, 2026-08-12

Source frozen at `4cbec8470bc7a19f23f978e8754ed20133122eb1`; environments,
arms, allowlist, secret boundary, denominators and the cell register above are
fixed. Independent review is `not_available`; the claim ceiling stays
`hypothesis`. No guest state had been changed at the time of freeze.

### 10.1 `B0` staging and qualification, 2026-08-12 — `pass`

Release build of the frozen revision on `DEV-LINUX-NATIVE-01`
(`cargo build --release --locked -p kernel-server -p admin-cli -p pi-agent-adapter`,
finished in 1 m 33 s; `pnpm install --frozen-lockfile` + `pnpm -r build`):

| Artifact | SHA-256 |
|---|---|
| `kernel-server` | `05b7411447903fa23c8bebbbd17513a5bbf4cd3e6eb40bec3d24c129ac15b644` |
| `admin-cli` | `622f6a7832c4fa7810bff947b0e2b9f6afd8f58b4ecabe38ef36bcc17993f462` |
| `cognitive` | `0bebe19668a0ccb02374baef54cc5f1689df89bafc22a7d7323ee848a97ccf43` |
| `pi-agent-adapter` | `cf65530d16980ec6840e1096f4d15605fd9705ccfed8521db70be1958a76869c` |

66 files (binaries, the unmodified `tools/personal` runners and the built
`packages/pi-cognitiveos/dist`) were transferred to the guest campaign root
`~/perfeval002` and verified with `sha256sum -c`: **all 66 matched**.

A campaign-owned daemon was started inside the allowlist at
`127.0.0.1:48282` with runtime root `~/perfeval002/runtime` and the frozen
`kernel-server` (pid 20981). The non-secret Provider descriptor
(`provider.json`: `base_url`, `provider_id`, `schema_version`, `secret_ref`,
`selected_snapshot_digest`) and `selected-model.json` were copied from the
residual P9-T04 config so the daemon could resolve the owner's own SecretStore
entry itself. **No credential was entered, read, copied or hashed at any point**,
and the residual P9-T04 root and its daemon were not modified.

`B0` exit criteria are met for the executable arms: exact-revision staging
verified by digest, campaign daemon reaching `overall: ready`, and the
Provider/secret boundary observed by product behaviour only.

### 10.2 `D` arm Provider route — cells `D1` and `D2`, 2026-08-12 — `partial`

Instrument: the unmodified frozen
`tools/personal/p9-t04-l3-provider-route-runner.mjs`. Three warmups were run
first and discarded before either cell began.

| Cell | Started | Retained | Outcome classes | Registered code | Marker | Usage |
|---|---:|---:|---|---|---:|---|
| `D1` provider-proxy marker | 30 | 30 | `outcome_unknown` 30 | `PERSONAL_PROVIDER_SECRET_UNAVAILABLE` | 0/30 | 0/30 measured |
| `D2` warm repeated | 50 | 50 | `outcome_unknown` 50 | `PERSONAL_PROVIDER_SECRET_UNAVAILABLE` | 0/50 | 0/50 measured |

No Provider network latency was measurable in either cell, so the campaign's
route-performance objective for the `D` arm is **not met**. What the 80 retained
samples do measure is the fail-closed path: `D1` total 109.30 ms p50 /
126.33 ms p95, `D2` total 115.73 ms p50 / 134.49 ms p95, `retry = 0`, zero
Provider dispatches, no plaintext fallback. Evidence digests
`sha256:b827246957925fe1b6a0d477b213e2f6056c6034c66ff06880fcb09b1e23ed26` (D1)
and `sha256:f48e983d27987f09ec238821ad67ef46387f0449943d20b77e911c9ff8f6c302`
(D2); payloads stay in the ignored campaign root.

#### Attribution of the failure (Secret Service, paths only)

The cause was determined without requesting, reading or hashing any secret
value, using `org.freedesktop.Secret.Service.SearchItems`, which returns object
paths only:

| Probe | Result |
|---|---|
| `secret-tool` binary | present (`/usr/bin/secret-tool`) |
| collections | `session`, `login` |
| `login` collection `Locked` | `false` (unlocked) |
| SearchItems `{application, provider, purpose}` | `[]` unlocked, `[]` locked |
| SearchItems `{application: cognitiveos-personal}` | `[]` unlocked, `[]` locked |
| campaign daemon env has `DBUS_SESSION_BUS_ADDRESS` / `XDG_RUNTIME_DIR` | yes / yes |

The Provider API key item **does not exist** in the guest Secret Service. This
is not a lock, a D-Bus, a namespace, or a runtime-root problem: the P9-T04
cleanup removed its campaign SecretStore entry as its policy required, while
`provider.json` — which references it — was left behind. Restoring it requires
owner graphical hidden input, which is an owner-only boundary a non-interactive
session must not attempt.

Consequence, applied without pausing the campaign per Operating Model §2.5:
every Provider-dependent cell (`D1`, `D2`, `O` Pi first response, `B3` selected-
model mismatch as a dispatch-suppression test, and any Provider-touching soak
block) is `not-run` for its latency objective, with reason **credential path
not available to a non-interactive session**.

#### Findings this cell produced

1. **Projection honesty defect (product-relevant).** With the referenced secret
   absent, `cognitive status` and `cognitive doctor` both report
   `secret: ready`, `provider: ready` and `overall: ready`. The `provider`
   component's source is `filesystem:provider-config` and it asserts only
   `secret_ref_present: true`; the `secret` component's source is
   `secret-store:probe` and it asserts only backend availability. Neither
   validates that the configured reference resolves, so a dangling `secret_ref`
   is reported as ready and then fails on every real request. This is plan §12's
   `registered` versus `execution-ready` drift row, observed on a real dangling
   reference rather than inferred.
2. **Fail-closed behaviour is correct (safety-positive).** 80 / 80 requests were
   refused with a registered code, no Provider dispatch occurred, no plaintext
   fallback was attempted, and the refusal was bounded near 110–116 ms p50.
3. **Measurement-side classification gap.** The frozen runner maps
   `PERSONAL_PROVIDER_SECRET_UNAVAILABLE` to `outcome_unknown` because that code
   is absent from its `DENIED_BEFORE_DISPATCH_CODES` set, although the product
   demonstrably refused before dispatch. The product is correct and the runner
   is imprecise. Under the measurement-only boundary the runner was **not**
   modified; the raw classification is retained above and corrected only in
   interpretation.

### 10.3 `UJ4` / `O1` Task admission truth, 2026-08-12 — `pass`

Instrument: the unmodified frozen
`tools/personal/p9-t04-l4-t1-scenario-runner.mjs`, 30 unique read-only Tasks
against the campaign daemon.

| Fact | Value |
|---|---:|
| Started / retained runs | 30 / 30 |
| Admitted | **30 / 30** |
| Terminal error codes | none (`null` × 30) |
| Executed mutations | 0 |
| Independent acceptance | 0 / 30 |
| Verified completions claimed | **0** |

| Stage | p50 | p95 (descriptive) | min | max | MAD |
|---|---:|---:|---:|---:|---:|
| session mint | 3.33 ms | 7.85 ms | 2.07 ms | 79.70 ms | 0.30 ms |
| `intent.record` | 14.07 ms | 25.20 ms | 11.69 ms | 26.54 ms | 1.96 ms |
| `intent.interpret` | 16.29 ms | 21.89 ms | 11.14 ms | 22.05 ms | 2.11 ms |
| `task.preview` | 8.32 ms | 13.66 ms | 5.62 ms | 15.01 ms | 0.48 ms |
| `task.admit` | 25.92 ms | 30.35 ms | 20.91 ms | 33.14 ms | 1.56 ms |
| **admission total** | **68.78 ms** | 86.07 ms | 60.45 ms | 154.69 ms | 3.78 ms |

At N = 30 the p95 column is descriptive only; plan §7.1 requires N >= 100 for
tail inference and no p99 is claimed. Admission is Provider-independent, which
is why this cell ran to completion while the Provider cells did not.

Two facts must stay separate, per plan §11 item 8: **30 / 30 admitted is not
30 / 30 completed.** Independent acceptance occurred in no run and the runner
claims zero verified completions, because the scheduler, Context, Tool, Effect
and verifier chain has no production caller. `admitted` measures the front half
of the Task contract only.

Against the P9-T04 baseline (10 runs, 8 admitted, 68.17 ms p50 / 140.77 ms p95)
the p50 is unchanged at 68.78 ms and the two admission rejections are gone —
consistent with the P9-T04 note that its rejections came from a Task draft that
did not match the generated `Budget` binding, not from the product. The tail
looks better (86.07 against 140.77 ms) but the denominators differ (30 against
10) and neither supports tail inference.

### 10.4 `UJ3` daily operations, 2026-08-12 — `pass`

Instrument: public-surface observation (documented HTTP routes and CLI verbs),
three warmups per surface discarded before any counted sample.

| Operation | N | p50 | MAD | min | max | p95 | Outcomes |
|---|---:|---:|---:|---:|---:|---:|---|
| `GET /personal/health` | 200 | 0.61 ms | 0.09 ms | 0.46 ms | 1.26 ms | 0.85 ms | 200 × 200 |
| `cognitive status` (CLI) | 100 | 70.77 ms | 6.83 ms | 57.73 ms | 92.21 ms | 88.04 ms | 100 × exit 0 |
| `cognitive doctor` (CLI) | 50 | 71.00 ms | 7.98 ms | 52.94 ms | 97.36 ms | n/a | 50 × exit 0 |
| `cognitive daemon status` (CLI) | 50 | 5.05 ms | 0.41 ms | 3.98 ms | 6.64 ms | n/a | 50 × exit 0 |
| six-resource `GET` × 6 families | 50 each | 0.65–0.87 ms | ≤0.13 ms | 0.51 ms | 2.19 ms | n/a | 300 × 200 |
| bounded watch/replay × 6 families | 10 each | 0.57–0.86 ms | ≤0.16 ms | 0.51 ms | 1.06 ms | n/a | 60 × 200 |
| task-bound resource watch | 20 | 0.95 ms | 0.08 ms | 0.72 ms | 1.81 ms | n/a | 20 × 200 |

p95 is reported only where N >= 100 (plan §7.1); no p99 anywhere.

**Channel isolation holds, measured rather than asserted.**
`GET /resource/v1/projection` answered `401` unauthenticated, `403` with a
task-channel bearer, and `200` with a management-channel bearer.
`/personal/status` and `/personal/doctor` answered `401` for all 150
unauthenticated calls, so those two rows measure the bounded rejection path
(0.68–0.79 ms p50), not the operation.

**The daemon is not where daily-operation latency lives.** Every in-daemon
read answers in well under 1 ms, while the same information through the CLI
costs 70 ms — roughly two orders of magnitude, dominated by process start plus
the readiness evaluation rather than by the daemon's work.

### 10.5 `T-GOV` Tool projection truth, 2026-08-12 — `partial`

One projection dump against the campaign daemon
(`authority_source: daemon-native-tool-registry`, `availability: available`):

| Tool family | availability | execution_readiness | risk | descriptor digest |
|---|---|---|---|---|
| `workspace_read` | enabled | **execution_ready** | read_only | present |
| `workspace_search` | enabled | registered_only | read_only | present |
| `workspace_write` | enabled | registered_only | workspace_mutation | present |
| `workspace_patch` | enabled | registered_only | workspace_mutation | present |
| `process_check` | enabled | **execution_ready** | process_execution | present |
| `http_fetch_read_only` | enabled | registered_only | network_read | present |

Registered 6, execution-ready 2, registered-only 4. **The projection is honest**:
it reports exactly the two executors the plan's §1.1 capability table says are
implemented and declares the other four `registered_only` rather than implying
they can be called. This is the P2-T09 readiness separation working on the real
surface, and it is the direct counter-example to the Provider `ready` defect in
§10.2 — the honesty problem there is specific to the Provider/secret readiness
component, not a systemic projection failure.

The dynamic lifecycle half is `not_available`: `tool/enable`, `tool/disable`,
`tool/quarantine` and `tool/discover` under `/management/resource/v1/` all fall
through to the generic authority handler and answer
`RESOURCE_OBJECT_ID_REQUIRED`, so no enable/disable/quarantine propagation could
be driven from the public surface. The cell is therefore `partial`, and no live
ecosystem claim is made.

### 10.6 `MS-AUTH` Memory/Skill authority smoke, 2026-08-12 — `partial`

Instrument: public-surface observation against `/management/resource/v1/`,
10 Skill lifecycle rounds and 10 Memory rounds, all with a management bearer.

**Negative controls — 6 / 6 fail closed with precise registered codes:**

| Negative | Status | Registered code |
|---|---:|---|
| bind to an unknown revision | 409 | `RESOURCE_SKILL_CONFLICT` |
| revoke an unknown binding | 400 | `RESOURCE_SKILL_ID_INVALID` |
| malformed object id | 400 | `RESOURCE_OBJECT_ID_INVALID` |
| task channel drives a management mutation | 403 | `SHELL_CHANNEL_BINDING_MISMATCH` |
| unauthenticated management mutation | 401 | `LOCAL_SESSION_UNAUTHORIZED` |
| forget without a reason | 400 | `RESOURCE_MEMORY_REASON_REQUIRED` |

**Positive lifecycle — 0 / 40 writes admitted:**

| Operation | N | Status | Registered code | p50 |
|---|---:|---:|---|---:|
| `skill.import` | 10 | 409 | `RESOURCE_SKILL_CONFLICT` | 0.79 ms |
| `skill.bind` | 10 | 409 | `RESOURCE_SKILL_CONFLICT` | 0.86 ms |
| `skill.supersede` (second revision) | 10 | 409 | `RESOURCE_SKILL_CONFLICT` | 0.78 ms |
| `skill.binding.revoke` | 10 | 400 | `RESOURCE_SKILL_ID_INVALID` | 0.78 ms |
| `skill.binding.explain` | 10 | 404 | `RESOURCE_SKILL_BINDING_NOT_FOUND` | 0.91 ms |
| `memory.remember` | 10 | 409 | `RESOURCE_MEMORY_CONFLICT` | 0.85 ms |
| `memory.review` | 10 | 404 | `RESOURCE_MEMORY_NOT_FOUND` | 0.81 ms |
| `memory.forget` | 10 | 409 | `RESOURCE_MEMORY_CONFLICT` | 0.84 ms |
| `memory.forget` non-resurrection | 10 | 404 | `RESOURCE_MEMORY_NOT_FOUND` | 0.80 ms |

Nothing was admitted, so `forget non-resurrection` and `revoked reuse` are
vacuously satisfied and are recorded as **`not_applicable`, not `observed_zero`**
— there was no admitted object to resurrect or reuse.

#### Route shadowing defect, confirmed by differential probe

`POST /management/resource/v1/skill/binding/revoke` never reaches the revoke
handler. The dispatcher in `resource_api.rs` tests
`starts_with(".../skill/bind")` before `starts_with(".../skill/binding/revoke")`,
and `"skill/bind"` is a prefix of `"skill/binding/revoke"`, so every revoke is
handled by `bind_skill`. Confirmed differentially rather than inferred: posting
to the revoke path with a *bind-shaped* payload returns the bind handler's own
message, `"Skill binding conflicts with existing authority facts"`, byte-identical
to a plain `POST .../skill/bind`. Skill binding revoke is therefore unreachable
on the public management surface at this revision.

#### Undiscriminating conflict vocabulary

Every store-level rejection — rich payload, minimal payload, Memory or Skill —
returns the same generic 409 with no discriminating detail:
`"Skill import conflicts with existing authority facts"` /
`"Memory admission conflicts with existing authority facts"`. Validation errors
are precise (six distinct codes above); transition-gate rejections collapse to
one code per family.

**Honest limit of this cell.** These observations do **not** establish that
Memory/Skill authority is broken. The deep per-operation negative matrix is
owned by the B08 Gate and the merged CI regressions, which this campaign does
not re-run or override. What is established is narrower and still useful: from
the product's own public management surface, using payloads derived from that
surface's own route contract, no Memory or Skill lifecycle write could be
admitted, and the surface returns no information that would let an operator
discover why. Determining the missing precondition would require reading the raw
authority store or changing the product, both of which are out of bounds
(plan §5.4, Operating Model §2.5).

### 10.7 `UJ3b` readiness cost decomposition, 2026-08-12 — `pass`

The `UJ3` result (0.6 ms in-daemon reads against a 70 ms CLI `status`) had two
candidate explanations. Both were measured instead of assumed.

| Path | N | p50 | p95 |
|---|---:|---:|---:|
| `GET /personal/health` (in-daemon, trivial) | 200 | 0.61 ms | 0.85 ms |
| `GET /personal/status` authenticated (in-daemon readiness) | 100 | 64.86 ms | 84.85 ms |
| `cognitive status` (CLI → same evaluation) | 100 | 70.77 ms | 88.04 ms |
| `GET /personal/doctor` authenticated | 50 | 69.32 ms | n/a |
| `cognitive doctor` (CLI) | 50 | 71.00 ms | n/a |
| `cognitive daemon status` (CLI, local files, no HTTP) | 50 | 5.05 ms | n/a |

CLI process start costs about 5–6 ms (`70.77 − 64.86`, corroborated by the
5.05 ms file-only `daemon status`). The readiness evaluation itself owns the
remaining ~65 ms.

Component attribution over 30 authenticated `status` samples, read from the
product's own per-component `duration_ms`:

| Component | N | median | min | max |
|---|---:|---:|---:|---:|
| `secret` | 30 | **57.5 ms** | 44 ms | 76 ms |
| `system` | 30 | 0 ms | 0 | 0 |
| `database` | 30 | 0 ms | 0 | 0 |
| `provider` | 30 | 0 ms | 0 | 1 ms |
| `daemon` | 30 | 0 ms | 0 | 0 |
| `pi` | 30 | 0 ms | 0 | 0 |

**The Secret Service probe is essentially the entire cost of the product's
most-used diagnostic**, and every other component is free. Combined with §10.2,
the same 57.5 ms probe reports `secret: ready` without validating that the
configured `secret_ref` resolves — the most expensive check in the readiness
path is also the one that missed the actual defect.

### 10.8 `B3` fault, restart and cleanup, 2026-08-12 — `partial`

| Sub-cell | Started | Retained | Result |
|---|---:|---:|---|
| selected-model mismatch | 10 | 10 | **10/10 `denied_before_dispatch`**, `PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH`, zero Provider dispatches, 36.48 ms p50 (26.03–52.06 ms) |
| bounded client deadline 20 ms | 0 | 0 | **`not-run`** — the frozen runner applies the deadline to its own `fetchSelectedModel` preflight, so it aborted before starting a sample |
| bounded client deadline 120 ms | 10 | 10 | 10/10 retained, all `PERSONAL_PROVIDER_SECRET_UNAVAILABLE`, 101.59 ms p50, 80.13–125.35 ms |
| daemon stop/start cleanup | 10 | 10 | see below |
| daemon unavailable | 10 | 10 | 10/10 connection refused in 0.23 ms p50 (0.21–0.43 ms) |
| Pi process kill | 0 | 0 | **`not-run`** — no Provider credential, so no real Pi conversation exists to interrupt |
| Provider upstream timeout, rate limit, response-size bound, stale epoch, `OUTCOME_UNKNOWN` | 0 | 0 | **`not-run`** — require a Provider credential or a reachable mutation path |

The selected-model mismatch cell is the one Provider-adjacent cell that *is*
meaningful without a credential: the daemon compares the requested model against
the selected model **before** resolving secret material, so the denial is
genuine and the zero-dispatch property is real rather than incidental.

The 120 ms deadline cell is reported as retained samples, not as a clean bound:
the maximum reached 125.35 ms, slightly past the client deadline, and the failure
was the secret path rather than an upstream stall. Note that the fail-closed path
pays the full ~57 ms keyring probe before refusing, which is why refusal costs
~100 ms rather than microseconds.

**Restart and cleanup, 10 cycles:**

| Fact | Result |
|---|---|
| `daemon stop` | 9.81 ms p50 (8.88–13.15 ms), exit 0 × 10 |
| `daemon start` (spawn + the CLI's own readiness wait) | 105.06 ms p50 (85.03–105.55 ms) |
| residual wait to first `200 /personal/health` after start returned | 1.73 ms p50 |
| orphan process after stop | **0 / 10** |
| stale `daemon.lock` after stop | **0 / 10** |
| stale `daemon-endpoint.json` after stop | **0 / 10** |
| health while down | connection refused, every cycle |
| daemon RSS across 10 cycles | 8828–9084 kB, no upward trend |
| FD count / threads across 10 cycles | constant 9 / constant 1 |

Teardown is clean on every cycle: no orphan, no stale lock, no stale endpoint
file, no RSS or descriptor growth. The P9-T04 figure of 182.6 ms
startup-to-ready is not directly comparable, because it timed a different
boundary; the comparable figure here is roughly 105 ms for the whole
`daemon start` command.

### 10.9 `B4` concurrency and bounded overload, 2026-08-12 — `pass`

100 local reads per profile (132 for the 33-connection profile), all retained.

| Profile | Concurrency | N | p50 | p95 | max | throughput | non-200 |
|---|---:|---:|---:|---:|---:|---:|---:|
| health | 1 | 100 | 0.70 ms | 0.91 ms | 3.07 ms | 1116.6 rps | 0 |
| health | 8 | 100 | 6.28 ms | 10.85 ms | 14.02 ms | 1146.4 rps | 0 |
| health | 16 | 100 | 7.61 ms | 17.51 ms | 20.80 ms | 1293.0 rps | 0 |
| tool projection | 1 | 100 | 0.63 ms | 0.80 ms | 2.04 ms | 1365.5 rps | 0 |
| tool projection | 8 | 100 | 5.43 ms | 9.32 ms | 11.15 ms | 1320.0 rps | 0 |
| tool projection | 16 | 100 | 8.23 ms | 19.56 ms | 23.55 ms | 1233.8 rps | 0 |
| bounded overload, 17 in-flight | 17 | 100 | 7.40 ms | 19.70 ms | 22.23 ms | 1390.3 rps | 0 |
| bounded overload, 33 connections | 33 | 132 | 6.36 ms | 18.68 ms | 20.97 ms | 1289.9 rps | 0 |
| health after overload | 1 | 100 | 0.51 ms | 0.64 ms | 2.02 ms | 1657.0 rps | 0 |

832 requests, **zero errors, zero refusals, zero dropped connections** across
every profile including 33 concurrent connections.

The shape is unambiguous and is explained by a fact the sampler read directly
from `/proc`: the daemon runs on **one thread** throughout. Throughput is flat
near 1.1–1.4 k local reads per second regardless of offered concurrency, while
p50 grows almost linearly with concurrency (0.70 → 6.28 → 7.61 ms for 1 → 8 →
16). That is queueing behind a single-threaded server, not resource exhaustion:
concurrency buys no throughput and converts directly into waiting time.

Degradation is graceful and recovery is immediate: after the 33-connection
overload, health returned to 1.5 ms within 0.74 ms of the first probe, and the
following serial profile measured 0.51 ms p50 — faster than the pre-overload
baseline. RSS moved from 9228 kB to 9820 kB across the whole B4 sequence
(~600 kB over 832 requests) with FD constant at 9 and threads constant at 1.

p95 is reported because each profile has N >= 100 (plan §7.1). No p99 is
claimed anywhere.
