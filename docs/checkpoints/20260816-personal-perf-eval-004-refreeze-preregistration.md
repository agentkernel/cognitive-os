# PERSONAL-PERF-EVAL-004 re-freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-004`
- Lease: `lease/personal/EVAL-20260816/full-os-only-refreeze`
- Date: 2026-08-16
- Frozen product source (intended): `origin/main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd`
  (merge of P2-T28 / PR #227; BR-01..BR-08 are on `main`)
- Target: `B01-DESKTOP-002` / `B01-Desktop-Linux-002`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse the 2026-08-15 campaign root
`/home/hal9001/perfeval004`, loopback port `48284`, SecretStore entry, broker,
runner, corpus, oracle, evidence denominator, or any prior EVAL-004 asset.
`PERSONAL-PERF-EVAL-002` remains closed and is not resumed.

## Owner authorization

Owner standing instruction: after BR-01..BR-08 merge, re-freeze EVAL-004 and
continue measurement. BR-08 closed via PR
[#227](https://github.com/agentkernel/cognitive-os/pull/227) at
`main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval004-20260816` mode `0700` (not yet created) | `/home/hal9001/perfeval004`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48286` | `48181`, `48282`, `48284` |
| SecretStore entry | new campaign-only item via owner-approved hidden/stdin path | any prior EVAL-004 or P9-T04 item |
| Source archive | to be produced from clean `1e71344a` | archive digest `sha256:3578b4fa…` from `93dde21` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| BR-01..BR-08 merged | **pass** | P2-T21..P2-T28 on `main@1e71344a` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `1e71344a7b2c4a443fd0581e7fd33f21e970efbd` |
| Source archive + SHA-256 | not-run | generate on `DEV-LINUX-NATIVE-01` from clean checkout |
| New campaign root/port | not-run | create only after archive pin |
| New SecretStore entry | not-run | owner-approved hidden/stdin path; never argv/env/log |
| Pure-Pi broker freeze | not-run | loopback-only, memory-only key, no body/header log |
| Equivalent fixture/oracle/runner | not-run | P/O tools, bytes, budget, timeout, retry=0 identical |
| Redactor/sampler/cleanup digests | not-run | campaign-only ignored artifact roots |
| Independent reviewer before B1 | not-run | `not_reviewed`; B0 may qualify target but cannot enter B1 |

No B0/B1/B2/B3/B4 sample has started under this freeze. No Gate, release,
Profile, B01, or Agent-benefit claim is created by this preregistration.

## Unique next action

Produce a clean source archive of `origin/main@1e71344a` on
`DEV-LINUX-NATIVE-01`, record SHA-256, then create the new guest campaign root
and port. Do not start a Provider sample and do not reuse the old SecretStore
entry.
