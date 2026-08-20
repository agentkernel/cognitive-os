# PERSONAL-PERF-EVAL-012 — preregistration

- Campaign ID: `PERSONAL-PERF-EVAL-012`
- Status: **closed** (2026-08-20). Evaluation routing **OFF**.
- Lease: `lease/personal/EVAL-012/c1-c2-paired-b0` (closed)
- Branch: `evaluation/EVAL-012-freeze`
- Claim ceiling: `hypothesis` / non-claim. Reviewer: `not_reviewed`.
- Parent: [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md)
  v1.1. C1/C2 overlay:
  [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md)
  §6 (replaces stale “OS arm unreachable” gates for this EVAL only).
- Freeze checkpoint:
  [20260820-personal-perf-eval-012-preregistration.md](../checkpoints/20260820-personal-perf-eval-012-preregistration.md)
- Running report:
  [personal-performance-assessment-20260820-eval-012.md](personal-performance-assessment-20260820-eval-012.md)

Closed EVAL-002 and EVAL-004 through EVAL-011 are never resumed.

## 1. Activation gate

Activation requires **all** of:

1. Packages 6–13 have supported evidence in the P9-T08 running report. **pass.**
2. Owner sets the Current snapshot `Owner-directed campaign` row to
   `PERSONAL-PERF-EVAL-012`. **pass** (owner “激活”, 2026-08-20).
3. An evaluation lease `lease/personal/EVAL-012/<purpose>` owns only
   `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`.
   **pass** (`lease/personal/EVAL-012/c1-c2-paired-b0`).

## 2. Isolation (bound at activation)

| Resource | Bound value |
|---|---|
| Guest | `B01-Desktop-Linux-002` only |
| Route | `wuz@192.168.1.2` → ProxyJump `hal9001@192.168.123.160` |
| Root | `/home/hal9001/perfeval012-20260820` |
| Daemon | `127.0.0.1:48300` |
| P-arm broker | `127.0.0.1:48400` |
| SecretStore | new item `/24` (≠ `/12`–`/19`; planned `/20` was the reservation name) |
| Git revision | `370b26fcc05976c7c1c97e5510a99ed3ebc23f2c` (pushed `origin/main` after P9-T08 merge). Instrument pin used for readiness evidence: `7dc8c999729028ddb850ab858e88e2f1ba8d5bf9` |

Procedure:
[20260820-personal-c1-c2-b01-guest-procedure.md](../checkpoints/20260820-personal-c1-c2-b01-guest-procedure.md).
Bind:
[20260820-personal-c1-c2-secret-bind-runbook.md](../checkpoints/20260820-personal-c1-c2-secret-bind-runbook.md).

Do not restore or delete snapshots. Do not contact `B01-Clean-Linux-001`.
Do not reuse closed EVAL ports `48286`–`48298` / `48386`–`48398` / `48383`,
SecretStore `/12`–`/19`, or P2-T37 roots.

## 3. Cell list (must not forget C2b–d)

B0 (package 15): one qualification seed per class C1, C2a, C2b, C2c, C2d;
three warmups per arm; secret scan; tool-equivalence; timeout; cleanup; no
claim samples.

B1 (package 16): five pilot seeds per class; two runs per arm.

B2 (package 17): 30 held-out paired seeds per class; three runs per arm when
the Provider lacks deterministic replay; `retry=0`; started = retained.

Frozen cell overlay: `tools/personal/c1-c2-paired/cells.json`.
C2b/C2c/C2d are split-score / capability-gap unless tool sets match.

Owner 2026-08-20 expanded this EVAL to the remainder of
[personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md)
(“完成所有”). C0, B3–B5, T6–T9, S4/S8, and the §10 matrix are now **in
scope** on this freeze only (`370b26fc` on `B01-Desktop-Linux-002`). Missing
frozen runner, corpus, oracle, or public observation surface stays
`not-run` / `not_available` (measurement-only; no product fix). Independent
reviewer remains `not_reviewed` (plan §9 step 2 cannot be reconstructed;
B0 already started). B5 24 h stays conditional. B6 stays later-only.
Closed EVAL-002 and EVAL-004–011 remain closed; do not reuse their
roots, ports, SecretStore items, or denominators.

## 3.1 Instrument freeze (readiness evidence)

Computed on `DEV-LINUX-NATIVE-01` at exact
`7dc8c999729028ddb850ab858e88e2f1ba8d5bf9`. `retry=0`. Seeds: B0×1, B1×5,
B2×30 per class C1/C2a/C2b/C2c/C2d (180 disjoint ids). Product freeze for
this campaign is `370b26fc`; re-pin instrument digests on the guest from
that revision before the first counted sample.

| File | SHA-256 |
|---|---|
| `fixtures/c1/workspace/note.txt` | `sha256:4fb26b79e8de937c59f203f9274d76998db1f063ae0de442fdbceedb6d74869b` |
| `fixtures/c2a/workspace/src/repair.ts` | `sha256:ddcfe6d7028b7c437c1315804376758a03d554fe3b83d3528d0a81d250764c28` |
| `fixtures/c2a/workspace/tests/repair.test.ts` | `sha256:40c098045577722f134d78027fdf688407b5bd83bf2ce0579bc1de303e09f930` |
| `fixtures/c2a/oracle.json` | `sha256:73dd7f3668593dc0f595f814ac3e4cb35973e2fb6a30c770abd578cbc0e4f7c2` |
| `fixtures/c2b/procedure.txt` | `sha256:33fa92bb1f244d97d00ca2a24c9ca86e13fc02cf18f22146691e23e802318d3f` |
| `fixtures/c2c/original-key.txt` | `sha256:f88e7a35a799f332ec60ac2ba31a714904bafb1ad314c5721d0b7beda25be9b4` |
| `fixtures/c2d/oracle.json` | `sha256:82f55335475866f2d4e745f89de1bde7d9205f2df12e27610877311eb5f4e91b` |
| `pure-pi-broker.mjs` | `sha256:7e859638a9c8d5ca5a06a8023a65764609044ed8da1bb5225ce78471c71f93d1` |
| `workspace-fixture-adapter.mjs` | `sha256:d2a1bc808c8d871fc4782fb247a75076503b7086f8cffe44d4c11841aeb738d9` |
| `linux-secret-service.mjs` | `sha256:f81ec80c72a37f7192c5a707c2aaf89204d6ece49ae4eb463ad315349533a297` |
| `linux-secret-get-helper.py` | `sha256:ae7fcc7fdfe5878c1acf6d9b50ac8db0b1a754720cae71e2f7122ff152380436` |
| `fairness-checker.mjs` | `sha256:07f2905c2dca2fadfed41ff98fbc1b33a7ad5b9addd768961209c094702f408f` |
| `redactor.mjs` | `sha256:ed646b45f0785204a8de38bd65936495a298504e1ef79d3cc25bc44bae7fb544` |
| `freeze.mjs` | `sha256:4a61164388455cae909e6b06f67a61cbd1b012169ec8d7252856fb7573c3580f` |
| `paired-runner.mjs` | `sha256:0fe2bca7d604a0680afcb64654fd1f5b28ba390bc2a19038673a355a54fb3510` |
| `prove-linux-secret-get.mjs` | `sha256:cd220a20a57e9f18ad566791b50d05102ecc0d44ddcdaf0080a828e8329b2b78` |
| `cells.json` | `sha256:82f93a40cd9b4a8a3486f14c584096bc0fa3268e2de6e9edba65807d6f0bd37a` |

## 4. Measurement rules

- Measurement-only (Operating Model §2.5): no product/contract/negative/test
  or generated-handbook edits to make a cell runnable.
- Provider cells `retry=0`.
- Missing capability is `not-run` / `not_available`.
- No Gate, release, Profile, B01, or Agent-benefit promotion.

## 5. Provider budget (recorded at activation, before B0)

Counted C1/C2 envelope only (EVAL-004 DeepSeek ceiling style):

| Batch | Counted arm-runs | Note |
|---|---:|---|
| B0 | 10 | 5 classes × 2 arms × 1 qualification; 30 warmups are non-counted |
| B1 | 100 | 5 × 5 seeds × 2 arms × 2 runs; after B0 pass |
| B2 | 900 | 5 × 30 seeds × 2 arms × 3 runs; after B1 |
| **Ceiling** | **1010** | stop when this remainder is exhausted; do not invent extra cells |

B0 sub-ceiling: **10** counted Provider-facing C1/C2 cells. A fairness fail
blocks C1/C2 B1. Do not open C1/C2 B1/B2 spend on this freeze until B0 pass.

Expanded remainder (C0 G/A families, B3–B5, T/S/O/UJ rows) does **not**
inherit extra counted Provider budget. Those cells execute only when a
frozen runner/oracle already exists on this pin; otherwise they are
dispositioned `not-run` / `not_available` without inventing a corpus.

## 6. Cleanup

Stop campaign daemon `48300` and broker `48400` only. Clear only the
campaign-unique SecretStore item with `secret-tool clear` on product
non-secret attributes; confirm with D-Bus `SearchItems` paths. Never
`secret-tool search` / `lookup`. Leave `48181` and closed EVAL roots untouched.

## 7. Non-claims

Activation is not B0 pass, not a paired performance result, and not Gate /
release / Profile / B01 / Agent-benefit evidence.
