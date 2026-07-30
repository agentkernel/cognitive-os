# P1-T09 verified experimental deployment handoff

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Lease: `lease/personal/P1-T09/verified-experimental-deployment`
- Branch: `lane/personal-p1-t09-abi-targeted-campaign-v2`
- Change class: implementation-only
- Normative surface: unchanged
- Development track: `experimental-local-only`

## Current result

The previous signed campaign (`30553835734`) passed archive hash checks and
independent offline verification, but its installer and daemon required glibc
versions newer than the qualified host's glibc `2.35`. The verified installer
failed closed before active-pointer promotion; the prior user service remained
active.

Campaign `30562532613` confirmed that the reviewed source fully validates on
the secret-free `ubuntu-latest` job. Its approved signing job failed before it
read the signing seed, built the Extension, signed an artifact, or uploaded an
artifact: Ubuntu 22's native linker could not resolve `__isoc23_sscanf` and
`__isoc23_strtol` required by the current dependency graph.

The pending corrective workflow replaces runner-libc coupling for the four
published Rust binaries with a pinned portable toolchain:

- Zig `0.14.0` via `mlugg/setup-zig@v2`;
- `cargo-zigbuild` `0.23.0`;
- target `x86_64-unknown-linux-gnu.2.35`; and
- an explicit pre-signing `readelf --version-info` check rejecting every
  `GLIBC_2.36+` requirement.

The regular full workspace validation remains separate and unchanged on
`ubuntu-latest`. The protected signing job still runs only after that validation
passes and after explicit approval of
`personal-linux-experimental-campaign`.

## Verification for this corrective slice

| Check | Result |
|---|---|
| Campaign `30562532613` full validation | pass |
| Campaign `30562532613` signing payload build | fail before signing/artifact creation; `__isoc23_*` linker symbols unavailable on Ubuntu 22 |
| `git diff --check` | pass |
| Workflow semantic execution after Zig correction | not-run; requires merge and a fresh immutable protected campaign |
| B01 / GMVP-LINUX / release / Profile | not-run / non-claim |

## Next executable action

Merge the corrective workflow through required CI, then dispatch a fresh
immutable campaign version. After full validation passes, approve only the
fixed protected Environment for that run. Before any deployment mutation,
verify the new artifact's hashes, signature, expected Pi pin, and dynamic glibc
requirements on the qualified host. Use only the verified installer; do not
manually copy the daemon, CLI, or Extension into product deployment paths.

After a compatible deployment, configure a persistent absolute exact-Pi
`0.81.1` executable path and run the existing redacted product-route runner,
native Secret Service smoke, and focused negatives. Do not treat any of these
experimental-host results as B01, release, GMVP-LINUX, or Profile evidence.

## Current blocker record

- `blocked_paths`: compatible signed artifact and product bundle deployment
  paths on the experimental Linux host.
- `blocked_task_ids`: `P1-T09`.
- `blocked_gate_ids`: `B01`, `GMVP-LINUX`, and Profile.
- Owner: P1-T09 verified-deployment lease holder.
- Next action: merge, dispatch, independently verify, then retry deployment
  with the explicit glibc-2.35 payload.
