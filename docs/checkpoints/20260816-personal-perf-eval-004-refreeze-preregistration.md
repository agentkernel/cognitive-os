# PERSONAL-PERF-EVAL-004 re-freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-004`
- Lease: `lease/personal/EVAL-20260816/full-os-only-refreeze`
- Date: 2026-08-16
- Frozen product source: `origin/main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd`
  (merge of P2-T28 / PR #227; BR-01..BR-08 are on `main`)
- Campaign registration: PR #228 merged at
  `main@f1fa00a1a9698e8059a594f047ab2d6676854e32` (docs-only; product tree unchanged)
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
| Campaign root | `/home/hal9001/perfeval004-20260816` mode `0700` | `/home/hal9001/perfeval004`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48286` (not listening; not bound) | `48181`, `48282`, `48284` |
| SecretStore entry | new campaign-only item via owner-approved hidden/stdin path | any prior EVAL-004 or P9-T04 item |
| Source archive | `sha256:a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8` (14,407,680-byte `git archive` of `1e71344a`) | archive digest `sha256:3578b4fa…` from `93dde21` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| BR-01..BR-08 merged | **pass** | P2-T21..P2-T28 on `main@1e71344a` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `1e71344a7b2c4a443fd0581e7fd33f21e970efbd` |
| Source archive + SHA-256 | **pass** | `git archive --format=tar --prefix=cognitiveos-personal-1e71344a/` of `1e71344a`; 14,407,680 bytes; `sha256:a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8` verified on Windows, `DEV-LINUX-NATIVE-01`, and the guest |
| New campaign root/port | **pass** | `/home/hal9001/perfeval004-20260816` mode `0700`; port `127.0.0.1:48286` unused. Prior root `/home/hal9001/perfeval004` and listeners `48181`/`48284` left untouched |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `cargo build --release --locked -p kernel-server -p admin-cli` from extracted archive, 1m40s, Rust 1.97.1. `kernel-server` 16,456,928 bytes `sha256:ecc1bf395d0d4368dfd4d32666cecaa2bb1bc5350f26fa2f52a7829aa1ce1e3e`; `cognitive` 10,313,952 bytes `sha256:760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5`. Digests matched host `/home/wuz/eval004-refreeze-20260816/` and guest campaign root |
| Campaign daemon on `48286` | **pass** | public `cognitive daemon start --runtime-root …/runtime --bind 127.0.0.1:48286 --kernel-server …/kernel-server`; pid 199172; lock live; bootstrap present (value not read). Public status: system/database/secret/daemon `ready`, provider `blocked` (`provider_config_missing`), pi `not_configured`, `first_conversation_ready: false`, `authority_side_effects: false`. Listeners `48181`/`48284` still present and untouched |
| New SecretStore entry | not-run | owner-approved hidden/stdin path; never argv/env/log |
| Pure-Pi broker freeze | not-run | loopback-only, memory-only key, no body/header log |
| Equivalent fixture/oracle/runner | not-run | P/O tools, bytes, budget, timeout, retry=0 identical |
| Redactor/sampler/cleanup digests | not-run | campaign-only ignored artifact roots |
| Independent reviewer before B1 | not-run | `not_reviewed`; B0 may qualify target but cannot enter B1 |

No B0/B1/B2/B3/B4 sample has started under this freeze. No Gate, release,
Profile, B01, or Agent-benefit claim is created by this preregistration.

## Archive and guest-root pin (2026-08-16)

Host `wuz@192.168.1.2` (`DEV-LINUX-NATIVE-01` / libvirt host `hal9000`) and
guest `hal9001@192.168.123.160` (`B01-Desktop-Linux-002`, running; identity
`hal9001-Standard-PC-Q35-ICH9-2009`) were contacted only on the registered
route. `B01-Clean-Linux-001` remained shut off and was not contacted.
`virsh -c qemu:///system` was read-only.

The source archive is a clean `git archive --format=tar
--prefix=cognitiveos-personal-1e71344a/` of
`1e71344a7b2c4a443fd0581e7fd33f21e970efbd` (1525 entries, 0 `.git` members).
SHA-256 `a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8`
matched on the Windows operator host, on `DEV-LINUX-NATIVE-01` at
`/home/wuz/eval004-refreeze-20260816/cognitiveos-personal-1e71344a.tar`, and
on the guest at
`/home/hal9001/perfeval004-20260816/cognitiveos-personal-1e71344a.tar`.
The guest root was created `0700`. Prior campaign root
`/home/hal9001/perfeval004` and pre-existing loopback listeners `127.0.0.1:48181`
and `127.0.0.1:48284` were left untouched. Port `48286` was not listening.

Windows GNU Rust build remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
Host disk at pin time: 32G free (93%). Guest disk: 42G free (26%).

## Exact-source binaries (2026-08-16)

`CARGO_NET_OFFLINE=true cargo build --release --locked -p kernel-server -p
admin-cli` finished in 1m 40s on `DEV-LINUX-NATIVE-01` (Rust 1.97.1) from the
extracted `1e71344a` archive. Guest `ldd` resolves only glibc/`libgcc`/`libm`
for both binaries (no missing libraries). Files were copied into the new
campaign root only; `/home/hal9001/perfeval004` was not read or modified.

| Binary | Bytes | SHA-256 |
|---|---|---|
| `kernel-server` | 16,456,928 | `ecc1bf395d0d4368dfd4d32666cecaa2bb1bc5350f26fa2f52a7829aa1ce1e3e` |
| `cognitive` | 10,313,952 | `760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5` |

## Campaign daemon start (2026-08-16)

Public caller:

`/home/hal9001/perfeval004-20260816/cognitive daemon start --runtime-root /home/hal9001/perfeval004-20260816/runtime --bind 127.0.0.1:48286 --kernel-server /home/hal9001/perfeval004-20260816/kernel-server`

Result: `action=started`, pid `199172`, endpoint `127.0.0.1:48286`, lock
`…/runtime/cognitiveos/daemon.lock`, `bootstrap_present=true` (value not read).
`cognitive status` reports system/database/secret/daemon `ready`, provider
`blocked` (`provider_config_missing`), pi `not_configured`,
`first_conversation_ready: false`, `authority_side_effects: false`. Guest
listeners are `48181`, `48284`, and `48286`; the first two were not stopped.

No Provider sample, Task, Tool, Effect, or SecretStore mutation occurred.

## Unique next action

Create a **new** campaign-only SecretStore item via the owner-approved
hidden/stdin path, then pin local Pi `0.81.1` under the new root. Do not
start a Provider sample, do not reuse the 2026-08-15 SecretStore item, and
do not bind `48284`/`48181`. Independent reviewer remains `not_reviewed`
(B0 may continue; B1 is forbidden).
