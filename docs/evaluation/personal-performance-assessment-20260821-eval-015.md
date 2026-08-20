# PERSONAL-PERF-EVAL-015 running assessment

- Campaign: `PERSONAL-PERF-EVAL-015`
- Freeze branch: `evaluation/EVAL-015-freeze`
- Product pin: `adc40499`
- Lease: `lease/personal/EVAL-015/remaining-plan-cells`
- Preregistration: [20260821-personal-perf-eval-015-preregistration.md](../checkpoints/20260821-personal-perf-eval-015-preregistration.md)
- Claim ceiling: `hypothesis` / `not_reviewed`
- Independent reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only.

This freeze completes parent plan §9 remainder after closed EVAL-014.
EVAL-014 C1/C2a B0/B1/B2 on this pin are carried as prior evidence and are
not re-run. This report does not promote Gate, release, Profile, B01, or
Agent-benefit. EVAL-002 and EVAL-004 through EVAL-014 remain closed.

## Cell log (`TEST-REPORT-INCREMENTAL-01`)

| Cell | Result | Note |
|---|---|---|
| Owner activation / lease | **pass** | `PERSONAL-PERF-EVAL-015`; `lease/personal/EVAL-015/remaining-plan-cells` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running id 35; MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off; residue `48181`/`48284`/`48383` untouched; `perfeval012`/`013`/`014` present unused |
| Carry EVAL-014 C1/C2a | **carried** | B0 pass; B1 5/5; B2 30/30 counted on pin `adc40499`. Not re-run. Report: [EVAL-014](personal-performance-assessment-20260820-eval-014.md) |
| Source archive | **pass** | 15,155,200 bytes; SHA-256 `0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`; `scp` into `/home/hal9001/perfeval015-20260821` |
| Exact-source binaries | **pass** | host `DEV-LINUX-NATIVE-01` `--locked --release` reused by digest; `kernel-server` `436725ec…`; `cognitive` `73bad94d…`; `pi-agent-adapter` `54ce9eaa…`; Extension `d27f9776…`; Pi `0.81.1`; glibc-only |
| Secret bind / doctor | **pass** | new item `/27`; `secret_material_written: true`; `secret_ref_redacted: true`; `selected_model: deepseek-v4-flash`; daemon pid 369399 on `48306`; doctor overall `ready`; `first_conversation_ready: true` |
| P-arm broker `48406` | **pass** | stdin broker pid 369469; D-Bus GetSecret paths `["27"]`; `material_written: true` to broker stdin only; health `key_loaded: true`; never `search`/`lookup` |
| C0 B0 warmups | **pass** (discarded) | 3/3 G1 pairs both oracle True; not in denominator |
| C0 B0 qualification | **pass** | 9/9 blocks started/retained; 0 timeout; 7/9 oracle both arms; G6/G9 both-fail (hardness). Broker 12/12 upstream_ok including warmups. Evidence secret-shaped 0/10. `retry=0` |
| C0 B1 | running | 90 pairs (9×5 seeds 1–5 × 2 replicas); disjoint from B0 seed 0 |
| C0 B2 N freeze | pending | |
| C0 B2 | pending | |
| MS-AUTH / T-GOV / UJ2 / UJ3 / UJ4 | pending | |
| B3 / B4 / B5 | pending | 24 h default deferred |
| C2b session-2 / Skill | pending | campaign-local; no daemon restart for resume |
| C2c fault profile | pending | campaign-authorized default-off; else `not-run` |
| Cleanup | pending | |

## Non-claims

Activation is not B0. Carrying EVAL-014 is not a new C1/C2a sample. No Gate /
release / Profile / B01 / Agent-benefit promotion.

## Source freeze (2026-08-21) — pass

Same-pin `git archive` of `adc40499` copied with `scp` (no PowerShell pipes):
15,155,200 bytes, SHA-256
`0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`.
Host freeze binaries reused by digest. Guest Pi `cli.js --version` is
`0.81.1`. Closed EVAL roots were file-copied for Pi runtime and C0
instruments only; they were not used as this campaign's binds.

## Secret bind / doctor (2026-08-21) — pass

Login collection was empty. Owner-designated key imported through
`cognitive init --api-key-file -` into new SecretStore item `/27`.
`secret_material_written: true`, `secret_ref_redacted: true`,
`selected_model: deepseek-v4-flash`. No temp key file. No
`secret-tool search`/`lookup`. Daemon `127.0.0.1:48306` pid 369399.
Doctor overall `ready`, `first_conversation_ready: true`. Residue
listeners `48181`/`48284`/`48383` untouched.

## C0 B0 (2026-08-21) — pass (path/fairness; not a performance claim)

Three discarded G1 warmup pairs (`start-index` 100, 3 replicas): both
arms oracle True. Qualification: one pilot seed per C0 family
(`start-index` 0, 1 replica). 9/9 started and retained. Every arm
`completed` (0 timeout). `retry=0`. 180 s timeout. Mechanical `ANSWER:`
oracle. Same Pi `0.81.1`, `--no-tools`, model `deepseek-v4-flash`.

| Family | Arm order | P oracle | O oracle | P wall ms | O wall ms |
|---|---|---|---|---:|---:|
| A5 | P,O | pass | pass | 16162 | 10270 |
| G2 | O,P | pass | pass | 3552 | 3350 |
| G6 | P,O | fail | fail | 23029 | 7175 |
| G1 | P,O | pass | pass | 3406 | 3741 |
| G3 | P,O | pass | pass | 6133 | 4681 |
| G4 | P,O | pass | pass | 4335 | 3792 |
| A1 | P,O | pass | pass | 4436 | 4126 |
| G9 | O,P | fail | fail | 4727 | 4678 |
| A4 | O,P | pass | pass | 3650 | 3793 |

G6/G9 both-fail is family hardness (same pattern as EVAL-004 B0), not an
arm-specific instrument defect. Evidence secret-shaped hits 0/10.

## Unique next action

Finish C0 B1 90-pair pilot (`retry=0`), freeze B2 N=30 per family, then
run C0 B2 confirmatory.
