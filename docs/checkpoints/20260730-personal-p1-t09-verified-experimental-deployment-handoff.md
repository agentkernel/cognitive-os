# P1-T09 verified experimental deployment handoff

- Date: 2026-07-31
- Task: P1-T09 install-to-first-conversation route
- Lease: `lease/personal/P1-T09/verified-experimental-deployment`
- Branch: `lane/personal-p1-t09-abi-targeted-campaign-v2`
- Change class: implementation-only
- Normative surface: unchanged
- Development track: `experimental-local-only`

## Current result

PR [#123](https://github.com/agentkernel/cognitive-os/pull/123) merged the
portable payload correction. Protected campaign `30566251554` passed both its
full reviewed-input validation and its approved signing/upload job for
`0.0.0-campaign.20260730.11`. The payload build uses Zig `0.14.0`,
`cargo-zigbuild` `0.23.0`, and `x86_64-unknown-linux-gnu.2.35` for the four
published Rust binaries; its pre-signing `readelf --version-info` check rejects
every `GLIBC_2.36+` dynamic requirement.

The experimental host was reconfirmed as Linux x86_64, glibc `2.35`, with
native user-systemd running. To avoid its failed direct GitHub transport, a
SHA-256-fixed local source bundle was copied to a session-owned directory; the
host then verified the source bundle, passed `git fsck`, checked out exact
commit `2523efd1af9d860b861d5f0ddb755237adc06001`, and built
`linux_bundle_verifier` with the locked toolchain. That verifier independently
accepted campaign `.11`'s signature, expected Pi pin, and public key. Separate
`readelf` checks found no `GLIBC_2.36+` requirement in either the installer or
the payload kernel server.

The verified installer activated campaign `.11` over `.4`, and
`cognitiveos-personal.service` is active. The exact Pi package was persistently
installed only after the registry SRI matched the campaign manifest; its
absolute executable reports `0.81.1`. The installed CLI configured only those
non-secret Pi and Extension paths. The redacted doctor projection reported
native SecretStore and first-conversation readiness. The redacted installed
route runner initially exposed a post-configure observation race, so a
failure-first delayed-doctor regression was added; its bounded retry fix passed
locally and the real installed route then reached its Pi first-response call.
That call timed out at 90 seconds with no response output. No Provider material,
SecretRef, SQLite path, Task, Effect, Verification, capability, or authority
data was printed, and no authority side effect was created. Session-owned
campaign and source material was removed.

Follow-up commit `33a05a9` corrects the Pi model-selection path: Pi does not
apply `--provider` without an explicit model, and the Extension had registered
but not activated its daemon-selected model. The focused failure-first
regressions, package build, package tests, route-runner tests, consistency
check, and whitespace check passed locally. Dispatch `30591622368` for
`0.0.0-campaign.20260731.1` passed reviewed-input validation but produced no
artifact because the protected signing Environment rejected this branch: its
custom branch policy permits only `main`.

## Verification for this corrective slice

| Check | Result |
|---|---|
| PR #123 required CI | pass; Ubuntu and Windows jobs succeeded |
| Campaign `30566251554` reviewed-input validation and signed upload | pass |
| Qualified host platform recheck | pass; Linux x86_64, glibc `2.35`, native user-systemd running |
| Immutable host verifier | pass; SHA-256-fixed source bundle, `git fsck`, exact commit, and locked build |
| Campaign `.11` offline verification | pass; signature, artifact, key, and expected Pi compatibility accepted |
| Installer and payload ABI checks | pass; no `GLIBC_2.36+` dynamic requirement found |
| Verified installer activation | pass; `.11` active and `cognitiveos-personal.service` active |
| Persistent exact Pi | pass; manifest-matching registry SRI and absolute executable `0.81.1` |
| Native SecretStore / first-conversation readiness | pass; redacted doctor projection only |
| Redacted installed first-response route | fail; reached Pi invocation but timed out at 90 seconds with no response output and no authority side effect |
| Delayed-doctor focused regression | pass after failure-first observation |
| Daemon-selected model activation regression | pass after failure-first observation |
| Campaign `30591622368` reviewed-input validation | pass |
| Campaign `30591622368` signing/upload | blocked; no artifact because the protected Environment permits only `main` |
| `git diff --check` | pass before documentation closure |
| B01 / GMVP-LINUX / release / Profile | not-run / non-claim |

## Next executable action

Merge `33a05a9` to `main`, then dispatch the protected experimental campaign
from `main`. Independently verify and install the resulting artifact before
rerunning the redacted first-response route. Do not treat the experimental-host
route as B01, release, GMVP-LINUX, or Profile evidence.

## Current blocker record

- `blocked_paths`: installed Pi first-response route and the protected signing
  Environment branch policy; B01 campaign design paths require a separate lease.
- `blocked_task_ids`: `P1-T09`.
- `blocked_gate_ids`: `B01`, `GMVP-LINUX`, and Profile.
- Owner: P1-T09 route-probe-reconciliation lease holder.
- Next action: merge the reviewed branch to `main`, dispatch the protected
  campaign, then independently verify/install and rerun the route before B01
  preregistration.
