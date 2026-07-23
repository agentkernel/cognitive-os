# Pi Agent integration plan

- Status: active, staged integration plan
- Owner: Lane-RUN
- Scope: Pi (`@earendil-works/pi-coding-agent`) as an external DeepSeek-backed
  candidate source; no Console work

## Current evidence

- Official package installed in an isolated local directory: version `0.81.1`,
  repository `https://github.com/earendil-works/pi.git`, MIT license, npm SRI
  `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==`.
- `pi-agent-adapter` disables tools, extensions, skills, project context,
  session persistence and project trust; it clears inherited API-token
  environment variables and redacts the process-scoped DeepSeek key from child
  output.
- Actual Windows-native DeepSeek smoke: 5/5 fixed-output runs passed with no
  tool results, no authority commit and no Effect. Requested `deepseek-chat`;
  provider-reported model was `deepseek-v4-flash`. Candidate-process latency:
  p50 6081 ms, p95 6451 ms, p99 6451 ms.

This is a candidate-only smoke measurement, not a REQ-PERF-004 hardware
campaign, a REQ-PERF-005 benefit claim, a C0/C1 compatibility claim, or an
AgentInstallation commit.

Local evidence (gitignored, no credential or raw transcript) is recorded at
`artifacts/evidence/pi-agent/20260724-deepseek-candidate-evaluation.json`.

## Delivery sequence

| Phase | Deliverable | Exit evidence | Current state |
| --- | --- | --- | --- |
| P1 | Candidate-only Pi launcher and real DeepSeek smoke/evaluation | no-tools policy tests; actual model and latency output; zero authority/Effect | delivered in this batch |
| P2 | Pi supply-chain verifier | immutable package source, digest/SRI plus a trusted signature/provenance policy accepted by `SignatureProvenancePort` | blocked: npm SRI alone is not the required trusted signature/provenance evidence |
| P3 | Durable InstallationStore | SQLite process-recovery, atomic visibility and management-authority commit for `AgentInstallation` | pending Lane-KRN/RUN split; in-process ledger is insufficient |
| P4 | OS sandbox adapter | Linux-native negative evidence for filesystem/network/secrets/subprocess/tool-proxy and no cross-platform claim merge | pending; Windows-native remains unsupported |
| P5 | Pi lifecycle/I/O adapter | mediated tool/memory/completion/checkpoint/recovery mapping; bypass, revoke and OOB tests | pending after P3/P4 |
| P6 | Governed installation and evaluation | committed installation with no automatic high-risk capability; prerequisite behavior vectors; preregistered workload report | blocked by P2-P5 |
| P7 | Performance campaign | REQ-PERF-004 L2-green reference platform, fixed hardware/topology/baseline and measured p50/p95/p99 | not started |

## Evaluation protocol for P1

Run only from an isolated work/config directory, with `DEEPSEEK_API_KEY` set
for one process and removed immediately after. Use:

```text
pi-agent-adapter evaluate --pi <pi-bin> --model <deepseek-model> \
  --prompt <fixed-prompt> --expected-text <expected> --runs <1..=20> \
  --work-dir <empty-dir> --config-dir <empty-dir>
```

Every sample records success, latency, requested and observed model, and
whether Pi emitted tool results. A failed, timed-out or model-mismatched sample
remains in the denominator. This command cannot claim governance overhead,
agent benefit or deployment readiness.

## Non-negotiable exclusions

- Never put a credential in repository files, logs, evidence committed to Git,
  command arguments or `auth.json`.
- Never let candidate output produce an authorization decision, capability,
  Effect or Task completion.
- Never claim Windows-native sandbox coverage from WSL2/Linux evidence.
- Do not promote the candidate launcher to C0/C1 without P2-P5 evidence.
