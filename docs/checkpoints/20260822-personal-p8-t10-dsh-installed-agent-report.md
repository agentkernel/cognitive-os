# P8-T10 dsh installed-agent report (2026-08-22)

- Change class: product-semantic Personal task `P8-T10` (install DeepSeek
  Harness onto cognitiveos-personal as a product agent path).
- Target: `B01-Desktop-Linux-002` (`linux-002`) through
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`.
- Claim ceiling: implementation evidence / tested-local / linux-002 /
  performance observation only; no Gate, release, Profile, B01, EVAL, or
  Agent-benefit claim.

## Task and lease

- Task: `P8-T10` (`in-progress`)
- Slice: `P8-T10/D01`
- Branch: `personal/P8-T10-dsh-installed-agent`
- Lease: `lease/personal/P8-T10/dsh-installed-agent`
- P8-T09 remains `done` (PR [#254](https://github.com/agentkernel/cognitive-os/pull/254))

## Pins

- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- bridge protocol: `cognitiveos.dsh-akp/0.1`

## Validation ledger

| Check | Result | Evidence / limitation |
|---|---|---|
| Formal task/lease/branch | **pass** | `P8-T10`; lease active; D01 in-progress |
| `cognitive dsh configure` pin + candidate-only digest | implementation written | local Windows GNU Rust `not-run` (`RUST-LINK-DEV-WIN-GNU-01`) |
| `cognitive dsh launch` Path B without requiring Pi | implementation written | doctor overall ready + five components; Pi may stay `not_configured` |
| Direct Flash product launch fail-closed | implementation written | `cognitive dsh launch --path a` returns measurement-only error |
| TypeScript adapter tests | **not-run** this slice | unchanged protocol crate; Path A/B helper scripts added |
| linux-002 identity | **not-run** | D02 |
| Installed Path B Workspace* + Flash | **not-run** | D02; must use `cognitive dsh configure` then `launch --print` |
| Same-host Path A vs B n≥5 | **not-run** | D03; P8-T09 n=1 jump-host vs n=2 guest is not a paired comparison |
| Secret residue cleanup | **not-run** | after D02/D03 |
| Required CI | **not-run** | after D01 push |

## P8-T09 gap this task closes

P8-T09 proved the adapter and a harness-driven real dsh `--patch` Path B on
linux-002. It did not install dsh via `cognitive dsh`, and Path A vs B used
different hosts with tiny n. Owner follow-on requires the product CLI path,
installed-agent measurement, and a same-host paired comparison. Injected
`startupEvents` remain candidate events; a dsh response is never Task
completion; `dsh.json` digest is not SQLite-durable daemon adapter state.

## Non-claims

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, EVAL, or
Agent-benefit promotion. Do not preset lossless.

## Unique next action

Push D01, confirm required CI, then D02 on identity-confirmed linux-002.
