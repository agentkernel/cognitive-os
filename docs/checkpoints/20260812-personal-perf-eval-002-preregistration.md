# PERSONAL-PERF-EVAL-002 preregistration and freeze record

- Status: **frozen; phases 1 and 2 complete; campaign closed 2026-08-12**
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

**Positive lifecycle — 0 / 60 writes admitted (50 × generic 409, 10 × 400 from
the shadowed revoke route); the 40 reads that followed all returned 404:**

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

### 10.10 `B5` 1 h soak, 2026-08-12 — `pass`

60 one-minute blocks, each 20 health reads plus the six resource projections
plus one bounded watch: **1620 started, 1620 retained, 0 non-200**.

| Slope fact | First minute | Last minute | Delta |
|---|---:|---:|---:|
| daemon RSS | 8820 kB → measured 9820 kB | 9860 kB | **+40 kB / hour** |
| FD count | 9 | 9 | 0 |
| threads | 1 | 1 | 0 |
| `authority.sqlite` | 1 044 480 B | 1 044 480 B | **0** |
| `authority.sqlite-wal` | 0 B | 0 B | 0 |
| process `write_bytes` | 1 347 584 | 1 347 584 | **0** |
| per-minute p50 | 0.677 ms | 1.288 ms | +0.61 ms |
| worst single sample over the hour | — | — | 13.16 ms |

The closing hourly cold restart was clean: `orphan_process: false`,
`stale_lock: false`, `stale_endpoint: false`, `restart_ready: true`.

No leak signature of any kind: memory, descriptors, threads, database and WAL
are all flat, and the daemon performed literally zero additional writes over an
hour of read traffic. The p50 drift from 0.68 ms to 1.29 ms is sub-millisecond
and non-monotonic across the run, so it is reported as observed variation rather
than a trend. Paired Provider soak blocks are `not-run`; `B5` 8 h and 24 h are
`not-run` per the plan's promotion gates.

### 10.11 Added cell `O-LAUNCH`, 2026-08-12 — `pass` (as a launch observation)

**Disclosure:** this cell was **added during execution** and is not the
preregistered `O` Pi-first-response cell, which is `not-run` for want of a
Provider credential. Its denominator (10) was fixed before it started, it
retains every started sample, and it is explicitly labelled a launch/failure
observation rather than a first-response latency result. It is not an arm in any
comparison.

Pi provenance in the campaign root: `@earendil-works/pi-coding-agent` `0.81.1`,
lock-resolved from `registry.npmjs.org`, integrity
`sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==`
— byte-identical to the pin P9-T04 recorded.

**Readiness before and after configuring Pi, with the Provider credential still
absent:**

| Moment | `overall` | `first_conversation_ready` | components |
|---|---|---|---|
| before `pi configure` | `ready` | `false` | pi `not_configured`, all others `ready` |
| after `pi configure` | `ready` | **`true`** | **all six `ready`** |

This is the sharpest form of the §10.2 defect. With no Provider key in the
Secret Service at all, the product declares every component ready *and*
`first_conversation_ready: true`. The frozen route smoke's own gate — which
refuses to run unless `overall == ready && first_conversation_ready == true` —
passed, and then the conversation failed.

**10 runs through the frozen `p1-t09-product-route-smoke.sh`:**

| Fact | Value |
|---|---:|
| Started / retained | 10 / 10 |
| Outcome | `error / first_response / pi_nonzero_exit` × 10 |
| Expected marker observed | 0 / 10 |
| Response received (non-empty output) | 10 / 10 |
| Pi invocation | **18 067 ms p50** (17 941–18 500 ms) |
| Full smoke wall time | 21 600 ms p50 (21 476–22 110 ms) |

**Spawn baseline, 10 samples each:** bare `node -e ''` 44.5 ms p50 (39–80 ms);
`pi --version` — Node start plus Pi CLI init, no Extension, no daemon, no
Provider — **1682.5 ms p50** (1518–1814 ms).

Decomposition discipline (plan §5.2): there is no nested timing inside a single
run, so the interval between the 1.68 s bare Pi start and the 18.07 s
route-to-failure is reported as an **unattributed incremental delta of roughly
16.4 s**. It must not be named spawn cost, Extension load, or governance cost.
Candidate contributors include Extension load, daemon discovery and whatever Pi
itself does when a provider call fails — the daemon-side refusal is only ~110 ms
(§10.2), so the daemon accounts for well under 1 % of it.

This is **time-to-failure on a broken Provider**, not first-response latency.
It is not comparable to P9-T04's 4625 ms first response, which was measured on a
working Provider.

### 10.12 Cleanup, secret scan and boundary reconciliation, 2026-08-12 — `pass`

| Check | Result |
|---|---|
| campaign `kernel-server` processes after stop | **0** |
| campaign Pi processes after stop | **0** |
| stale `daemon.lock` / `daemon-endpoint.json` | none / none |
| listener on 48282 | none |
| bootstrap secret file mode | `600` |
| Secret Service entries for `application=cognitiveos-personal` | **`[]`** — the campaign created none |
| key-shaped scan hits inside `evidence/` or `runtime/` | **0** |

The repository-root scan reported 10 files matching `sk-[A-Za-z0-9_-]{16,}`,
which was triaged by path and character class without printing any matched text.
All were pure-alphabetic hyphenated identifiers — mask `AA-AAAAAAAA-AAAAAAAAA`,
containing **no digits at all** — inside vendored OpenTelemetry
`experimental_metrics` constants under the Pi package and the frozen runner's
own filename. No API key shape exists anywhere in the campaign root.

Boundary reconciliation against the §4 allowlist: the residual P9-T04 daemon is
still running unchanged as pid 11176; the P9-T04 config mtimes are unchanged at
12:51 and 13:37, hours before this campaign began; no snapshot was created,
reverted or deleted; no guest power-state change occurred; no system
configuration was touched. **Scenario boundary violations: 0.**

**Evidence retention (plan §8.3).** Raw payloads are retained rather than
deleted, because a digest without a retrievable payload cannot support later
review. Locator: `b01guest:~/perfeval002/evidence/` on `B01-Desktop-Linux-002`,
17 files, 156 KiB, retained until the independent verifier disposition changes
from `not_reviewed`. Per-file SHA-256 digests were recorded at cleanup time.

### 10.13 Phase-1 closure, 2026-08-12

All cells executable under the phase-1 boundary have run. Phase-1 disposition is
published in
[personal-performance-assessment-20260812.md](../evaluation/personal-performance-assessment-20260812.md).
Claim level `hypothesis`, independent verifier `not_reviewed`, Agent benefit not
claimed, no Gate/release/Profile/B01 promotion.

## 11. Phase 2 — owner scope change, 2026-08-12

The owner granted highest authorization to act on the phase-1 recommendations
and complete all tests. This lifts the two phase-1 blockers and is recorded here
as a deliberate scope change, not a quiet reinterpretation.

- Lease: `lease/personal/EVAL-20260812/performance-evaluation-002-phase2`
- Unchanged: source revision `4cbec8470bc7a19f23f978e8754ed20133122eb1`,
  environment, guest allowlist, denominator and retention discipline, claim
  ceiling `hypothesis`, verifier `not_reviewed`.
- **Still unchanged and non-negotiable:** no product code, contract, negative,
  test or generated documentation source may be modified. The phase-2
  instruments (broker, corpus, paired runner) are campaign-only measurement
  tools living solely in the ignored `artifacts/` root and the guest campaign
  root. Nothing under `crates/`, `apps/`, `packages/`, `specs/`, `conformance/`
  or `handbook/` is touched.

### 11.1 Provider credential import — `pass`

Operating Model §2.3 already authorizes importing an owner-designated local
Provider key into an approved Secret Store, so this needed no new permission
once the owner widened the campaign scope.

The owner's own file `~/下载/deepseek.txt` was located by name, and inspected by
**shape only** — line count, line lengths, and a character-class mask. That
inspection showed the file is the owner's own saved `cognitive init` invocation:

| Line | Content class |
|---|---|
| 2–3 | the `cognitive` binary path and `--runtime-root` |
| 4 | `--provider deepseek` |
| 5 | `--base-url https://api.deepseek.com/v1` |
| 6 | `--model-id deepseek-v4-flash` |
| 8 | a 35-character `sk-`-prefixed key |

The campaign therefore invented no Provider parameters: provider id, base URL
and model came from the owner's own recorded intent, and they match the
non-secret config already present in the campaign runtime.

Import used the product's own stdin path,
`sed -n '8p' … | cognitive init … --api-key-file -`. The key travelled
file → pipe → product stdin → SecretStore and never entered argv, environment,
ordinary config, logs, evidence or chat. Result: `secret_material_written: true`,
backend `linux-secret-tool`, Secret Service item now present at
`/org/freedesktop/secrets/collection/login/7`, owner source file untouched
(mtime unchanged at `12:51:02`).

### 11.2 `D1` / `D2` re-run against a working Provider — `pass`

| Cell | Started | Retained | Outcomes | Marker | Usage |
|---|---:|---:|---|---:|---|
| `D1` proxy marker | 30 | 30 | `complete_response` 30 | 30/30 | measured 30/30 |
| `D2` warm repeated | 50 | 50 | `complete_response` 50 | 50/50 | measured 50/50 |

| Measurement | `D1` | `D2` | P9-T04 baseline |
|---|---:|---:|---:|
| Provider network p50 | 985.81 ms | 922.91 ms | 898.9 / 1016.1 ms |
| Provider network p95 | 1404.27 ms | 1229.73 ms | 1224.6 / 1400.2 ms |
| local governance + loopback residual p50 | **127.30 ms** | **128.48 ms** | **126.5 / 128.5 ms** |
| residual MAD | 11.47 ms | 12.22 ms | — |
| residual share of the loopback exchange | 11.51 % | 12.24 % | 11.76 % |

The local residual reproduces the P9-T04 figures to within 1 ms at a different
revision, on a different day, with an independently rebuilt binary. That is the
strongest stability evidence this campaign produced.

### 11.3 `O1` OS Pi first response — `pass`

30 counted runs through the unmodified frozen route smoke, 3 warmups discarded:
**30/30 `ok`**, marker observed 30/30, first response **4625 ms p50 / 5006 ms
p95** (4270–5089 ms, MAD 142.5 ms), full smoke wall 8195 ms p50.

P9-T04 measured 4625 ms p50 / 5004 ms p95. The p50 is identical and the p95
differs by 2 ms.

### 11.4 Pure-Pi credential broker — qualified

Plan §2.2 option 2, implemented as a campaign-only instrument.

| Fact | Value |
|---|---|
| Instrument | `pure-pi-broker.py`, Python standard library only |
| Digest | `sha256:d29bc159254b596d739f4289a3d1ebeb43aa53b01ee424690a5e8fdb456b74be` |
| Bind | `127.0.0.1:48383`, loopback only, single user |
| Key source | read once from the Linux Secret Service into process memory |
| Pi-facing credential | fixed **non-secret** string `campaign-broker-nonsecret-token` |
| Bounds | 1 MiB request, 4 MiB response, 120 s upstream timeout |
| Retry | none |
| Logging | counters, durations, status, byte counts and non-secret request parameters only — never a body, header, key or model output |

Secret-boundary review against plan §2.2, item by item: the key is never written
to disk, never placed in argv or environment, and never echoed; Pi's own
`models.json` holds only the placeholder token, verified by scanning it for
`sk-` (0 matches); the broker performs **no** Context assembly, Tool dispatch,
Memory, Task state, retry, caching or verification, so the `P` arm remains pure
Pi; and its own local latency is recorded separately from upstream time.

Verification: one pure-Pi call returned the expected marker with exit 0 in
3418 ms, broker local overhead **1.48 ms**, upstream 1458 ms.

### 11.5 Frozen corpus and paired runner

| Instrument | Digest |
|---|---|
| `paired_corpus.py` (corpus v1) | `sha256:38e282d4e3ceba0d62768073cf64e27a0e910832ad2ef4bfcca3f2460c919ab1` |
| `paired_runner.py` | `sha256:6b3989520a51aa5c6a59c3d2f2a7dcea0233ad10446c1f4067a275063a69465b` |

Nine C0 families — `G1`, `G2`, `G3`, `G4`, `G6`, `G9`, `A1`, `A4`, `A5` — each
generated from a frozen seed string and each carrying a **mechanical** oracle:
exact number, exact text, sorted id set, or a schedule validated as a
permutation satisfying every dependency edge. No model-as-judge is used
anywhere. Every prompt ends with the identical output contract
(`ANSWER: <value>` as the final line) so the parser is byte-identical across
arms. Each family cycles the three plan §4.3 difficulty layers (`basic`,
`interleaved`, `adversarial`). Pilot and confirmatory seeds are generated from
different stratum strings and therefore cannot overlap (plan §4.7).

**Realized fairness contract (plan §2.3).** Both arms use the same Pi `0.81.1`
binary, the same Node, the same model `deepseek-v4-flash`, byte-identical
prompts, the same tool policy (`--no-tools`), the same discovery suppression
(`--no-extensions`, `--no-skills`, `--no-prompt-templates`, `--no-context-files`,
`--no-themes`, `--no-session`), the same 180 s timeout, `retry = 0`, the same
guest, and arm order randomized inside each block from a frozen seed
(`20260812`). Sampling parameters are not injected by either path: the daemon
proxy forwards the request body **verbatim** (`body: Some(request_body.to_vec())`
in `provider_proxy.rs`), and the broker forwards it verbatim too.

**Two declared arm differences, both consequences of the OS path itself:**

1. the `O` arm's daemon proxy is **non-streaming** by design, while Pi streams
   to the broker in the `P` arm (`stream: true` observed in broker metrics).
   The fair endpoint is therefore time-to-complete-response, and TTFT stays
   `not_available` exactly as plan §6.6 requires;
2. token and cost accounting is observable for `P` (from the broker) but not for
   `O`, because the Extension does not surface per-request usage to the runner.
   Token/cost delta is therefore reported `not_available` rather than estimated.

Harness validation before any counted batch: one block, both arms completed and
both oracles passed, arm order randomized, `P` wall 4853 ms with 0.37 ms broker
local overhead, `O` wall 5567 ms.

### 11.6 `B1` pilot — `pass`

9 families × 5 seeds × 2 replicas = **90 paired blocks, 180 started runs, 180
retained**. No timeout and no process error in either arm.

| Endpoint | `P` pure Pi | `O` OS Pi |
|---|---:|---:|
| oracle completion | 82 / 90 = **91.1 %** | 81 / 90 = **90.0 %** |
| wall time median | **4504.7 ms** | **6182.7 ms** |
| wall MAD | 746.4 ms | 807.5 ms |

- paired completion difference `O − P`: **−1.1 pp**, 95 % clustered bootstrap
  CI **[−3.33, 0.00] pp** (10 000 resamples, clustered on task-seed);
- McNemar exact on discordant pairs (P-only 1, O-only 0): **p = 1.0000**;
- paired wall delta `O − P`: median **+1854.3 ms**, 95 % CI
  **[1643.3, 2007.7] ms**; relative median **+43.3 %**;
- broker local overhead: **0.5 ms** median (0.3–2.0 ms), against the daemon
  path's ~127 ms residual;
- Provider calls per `P` task: median 1, max 2.

Per-family completion was identical in both arms for seven of nine families
(all 100 %). `G6` policy handling sat at 60 % in both arms and `G9` security
review at 60 % `P` versus 50 % `O`. Those two families are what makes the corpus
discriminating: an oracle set where everything passes would measure nothing.

Retained outliers, not deleted (plan §7.1): one `P` run reached 149 880 ms and
produced the paired-delta minimum of −140 305 ms. It stays in every figure
above, which is why medians and MAD are the headline statistics rather than
means.

**Power reading for `B2`.** The completion difference is already bounded inside
±3.3 pp at N = 90 with a single discordant pair, so the confirmatory batch is
sized by the plan's floor (30 paired seeds per family) rather than by a variance
estimate: 9 × 30 = 270 held-out blocks, generated from a different stratum
string so they cannot overlap the pilot seeds. No sample-size change was made in
response to how close anything looked.

### 11.7 `B2` confirmatory held-out paired batch — `pass`

9 families × 30 held-out seeds × 1 replica = **270 paired blocks, 540 started
runs, 540 retained**. No timeout and no process error in either arm.

| Endpoint | `P` pure Pi | `O` OS Pi |
|---|---:|---:|
| oracle completion | 240 / 270 = **88.9 %** | 242 / 270 = **89.6 %** |
| wall median | **4367.2 ms** | **6204.6 ms** |
| wall MAD | 682.8 ms | 804.2 ms |
| wall p95 (N >= 100, reportable) | 23 190.1 ms | 17 408.4 ms |
| wall max | 123 482.0 ms | 133 013.5 ms |

- paired completion difference `O − P`: **+0.7 pp**, 95 % clustered bootstrap CI
  **[−2.22, +3.70] pp** (10 000 resamples, clustered on task-seed);
- McNemar exact, discordant pairs P-only 8 / O-only 10: **p = 0.8145**;
- paired wall delta `O − P`: median **+1828.5 ms**, 95 % CI
  **[1753.6, 1893.9] ms**, relative median **+44.2 %**, delta p95 8061.7 ms;
- broker local overhead: **0.5 ms** median, p95 1.0 ms;
- Provider calls per `P` task: mean 1.01, only 2 of 270 tasks needed two.

Per-family completion:

| Family | `P` | `O` | delta |
|---|---:|---:|---:|
| `A1` root cause | 30/30 | 30/30 | 0.0 pp |
| `A4` operations | 30/30 | 30/30 | 0.0 pp |
| `A5` ambiguity | 24/30 | 27/30 | +10.0 pp |
| `G1` research | 30/30 | 30/30 | 0.0 pp |
| `G2` tabular | 30/30 | 30/30 | 0.0 pp |
| `G3` scheduling | 30/30 | 30/30 | 0.0 pp |
| `G4` procurement | 30/30 | 30/30 | 0.0 pp |
| `G6` policy | 20/30 | 21/30 | +3.3 pp |
| `G9` security review | 16/30 | 14/30 | −6.7 pp |

Six families saturate at 100 % in both arms; `G6` and `G9` are the
discriminating ones. Per-family deltas are secondary endpoints and none would
survive Holm correction across nine families, so they are descriptive only.

Every failure in both arms was a task-quality failure with output present
(`set` mismatch 20 `P` / 19 `O`, `value` mismatch 10 `P` / 9 `O`). There were no
transport, timeout or process failures at all.

Retained outliers, not deleted: `P` max 123 482 ms and `O` max 133 013 ms, with
a paired-delta minimum of −108 501 ms. Medians and MAD are the headline
statistics for exactly this reason.

### 11.8 Attribution of the `O − P` gap — four causes excluded by measurement

The +1828 ms is real and tight (CI ±70 ms). Rather than assert where it comes
from, each candidate was measured.

| Candidate cause | Measurement | Verdict |
|---|---|---|
| CognitiveOS Extension **load** cost | `pi --version` with and without `--extension`, 10 samples each: 1619.0 ms vs 1614.5 ms p50 (MAD 41–56 ms) | **excluded** — 4.5 ms, inside noise |
| Non-streaming OS proxy vs streamed `P` arm | identical prompts through the same broker, `stream=true` vs `false`, 12 pairs, order alternated: medians 2203.9 vs 2164.0 ms | **excluded** — paired median delta **−38.7 ms** |
| Daemon residual scaling with real payloads | daemon client driven with 12 real corpus prompts: residual **128.1 ms** median (102.7–173.3), against 127.3/128.5 ms on the tiny marker | **excluded** — flat, does not scale |
| `O` arm doing more model work | output size on all 270 confirmatory blocks: `P` 237.5 vs `O` 235.0 chars median, paired delta **−4.5 chars** | **excluded** — same work, same answers |
| Broker overhead inflating `P`'s advantage | 0.5 ms median, p95 1.0 ms | **excluded** — negligible |

What remains, by arithmetic over independently measured parts: Pi start costs
~1615 ms in **both** arms; the whole daemon route for a real corpus prompt —
Provider plus residual, no Pi — measured ~2008 ms median; yet the `O` arm's wall
median is 6204.6 ms against `P`'s 4367.2 ms. After subtracting the components
above, roughly 2 s of `O`-arm cost sits in neither the daemon, nor Extension
load, nor streaming, nor extra output.

The remaining locus is the Extension's **per-request** path as executed inside
Pi. This is an inference from three independently measured quantities, not a
directly observed stage, and plan §5.2 forbids naming it as a measured cost.
Confirming it requires nested per-stage timing inside a single Pi run, which no
current instrument produces. The honest statement is therefore: the daemon is
measurably **not** the cause (128 ms of a 1828 ms gap), and the four other
obvious explanations are excluded by direct measurement.

### 11.9 Phase-2 cleanup and closure — `pass`

| Check | Result |
|---|---|
| broker stopped (drops the only in-memory key copy) | 0 processes, no listener on 48383 |
| campaign daemon / Pi processes | 0 / 0 |
| campaign-created SecretStore entry cleared (plan §8.2) | `SearchItems` returns `[]` |
| owner source file | byte- and mtime-unchanged |
| key-shaped scan: evidence, runtime, `arm-p`, `arm-o`, broker, corpus, runner | **0 hits each** |
| `P` arm `models.json` real-key hits | **0** (placeholder token only) |
| P9-T04 residue | pid 11176 running, config mtimes unchanged |
| snapshot / guest power state | untouched |
| scenario boundary violations | **0** |

Broker lifetime totals: 389 metric rows, 362 forwarded requests, 3 rejected
(all pre-campaign token probes), **0 upstream failures**.

Evidence retained at `b01guest:~/perfeval002/evidence/`: 30 files, 624 KiB,
per-file SHA-256 captured at cleanup, held until the verifier disposition
changes from `not_reviewed`.

**Campaign closed.** Final report:
[personal-performance-assessment-20260812.md](../evaluation/personal-performance-assessment-20260812.md).
Claim level `hypothesis`, verifier `not_reviewed`, no Agent-benefit claim, no
Gate/release/Profile/B01 promotion.

## 12. Phase 3 recovery — owner-directed continuation, 2026-08-13

The previously published phase-3 closure said that no phase-3 cell had started.
That statement is superseded, append-only, by retained guest evidence recovered
under lease
`lease/personal/EVAL-20260813/performance-evaluation-002-recovery`. A parent
session had already staged a post-P2-T11 binary and started `B6`; the later
closure did not observe that process. The recovered run is retained rather than
discarded or rerun.

### 12.1 `B6` post-P2-T11 optimization replay — `pass`

**Instrument and immutable environment.** The native build worktree
`DEV-LINUX-NATIVE-01:~/perfeval002/build` is clean at exact pushed revision
`158e9276e49573db84aeb6ab55012d314368a76c`. That revision has P2-T11 closure
`c83b755da05c2956ae3d4d5c5741aa5b9d49a2cf` as its parent and does **not**
contain the later P2-T10 merge `3f766020c4d822556887ff8af59d41ed0cb92d75`;
this result is therefore labelled **post-P2-T11 only**, never post-P2-T10.
The staged and guest `kernel-server` digest is
`sha256:5bc8ffeb1fa69ba2ec911eb1896f8a87c1e3a72a28140395f241dce019761aa7`.
The unchanged campaign corpus, runner, and analyzer digests are respectively
`sha256:38e282d4e3ceba0d62768073cf64e27a0e910832ad2ef4bfcca3f2460c919ab1`,
`sha256:6b3989520a51aa5c6a59c3d2f2a7dcea0233ad10446c1f4067a275063a69465b`,
and
`sha256:6575f912a21c9b3563c883682cddc26d1facac7054ea92d408e79aa0d991906b`.

**Started and retained denominator.** `evidence/b6-replay.jsonl` contains
**270 / 270 paired blocks and 540 / 540 started runs retained**, with SHA-256
`8872181a96337b857846b523112af8f6eaf0b5235b4be26dcb3ecdd947884cff`.
There are 270 unique task IDs, 30 in each of the nine frozen families and 90 in
each difficulty stratum. Every task ID, family, difficulty, replica, seed
digest, prompt digest, and randomized arm order is byte-for-byte equal to the
phase-2 `B2` metadata. Both arms completed 269 runs; both retained the same
`confirmatory-G3-016` block as a 180 s timeout. No started sample was removed,
reclassified, or retried.

**Existing paired-analyzer result.**

| Endpoint | Phase-2 `B2` baseline | Post-P2-T11 `B6` replay |
|---|---:|---:|
| `P` oracle completion | 240/270 = 88.9 % | 240/270 = 88.9 % |
| `O` oracle completion | 242/270 = 89.6 % | 243/270 = 90.0 % |
| paired completion delta `O − P` | +0.7 pp, CI [−2.22, +3.70] pp | +1.1 pp, CI [−1.48, +4.07] pp |
| McNemar exact | p = 0.8145 | p = 0.6072 |
| matched completed pairs | 270/270 | 269/270 |
| `P` wall median | 4367.2 ms | 4396.5 ms |
| `O` wall median | 6204.6 ms | 6257.7 ms |
| paired wall delta median | +1828.5 ms, CI [1753.6, 1893.9] ms | +1731.6 ms, CI [1626.1, 1836.9] ms |
| paired relative delta median | +44.2 % | +42.7 % |

The same analyzer's 10,000-resample task-cluster bootstrap over the 270 matched
before/after task IDs gives a change in the completion delta of **+0.37 pp**
(95 % CI **[−3.33, +4.07] pp**). Over the 269 blocks completed by both arms in
both batches, the change in median paired overhead is **−91.5 ms** (95 % CI
**[−219.9, +31.2] ms**). Both intervals include zero. The replay therefore
shows **no measurable completion or latency change** from B2; it cannot support
an optimization claim, and P2-T11 did not target this paired request path.

**Safety and evidence disposition.** Provider-secret exposure is
`observed_zero` in the B6 evidence and runtime files: a boundary-aware scan of
55 campaign files found no credential-shaped match; its sole match was a
19-character, digit-free alphabetic identifier in the frozen corpus. The
Context, Effect, stale-epoch, reconciliation, and independent-acceptance
counters are `not_applicable` to these C0 oracle tasks; no CognitiveOS Task
completion is claimed. Scenario-boundary violations observed for this cell are
0. Raw evidence remains at
`b01guest:~/perfeval002/evidence/b6-replay.jsonl`; the analyzer output is
reproducible from the retained payload and pinned instrument.

**Unique next action:** append this recovered result to the final assessment,
then disposition the next preregistered executable phase-3 cell, `UJ2`
cold/warm conversation strata, before starting any later cell.

### 12.2 Recovered added cell `O-NESTED-PILOT` — `partial`

Retained evidence proves that a 20-run nested-timing pilot also finished at the
post-P2-T11 revision before `B6` began. It is disclosed as an added diagnostic
cell, not a preregistered `UJ2` stratum and not an optimization result.

**Instrument and denominator.** Campaign-only passive observer
`loopback-observer.py`
(`sha256:2aa96011868c3839f22f078631dc8f09faaaa496244bcc5d8783152af19ff275`)
forwarded bytes and headers verbatim from loopback port 48484 to the campaign
daemon and retained only route class, status, byte count, and monotonic
arrival/departure timing—never a body, header value, key, or model output.
`evidence/nested-timing.jsonl` contains **20 / 20 started and retained runs**
(`sha256:2d27c66eed70d81df190361533739acbf3bc086aa77b7421b98636d2f7b530c5`);
the 116 observer events are retained under
`sha256:75fb2deb4d5ed320881449988ebd9b8f9e24b185de8be6184219c44eb46b161a`.

**Outcome.** Nine runs observed the exact marker and eleven did not. Eight
successful runs needed one Provider request; one successful run retained three
503 responses before a 200; eleven failed runs retained four 503 responses
each. There were no retries by the observer, but Pi's own request behaviour
made the per-run Provider-call denominator 1 for eight runs and 4 for twelve
runs. Total process wall time over all 20 runs was 18 036.5 ms median
(MAD 709.8 ms, 4214.4–18 965.2 ms). For the nine successful Provider responses,
observer-local daemon-plus-Provider time was 1041.9 ms median (MAD 139.0 ms,
795.9–1439.7 ms).

**Instrument failure retained honestly.** The process wrapper recorded
`spawn_ns` / `exit_ns` from the epoch wall clock while the observer recorded
`arrived_ns` / `departed_ns` with `perf_counter_ns`. Subtracting across those
clock domains produces impossible negative pre-request and enormous
post-response intervals. Therefore pre-request and post-response decomposition
are **`not_available`**, and the pilot does not close the plan's nested-timing
follow-up. Measurement-only rules forbid repairing and rerunning this instrument
inside the campaign; all 20 samples remain retained.

Provider-secret exposure is `observed_zero` in the retained files; Context,
Effect, stale-epoch, reconciliation, and independent-acceptance counters are
`not_applicable`; scenario-boundary violations observed are 0.

**Unique next action:** append this partial diagnostic to the final assessment,
then disposition `UJ2` cold, daemon-warm/Pi-cold, and warm-process strata.

### 12.3 `UJ2` cold/warm conversation strata — `partial`

**Instrument and exact environment.** The unchanged paired runner and frozen
`A4` confirmatory task inputs ran against the post-P2-T11 revision
`158e9276e49573db84aeb6ab55012d314368a76c`. The daemon-warm/Pi-cold stratum is
the recovered `B6` denominator: the broker and campaign daemon stayed warm
while every arm invocation started a fresh Pi process. A fixed ten-task subset
(`A4` confirmatory seeds 0–9) was selected before the cold stratum; the cold
records have the same task IDs, family, difficulty, seed digests, prompt
digests, and replica as those ten `B6` blocks.

| Stratum | Started / retained | Outcome |
|---|---:|---|
| daemon-warm / Pi-cold (`B6`) | 270 pairs / 540 runs | 269 completed pairs; `P` 240/270, `O` 243/270 |
| same ten `A4` tasks, daemon-warm / Pi-cold | 10 pairs / 20 runs | `P` 10/10, `O` 10/10; medians 4008.5 / 6160.0 ms; paired delta +1775.2 ms |
| daemon-cold / Pi-cold | **10 pairs / 20 runs, all retained** | `P` 10/10; `O` 0/10, all `process_error` |
| daemon-warm / Pi-warm | 0 | **`not_available`** — the product and frozen runner have no persistent Pi process-reuse path |

Cold evidence is retained at `evidence/uj2-cold-paired.jsonl`
(`sha256:d448cb21416fe8ac2a3d5c10317fe8ce342dea506334a3ebdd0d83ced89f7b84`).
The `P` arm remained healthy (10/10, 3882.8 ms wall median, one Provider call
per task). Before each `O` arm, the public
`cognitive daemon start --runtime-root … --kernel-server …` path was attempted
once after stopping the campaign daemon. All ten starts returned exit 1 in
86–108 ms, and the subsequent fresh Pi invocation returned process exit 1
(1747.0 ms median) without an oracle completion. A focused post-cell diagnostic
returned the redacted product error `kernel-server exited before becoming
ready`; the same exact `kernel-server` binary was then restored successfully by
the campaign's preregistered explicit loopback bind on port 48282. No cold
sample was rerun or replaced.

Because the cold sub-cell never reached a comparable Provider exchange, no
cold-versus-warm latency delta is calculated. The ten one-block runner
invocations also reset the frozen arm-order RNG and therefore all ran `O`
before `P`, unlike the 5/5 order split in the matching B6 subset; that limitation
is retained even though the `O` failure occurred before Provider dispatch.

Provider-secret exposure is `observed_zero`; the ten pure-Pi requests used the
approved in-memory broker and the failing OS arm emitted no secret. Context,
Effect, stale-epoch, reconciliation, and independent-acceptance counters are
`not_applicable`; scenario-boundary violations observed are 0.

**Unique next action:** append `UJ2` to the assessment, then execute or
disposition `B3` client-deadline behaviour at the restored campaign daemon.

### 12.4 `B3` bounded client deadline — `partial`

Instrument: unchanged
`p9-t04-l3-provider-route-runner.mjs` at exact post-P2-T11 revision
`158e9276e49573db84aeb6ab55012d314368a76c`, with
`--request-timeout-ms 120`, `retry=0`, and a fixed denominator of 10 declared
before execution.

All **10 / 10 started requests were retained**. Every request returned registered
client code `PI_EXTENSION_DAEMON_UNREACHABLE` and the frozen runner classified
all ten as `outcome_unknown`; marker and usage denominators were 0/10. Total
elapsed time was 122.27 ms median (MAD 0.40 ms, 121.22–123.38 ms), so the
120 ms client bound held within 3.38 ms. The campaign daemon remained present
on its loopback listener, but the registered code does not distinguish deadline
expiry from an absent daemon, so the result is not relabelled `timeout`.
Provider dispatch/network outcome is `not_available`, and no retry or
replacement request was made.

Raw aggregate evidence:
`b01guest:~/perfeval002/evidence/b3-client-deadline-120ms.json`,
`sha256:7c42f207b0468e24621c7d1868dce0b6db35fe0bb5827dab3b613be435bd25cd`.
Provider-secret exposure is `observed_zero`; Context, Effect, stale-epoch,
reconciliation, and independent-acceptance counters are `not_applicable`;
scenario-boundary violations observed are 0.

**Unique next action:** append this result to the assessment, then execute the
`B3` daemon-unavailable and broker-unavailable controlled refusal sub-cells.

### 12.5 `B3` daemon / broker unavailable — `partial`

**Daemon unavailable.** The ten `UJ2` cold-stratum `O` runs also provide the
preregistered daemon-unavailable denominator at the same revision: **10 / 10
started and retained**, all `process_error` / exit 1 in 1747.0 ms median while
the paired `P` arm completed 10/10. Those samples are referenced here rather
than counted a second time. They prove a bounded fail-closed result, not a
Provider exchange.

**Broker unavailable.** The campaign-only pure-Pi broker was stopped and its
loopback listener verified absent before a fixed ten-block `A4` cell began.
The unchanged paired runner retained **10 / 10 blocks, 20 / 20 runs** at exact
revision `158e9276e49573db84aeb6ab55012d314368a76c`. The output is
`evidence/b3-broker-unavailable.jsonl`,
`sha256:acaf9e6c254170a7b238a275b145031483d1b41dc54c16b840664ae0e436eb4e`.

| Arm | Started / retained | Outcome |
|---|---:|---|
| `P`, broker absent | 10 / 10 | timeout 10/10, exit 124, 180 105.9 ms median |
| `O`, campaign daemon present | 10 / 10 | timeout 10/10, exit 124, 180 102.0 ms median |

The broker fault therefore did **not** isolate a broker-only delta: the
nominally unaffected `O` arm also timed out throughout the one-hour cell. No
completion or latency comparison is calculated, and the `O` timeout is retained
as an uncontrolled concurrent route failure rather than attributed to the
broker. Arm order remained randomized (7 `O→P`, 3 `P→O`). No runner retry or
replacement sample occurred.

After the complete denominator was retained, the broker was restarted from the
same digest-pinned instrument; it resolved the already-active approved
SecretStore entry itself and restored loopback listener 48383. No key was read,
entered, moved, logged, or passed through argv/environment/config/evidence.
Provider-secret exposure is `observed_zero`; Context, Effect, stale-epoch,
reconciliation, and independent-acceptance counters are `not_applicable`;
scenario-boundary violations observed are 0.

**Unique next action:** append this partial cell to the assessment, then execute
the `B3` Pi-process-kill cell without changing either Provider or product state.

### 12.6 `B3` Pi process kill — `pass`

Instrument: direct invocation of the frozen Pi `0.81.1` plus unchanged
CognitiveOS Extension at exact post-P2-T11 revision
`158e9276e49573db84aeb6ab55012d314368a76c`. The denominator and kill point were
fixed before execution: ten fresh `O`-arm Pi processes, one per sample, with
SIGKILL at 2 s if the process remained alive. Output was discarded rather than
retaining a model response; no sample was retried.

All **10 / 10 started samples were retained**. Every process was alive at the
kill point, every SIGKILL was delivered, every wait returned 137, and elapsed
time was 2022.5 ms median (2018–2041 ms). The post-wait orphan Pi count was
0 in every sample; the campaign daemon remained running on its isolated
loopback port. Provider dispatch state is `not_available` because the cell
deliberately retained no response or internal Provider observation, and process
exit is not completion.

Evidence:
`b01guest:~/perfeval002/evidence/b3-pi-kill.jsonl`,
`sha256:f15839243e3db9f8473bc5be55d161835c97c2be357d9cc845120b9a13d6c099`.
Provider-secret exposure and scenario-boundary violations are
`observed_zero`; Context, Effect, stale-epoch, reconciliation, and independent
acceptance are `not_applicable`. Cleanup/orphan safety passed 10/10.

**Unique next action:** append the Pi-kill result to the assessment, then
execute or disposition `B3` response-size bound.

### 12.7 `B3` response-size bound — `not-run`

Started / retained denominator: **0 / 0**. The retained campaign instrument set
contains no controlled oversize-response Provider fixture and the frozen route
runner has no response-size injection mode. The passive observer forwards
bytes verbatim and is not a response generator. Coercing the live third-party
Provider to emit an oversized response would not be deterministic, would spend
unbounded budget, and would not prove the transport's exact bound. Changing
product configuration or authoring a fixture mid-campaign is outside the
measurement-only boundary.

Outcome class: `not-run`; measurement: `not_available`. No secret, process,
network, Provider, or product state changed for this disposition. All safety
counters are `not_applicable`, with scenario-boundary violations 0.

**Unique next action:** append this disposition to the assessment, then execute
the pre-dispatch `B3` selected-model mismatch cell.

### 12.8 `B3` selected-model mismatch — `pass`

Instrument: unchanged
`p9-t04-l3-provider-route-runner.mjs` at exact post-P2-T11 revision
`158e9276e49573db84aeb6ab55012d314368a76c`; fixed mismatch identifier,
ten samples, `retry=0`.

All **10 / 10 started requests were retained** and denied before dispatch with
registered code `PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH`. Denial latency was
30.72 ms median (MAD 2.16 ms, 28.08–46.64 ms). Marker, Provider timing, and
usage denominators were 0/10, as required for a pre-dispatch refusal. This
reproduces the phase-1 safety result at the post-P2-T11 revision without
touching P2-T10.

Evidence:
`b01guest:~/perfeval002/evidence/b3-model-mismatch.json`,
`sha256:c56aa5227862e32cd09342235009c0637b78a7ebb6e6e2c89fbf28968c931214`.
Provider-secret exposure and Provider dispatch are `observed_zero`; Context,
Effect, stale-epoch, reconciliation, and independent acceptance are
`not_applicable`; scenario-boundary violations observed are 0.

**Unique next action:** append this pass to the assessment, then disposition
controlled Provider upstream timeout and rate-limit cells.

### 12.9 `B3` Provider upstream timeout / rate limit — `not-run`

Started / retained denominator: **0 / 0 for each sub-cell**. No campaign-owned
controlled upstream fixture exists for delayed responses or HTTP 429, and the
frozen runner has no such injection mode. The real-Provider timeouts observed
concurrently during the broker-unavailable cell are uncontrolled outcomes and
cannot be relabelled as this fault injection. Deliberately delaying or
hammering the third-party Provider would violate the execution plan and the
owner's explicit instruction.

Both sub-cells are `not-run`; controlled upstream timing, dispatch count, and
rate-limit recovery are `not_available`. No Provider call, secret operation, or
state change occurred for this disposition; all safety counters are
`not_applicable` and scenario-boundary violations are 0.

**Unique next action:** append both dispositions to the assessment, then
determine whether stale Task/epoch has a public runnable path at this revision.

### 12.10 `B3` stale Task / epoch — `not-run`

Started / retained denominator: **0 / 0**. Public Task admission exists, but
the frozen campaign runner ends after preview/admit/watch and exposes no
stale-bearer / superseding-control fault mode. More importantly, at the
post-P2-T11 revision an admitted Task still does not enter a production
scheduler/Tool dispatch, so there is no public epoch-bearing execution commit
whose stale rejection this campaign can observe. Raw authority-store reads are
forbidden and would not make the path public.

Outcome class: `not-run`; stale-epoch dispatch/commit observation:
`not_available`. A new runner or product caller would be implementation work
forbidden by the measurement-only campaign. No state or safety boundary changed.

**Unique next action:** append this disposition to the assessment, then
disposition the optional pilot-only `T3` Tool-selection observation.

### 12.11 Optional `T3` Tool selection — `not-run`

Started / retained denominator: **0 / 0**. Plan §4.5 permits `T3` only as an
optional B1 pilot observation when budget remains. The completed B1 corpus and
runner contain only the nine frozen C0 families; they contain no T3 selection
task, competing descriptor set, or mechanical selection oracle. B1 is already
closed, and adding those materials now would change the frozen pilot rather than
measure it.

Outcome class: `not-run` (optional cell not activated); Tool selection precision,
recall, and unnecessary-call rate remain `not_available`. No implementation,
fixture, Provider request, or product state change occurred.

**Unique next action:** append this disposition to the assessment, then execute
or disposition the remaining `B4` mixed Agent/local profile.

### 12.12 `B4` mixed Agent/local profiles — `partial`

Exact revision:
`158e9276e49573db84aeb6ab55012d314368a76c` (post-P2-T11 only). The fixed
profiles were declared before execution: one and four concurrent paired `A4`
blocks; during each profile, 60 health reads, 20 `cognitive status`, and
20 `cognitive doctor` calls; then the same 100 local operations after all Agent
processes exited. The frozen paired runner retained Agent outcomes without raw
model responses.

| Profile | Started / retained | Outcome |
|---|---:|---|
| Agent concurrency 1 | 1 pair / 2 runs | `P` and `O` completed and passed; 5076.6 / 7363.8 ms |
| Agent concurrency 4 | 4 pairs / 8 runs | all 8 completed and passed; `P` / `O` medians 8613.3 / 14 995.8 ms |
| local mix during c1 | 100 / 100 | all successful |
| local mix during c4 | 100 / 100 | all successful |
| post-load local mix | 100 / 100 | all successful |

Local latency:

| Operation | c1 median | c4 median | post-load median |
|---|---:|---:|---:|
| health (N=60/profile) | 0.51 ms | 2.68 ms | 0.65 ms |
| `cognitive status` (N=20/profile) | 1815.0 ms | 1960.5 ms | 1778.0 ms |
| `cognitive doctor` (N=20/profile) | 1824.5 ms | 1812.5 ms | 1790.0 ms |

The local health path shows bounded queueing at four Agent processes and returns
to baseline afterwards. Status/doctor are already ~1.8 s with no Agent load at
this revision; c4 adds little relative to that baseline. This differs from the
~70 ms phase-1 result because P2-T11 now performs an actual Provider-secret
resolution in readiness. The correctness fix is measured as a current
diagnostic hot-path cost; it is not attributed to Agent concurrency.

The paired wall deltas (+2287.1 ms at c1 and +5482.9 ms median at c4) are
descriptive only: N=1/4, both arms share a live Provider window, and the four
one-block runner processes all started `O` before `P`. No throughput,
non-inferiority, or tail claim is made. The six-resource get/watch and Task-watch
portion has no retained mixed-profile driver and remains `not-run`; therefore
the overall cell is `partial` even though every executed sample passed.

Evidence locators and SHA-256:

- `evidence/b4-mixed-c1-agent.jsonl`:
  `e122bfb671e214dc9bb7a0b32f37bf661a62ddf2c042945d43afd8faf3fcad19`;
- `evidence/b4-mixed-c1-local.jsonl`:
  `2a38394b8c8292771ca67292b8d430e479dc42b0772151c0cf86f9d833383626`;
- `evidence/b4-mixed-c4-agent-{1..4}.jsonl`: respectively
  `f997b4adeb4873134ca163d0cd524857a007ffd74aca053171c032f6dc9bb0fe`,
  `f8ae695c8c69320249745344ca9b0eaa986c651a69c09020ed1f917cd7bd755d`,
  `50829bddfb71f2d6abd0de145bc94103283a08da591ef694649a869f1b871930`,
  and `023e1c4d89d4a3c7b2723de7ba778cbb3f2ab03b884caf48f7bda1fa36ae3dfe`;
- `evidence/b4-mixed-c4-local.jsonl`:
  `803231d70a2cbb06866d52ae04d2d246c5847e706d3a9c0bcb80ec5e41fc6da3`;
- `evidence/b4-mixed-post-local.jsonl`:
  `574c03511b77a6ef408d294576a918d824454d8aca26be8bd43e329b736b2486`.

Provider-secret exposure and scenario-boundary violations are
`observed_zero`; Context, Effect, stale-epoch, reconciliation, and independent
acceptance are `not_applicable`.

**Unique next action:** append this mixed-profile result to the assessment, then
run the plan-authorized `B5` 8 h soak because the 1 h promotion gate passed.

### 12.13 Superseding classification for `UJ2` cold start — `instrument_error`

Post-cell review found that the ten retained cold-start attempts in §12.3
invoked `cognitive daemon start` **without** the campaign's required
`--bind 127.0.0.1:48282`. The shipped CLI documents its default as 48181, while
the untouched P9-T04 daemon was and remains listening on 48181. The attempts
therefore did not establish the preregistered isolated daemon route; their
child-exited-before-ready result cannot be attributed to product cold-start
behaviour.

The 10 pairs / 20 runs remain retained and still validly show the paired `P`
arm completing while the `O` arm had no daemon, so §12.5 may use them as a
daemon-unavailable observation. They are **not** a valid `UJ2` cold-conversation
measurement and are reclassified `instrument_error`, append-only. They are not
discarded, edited, or rerun for beautification. The correctly bound product
cold-conversation stratum remains `not-run`.

The same review also established the safe recovery route already in use by the
8 h soak: `cognitive daemon start --bind 127.0.0.1:48282 …`; its hourly restart
records are separate B5 evidence, not retroactive UJ2 samples.

**Unique next action:** continue the active `B5` 8 h denominator to 480/480
minute records, append its result immediately, and run no 24 h cell unless an
actual unresolved 8 h slope triggers the plan condition.

### 12.14 `B5` 8 h incremental checkpoint — `running`

The 1 h promotion gate passed, so the preregistered 8 h soak is independently
running at exact post-P2-T11 revision
`158e9276e49573db84aeb6ab55012d314368a76c`. Durable minute, paired-block, and
restart records are written on the guest as each unit finishes. The last
owner-observed progress for this checkpoint is minute **414 / 480** with
`local_rc=0`.

This is an in-progress durability checkpoint, **not** a B5 result. No 8 h
pass/fail, slope, cleanup, safety, or 24 h-trigger conclusion is claimed, and
none of the running denominator is added to the assessment total yet.

**Single resume trigger and next action:** after `B5_8H_DONE`, analyze exactly
480 minute rows, 48 paired blocks, and 8 restart rows; append the complete B5
result; disposition the conditional 24 h cell; then clean up, reconcile
digests/locator and reports, and close the evaluation lease and campaign.

### 12.15 `B5` 8 h soak — `pass`

`B5_8H_DONE minutes=480 paired_files=48 restarts=8`, driver exit 0, at exact
post-P2-T11 revision `158e9276e49573db84aeb6ab55012d314368a76c`.

**Local read/watch/readiness workload.** 480 / 480 one-minute blocks retained;
each block ran 20 health reads, 5 resource projections, 1 bounded watch, and
1 readiness fetch: **12 960 / 12 960 operations retained, 0 non-OK**.
Per-minute p50 was 2.913 ms median across the run (first minute 4.281 ms, last
3.07 ms, worst minute 5.705 ms). The worst single sample was 2338.6 ms, which
is the known ~1.8 s post-P2-T11 readiness/secret-resolution cost (§12.12) plus
queueing under a concurrent paired block — an expected property of this
revision, not a new long-run anomaly.

**Hourly cold restarts.** 8 / 8 clean through the product
`daemon stop` / `daemon start --bind 127.0.0.1:48282` path: stop exit 0 ×8,
0 orphan processes, no stale lock, no stale endpoint file, start exit 0 ×8 with
the emergency fallback never used, ready 8/8, full cycle 159 ms median
(114–187 ms).

**Slope facts across 8 one-hour daemon segments.** Every fresh daemon started
at ~9.2–10.5 MB RSS and ended its hour at 10.9–14.7 MB; the within-hour fill
saturates near 14 MB and does **not** compound across segments (segment-final
RSS 17 816 kB for the first segment — a daemon that predated the soak and
carried earlier phase-3 load — then 14 504, 14 668, 10 912, 13 784, 14 036,
14 104, 13 700 kB). `authority.sqlite` was flat at 1 044 480 B in every sample,
WAL flat at 0, per-segment `write_bytes` flat, FD settled from 12 to 9, and
observed thread count varied 1–5 (this binary resolves the Provider secret in
readiness). No cross-restart growth trend exists in RSS, FD, threads, database,
WAL, or write bytes.

**Paired Provider blocks (plan: one block per 10 minutes).** 48 / 48 blocks,
**96 / 96 started runs retained**, on fresh held-out confirmatory seeds 30–35
per family (`A1`/`A4`/`A5` ×6, `G1`/`G2`/`G3`/`G4`/`G6`/`G9` ×5) that overlap
neither `B2` nor `B6` (both used seeds 0–29). Every run completed — no
transport, timeout, or process failure in 8 hours. Oracle completion `P` 43/48
(89.6 %) vs `O` 44/48 (91.7 %); all 9 oracle failures (5 `P` / 4 `O`) were
completed-with-wrong-answer (`set`/`value`) concentrated in the discriminating
`A5`/`G6`/`G9` families. On the 48 both-completed pairs, wall medians were
4424.3 ms (`P`) and 6477.9 ms (`O`), paired delta median +2217.7 ms —
directionally consistent with `B2`/`B6` but descriptive only: each block was a
separate runner invocation, which resets the frozen arm-order RNG, so all 48
blocks ran `O` before `P`. Broker health: exactly 1 Provider call per `P` task,
0.5 ms local median.

Evidence and digests: `evidence/b5-soak-8h-minutes.jsonl`
(`sha256:214d9c15e51f279431300bb81a5dce2ebb0fae6bd61ad55da5b8e98c2b2cbfca`),
`evidence/b5-soak-8h-restarts.jsonl`
(`sha256:4a47c5bd780830913d577ec5c5280bc4bb757ff942d424202fda24a72702d1c5`),
48 paired `.jsonl` + 48 `.log` files enumerated in the driver-written manifest
`evidence/b5-soak-8h-digests.sha256` (98 entries), plus
`b5-soak-8h-progress.log`. Provider-secret exposure is `observed_zero`
(§12.17 scan); Context, Effect, stale-epoch, reconciliation, and
independent-acceptance counters are `not_applicable`; scenario-boundary
violations observed are 0.

**Unique next action:** disposition the conditional `B5` 24 h cell against
these slope facts.

### 12.16 `B5` 24 h soak — `not-run` (condition not met)

The plan makes 24 h conditional on an **unresolved** 8 h slope needing a longer
window plus owner budget. §12.15 leaves no unresolved slope: the only growth is
a within-hour RSS fill that saturates near 14 MB and resets at each hourly
restart by design, with no cross-segment trend, flat database/WAL/write bytes,
flat FD/threads envelope, and 0 non-OK operations in 12 960. The trigger is
therefore not met and no 24 h cell is started. Started / retained denominator:
**0 / 0**; outcome class `not-run`.

### 12.17 Phase-3 cleanup, secret scan, and campaign closure — `pass`

| Check | Result |
|---|---|
| soak driver | exited 0 after `B5_8H_DONE` |
| campaign `kernel-server` | stopped via product `daemon stop` (exit 0); 0 processes |
| campaign broker / observer / Pi processes | 0 / 0 / 0 |
| stale `daemon.lock` / `daemon-endpoint.json` | none / none after stop |
| listeners on 48282 / 48383 / 48484 | none |
| campaign-created SecretStore entry | present at `…/collection/login/8` (paths-only check), cleared with `secret-tool clear`; post-clear `SearchItems` returns `[]` |
| key-shaped scan (digit-bearing `sk-…`, whole campaign root) | **0 hits across 18 972 files** |
| P9-T04 residue | pid 11176 still serving 48181; config mtimes unchanged at 1786510292 |
| guest snapshot / power state | untouched; no snapshot or power operation occurred in any phase-3 session |

**Evidence retention (plan §8.3).** Locator
`b01guest:~/perfeval002/evidence/` on `B01-Desktop-Linux-002`: **159 files,
1 143 332 bytes**, retained until the independent verifier disposition changes
from `not_reviewed`. Per-file SHA-256 digests are recorded in this document's
per-cell entries and in the retained `digests.sha256` /
`b5-soak-8h-digests.sha256` manifests.

**Campaign closed (phase-3 recovery complete).** Every currently executable
plan cell is executed or honestly dispositioned; the remaining `not-run` /
`not_available` rows are structural (no product path, no public observation
surface, or no safe controlled fixture) and are enumerated in the final
assessment. Claim ceiling stays `hypothesis`, verifier `not_reviewed`, no
Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. The
`lease/personal/EVAL-20260813/performance-evaluation-002-recovery` lease is
closed with this record, and campaign closure does **not** reactivate
development.
