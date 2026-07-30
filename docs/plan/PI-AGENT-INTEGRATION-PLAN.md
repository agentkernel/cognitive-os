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

Product Gate incompleteness does not block implementation in the
`experimental-local-only` development track. P2-P6 work may be developed and
tested locally in parallel, but its product state remains pending/blocked until
the listed exit evidence exists. Local execution is `tested-local`, never a
C0/C1, Profile, release, sandbox, or provenance claim.

| Phase | Deliverable | Exit evidence | Current state |
| --- | --- | --- | --- |
| P1 | Candidate-only Pi launcher and real DeepSeek smoke/evaluation | no-tools policy tests; actual model and latency output; zero authority/Effect | delivered in this batch |
| P2 | Pi supply-chain verifier | immutable package source, digest/SRI plus a trusted signature/provenance policy accepted by `SignatureProvenancePort` | official-publisher path remains blocked: npm SRI alone is not trusted signature/provenance evidence. Custom User-Provided mode now requires the user to review a fixed risk notice and confirm a digest-pinned `file://` project bundle bound to a `principal://` operator. After acknowledgement it uses the same normal installation, authorization and runtime path; it is still not an official-publisher, C0/C1, Profile or sandbox claim. |
| P3 | Durable InstallationStore | SQLite process-recovery, atomic visibility and management-authority commit for `AgentInstallation` | KRN SQLite WAL staging/commit/recovery slice is merged. Lane-RUN now consumes it through an exclusive in-process `DurableInstallationManager` session: verification precedes stage/commit, recovery is manager-only, and durable persistence grants zero capabilities. Targeted runtime tests and lint passed locally; cross-process lifecycle leasing remains a separate KRN API decision. This is still not a governed `AgentInstallation` completion or C0/C1 claim. |
| P4 | OS sandbox adapter | Linux-native negative evidence for filesystem/network/secrets/subprocess/tool-proxy and no cross-platform claim merge | pre-launch admission is provided on `lane/run-pi-batch1`: Windows-native is refused, WSL2 is separately refused, and a Linux request requires exact policy/adapter/compatibility digests, a healthy registered adapter and an HTTPS model egress proxy to the exact DeepSeek endpoint. No concrete sandbox adapter or Pi subprocess launch exists; Linux-native evidence remains pending. |
| P5 | Pi lifecycle/I/O adapter | mediated tool/memory/completion/checkpoint/recovery mapping; bypass, revoke and OOB tests | pending after P3/P4 |
| P6 | Governed installation and evaluation | committed installation with no automatic high-risk capability; prerequisite behavior vectors; preregistered workload report | blocked by P2-P5 |
| P7 | Performance campaign | REQ-PERF-004 L2-green reference platform, fixed hardware/topology/baseline and measured p50/p95/p99 | not started |

Before P7, the local Personal performance runner must attribute latency to four
separate boundaries: CognitiveOS deterministic processing, Pi/Node process
startup and RPC handling, Provider/network/model latency, and filesystem/SQLite
work. Fixed-platform campaigns and A/B/C/D agent-benefit evaluation remain
later activities; local samples cannot claim either result.

## Evaluation protocol for P1

Run only from an isolated work/config directory. Current local development
must use the explicit ADR-0018 exception and an independent Personal Provider
config directory; it must not read a parent-process `DEEPSEEK_API_KEY`. Use:

```text
pi-agent-adapter evaluate --pi <pi-bin> --model <deepseek-model> \
  --prompt <fixed-prompt> --expected-text <expected> --runs <1..=20> \
  --work-dir <empty-dir> --config-dir <empty-pi-dir> \
  --provider-config-dir <personal-provider-config-dir> \
  --allow-local-native-provider-secret-development
```

Every sample records success, latency, requested and observed model, and
whether Pi emitted tool results. A failed, timed-out or model-mismatched sample
remains in the denominator. This command cannot claim governance overhead,
agent benefit or deployment readiness. This local exception is Linux native
only, default-deny, and remains explicitly uncontained because the initial Pi
child may pass its environment to descendants; it is not a sandbox or release
credential-delivery design. The future daemon-owned Provider proxy remains the
normal Personal product path. Adapter admission classifies WSL independently
from Linux native and rejects WSL, Windows, and enabled CI before selecting or
probing the native Secret Service backend.

For future Linux-native local runs, the designated experimental SSH host is
`personal-linux-native-01` at `wuz@192.168.1.2`. A no-secret qualification
probe on 2026-07-30 confirmed non-WSL Linux x86_64, a running native
user-systemd and user D-Bus, Rust `1.97.1`, and Node `22.19.0`; SSH
authentication is available. Treat it as a local-only engineering host, keep
its evidence separate from CI Ubuntu and Windows/MSVC, and do not upgrade it
into a product, sandbox, Gate, Profile, or release claim merely because a
command succeeded there.

Each remote slice must run through non-interactive SSH, use a disposable
remote work directory, and re-check the exact Pi package/binary version before
load. Do not put credentials, `SecretRef`, Provider configuration, SQLite or
authority paths, selected-model material, or raw Pi output in SSH arguments,
environment, terminal captures, or committed evidence. On 2026-07-30, `pi`
was absent from PATH and an uncredentialed
`npm exec --package=@earendil-works/pi-coding-agent@0.81.1 -- pi --version`
probe produced no version output after two minutes and was stopped. Therefore
exact Pi availability remains `not-run`, not a satisfied pin and not a product
blocker. The next real-load slice must resolve this availability check before
attempting `--extension <absolute-path>`.

The P0-T06 `extension-load` verb is a bounded local evidence probe. It requires
the reviewed fixture and `/cognitiveos-p0-t06-status`, starts a real pinned Pi
RPC session, waits for the Extension status response, and returns only redacted
event types, status text, timeout state, and timing. It must run only after
`verify:local` report/evidence validation is green on a supported local path.
No credential, raw transcript, model content, command argument containing a
secret, SQLite write, authority transition, or Effect is permitted in its
output. A successful probe is real Extension/RPC load evidence only; it does
not satisfy the later compatibility, sandbox, product Gate, Profile, or release
criteria.

## Non-negotiable exclusions

- Never put a credential in repository files, logs, evidence committed to Git,
  command arguments or `auth.json`.
- Never let candidate output produce an authorization decision, capability,
  Effect or Task completion.
- Custom User-Provided installation requires explicit risk acknowledgement for a
  digest-pinned local bundle. It never upgrades a user declaration into publisher
  provenance; later runtime permission remains governed by the same normal
  authorization path.
- Never claim Windows-native sandbox coverage from WSL2/Linux evidence.
- Do not promote the candidate launcher to C0/C1 without P2-P5 evidence.

## P4 pre-launch admission evidence (2026-07-24)

`cognitive_runtime::admit_pi_launch` has no success path on Windows-native or
WSL2. On a non-Linux host it also refuses a caller-supplied `linux_native`
label. The only Linux-host admission shape is an opaque permit; it carries no
authority and no concrete adapter in this repository can turn it into a
subprocess. Missing/faulted/unregistered adapters, any binding digest mismatch,
missing proxy, a non-HTTPS proxy, malformed/empty digest binding, and a non-registered model endpoint all fail
closed with the existing `AGENT_ADAPTER_BYPASS_DETECTED` code.

The verification run for this code was a WSL2 Linux guest diagnostic only:
`cargo test -p cognitive-runtime --offline` = 52 passed / 0 failed and
`cargo clippy -p cognitive-runtime --all-targets -- -D warnings` = pass.
It is not Linux-native evidence and does not update F-017 or Profile status.
