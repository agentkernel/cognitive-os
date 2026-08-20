# P9-T08 package 9 — B01 guest procedure (no live B0)

- Status: **procedure only**. No guest mutation. No samples.
- Guest: `B01-Desktop-Linux-002` only. `B01-Clean-Linux-001` is forbidden.
- Claim ceiling: `hypothesis` / non-claim.

This is a checkable start gate for a **future** `PERSONAL-PERF-EVAL-012`
measurement campaign. Writing it is not B0, not B01 Gate evidence, and does
not activate evaluation routing.

## 1. Control route (checkable)

1. SSH to libvirt host `wuz@192.168.1.2` (`hal9000`), `BatchMode=yes`.
2. Guest operations use `virsh -c qemu:///system` on that host.
3. Guest SSH: ProxyJump `wuz@192.168.1.2` → `hal9001@192.168.123.160`.
4. Confirm guest identity is `B01-Desktop-Linux-002` before any product
   install. If identity differs, **stop**.

## 2. Reserved isolation (bind only at EVAL activation)

| Resource | Reserved value | Bind when |
|---|---|---|
| Campaign ID | `PERSONAL-PERF-EVAL-012` | Owner activates Current snapshot row |
| Runtime root | `/home/hal9001/perfeval012-<activation-date>` | Package 15 start |
| Daemon loopback | `127.0.0.1:48300` | Package 15 start |
| P-arm broker loopback | `127.0.0.1:48400` | Package 15 start |
| SecretStore item | planned `/20` unless already present; then next free unused item ≠ `/12`–`/19` | Package 15 start |

Do not bind these ports or the SecretStore item during P9-T08. Do not reuse
closed EVAL ports `48286`–`48298` / `48386`–`48398` / `48383`, SecretStore
`/12`–`/19`, or P2-T37 roots `p2-t37-c2a-write-20260820` /
`p2-t37-c2a-patch-20260820`.

## 3. Snapshot / baseline

- Restore or residual P9-T04 / closed-EVAL state changes need a **separate
  owner decision**.
- This package must not take, revert, or delete snapshots.
- Standing operator authorization does **not** authorize package-9 guest
  mutation, force push, or `B01-Clean-Linux-001`.

## 4. Start sequence (package 15, not now)

1. Owner activates `PERSONAL-PERF-EVAL-012` in `PROGRESS.md` Current snapshot.
2. Claim `lease/personal/EVAL-012/<purpose>` with writable paths limited to
   `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`.
3. Confirm snapshot/baseline against the preregistration allowlist.
4. Create the new root; checkout the frozen exact Git revision.
5. Follow the secret bind runbook (package 11). Never copy `provider.json`.
6. Start daemon `127.0.0.1:48300` and P-arm broker `127.0.0.1:48400`.
7. Run B0 qualification cells only. `retry=0`. Retain every started sample.

## 5. Cleanup and stop

- Stop campaign daemon and broker only. Do not clear the owner key unless the
  campaign imported a **campaign-unique** SecretStore item; then clear that
  item with `secret-tool clear` on non-secret attributes, confirming with
  D-Bus `SearchItems` paths only.
- Never `secret-tool search` or `secret-tool lookup`.
- Stop for secret print, snapshot change beyond allowlist, or owner pause.

## 6. Non-claims

Writing this procedure is not B0, not a paired result, and not Gate / release
/ Profile / B01 / Agent-benefit evidence.
