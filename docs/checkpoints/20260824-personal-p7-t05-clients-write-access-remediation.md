# P7-T05/D10 clients write-access remediation and linux-002 operator runbook

- Date: 2026-08-24 (follow-up session to the blocked D10 record on
  kernel Draft PR [#266](https://github.com/agentkernel/cognitive-os/pull/266))
- Status: **blocked** on an owner-only GitHub administration action
- Change class: `corrective` (operational record only; no product code,
  contract, negative, or handbook-mapped source is changed)
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit claim; no
  client push or client Draft PR is claimed to exist; running the runbook below
  is not Task completion evidence

## 1. Verified access facts (2026-08-24, this session)

| Fact | Verified result |
|---|---|
| Client repository | `agentkernel/cognitiveos-clients` (public; owner account `agentkernel` is a **user account**, not an organization) |
| Push identity used by Cloud Agents | GitHub App installation token for the `cursor` app (`cursor[bot]`) |
| App installation repository list (`GET /installation/repositories`) | **only** `agentkernel/cognitive-os` — the clients repository is not in the installation, which is the root cause of the recorded HTTP 403 push failure |
| `PUT /repos/agentkernel/cognitiveos-clients/collaborators/cursor[bot]` with the installation token | **403** `Resource not accessible by integration` |
| Installation self-modification with its own token | not possible (`/user/installations/...` requires the account owner's user token) |
| Client commit `07f7513ec45b65a3327bb806f295558934b26049` on the remote | **absent** (`No commit found`) |
| Recovery bundle `provider-webui-apple-theme-8d2f.bundle` (SHA-256 `93d35c3986da919668e60bf5d586da238c7ae9039030668c06408bbee6ed6741`) | **not present** in this follow-up environment's agent store; the bundle exists only in the original subagent session's store |

Conclusion: no credential available to any Cloud Agent session can grant the
write access itself. This is a genuine owner-only boundary, not a recoverable
environment fault.

## 2. Unique owner recovery action

Add `agentkernel/cognitiveos-clients` to the Cursor GitHub App installation
(one-time, account settings). Either route works:

Web console:

1. Open <https://github.com/settings/installations>.
2. Next to the **Cursor** app choose **Configure**.
3. Under **Repository access** add `agentkernel/cognitiveos-clients`
   (keep `agentkernel/cognitive-os`), then **Save**.

Or with the owner's own authenticated `gh` (user token, not an app token):

```bash
INSTALLATION_ID=$(gh api /user/installations --jq '.installations[] | select(.app_slug=="cursor") | .id')
# 1312850564 is the verified repository id of agentkernel/cognitiveos-clients
gh api --method PUT "/user/installations/${INSTALLATION_ID}/repositories/1312850564"
```

After the grant, any Cursor Cloud Agent session holding the original recovery
bundle (or a rebuilt equivalent) can import and push the client revision:

```bash
# in a session where the bundle exists; verify integrity first
sha256sum provider-webui-apple-theme-8d2f.bundle \
  # expect 93d35c3986da919668e60bf5d586da238c7ae9039030668c06408bbee6ed6741
git clone https://github.com/agentkernel/cognitiveos-clients.git
cd cognitiveos-clients
git fetch ../provider-webui-apple-theme-8d2f.bundle 'refs/*:refs/bundle/*'
git checkout -b cursor/provider-webui-apple-theme-8d2f 07f7513ec45b65a3327bb806f295558934b26049
git push -u origin cursor/provider-webui-apple-theme-8d2f
```

If the bundle is unrecoverable, D10 is rebuilt from the design record on
kernel PR [#266](https://github.com/agentkernel/cognitive-os/pull/266) against
clients baseline `db563744f1bfe6b42fa977d59f4ee48a16cee3c2`; the SHA-256 above
then no longer applies and the rebuilt revision must re-run the client
test/build validation before any Draft PR.

## 3. linux-002 operator runbook (dsh agent + pi agent)

"linux-002" is `B01-Desktop-Linux-002` (registry `B01-DESKTOP-002`): the KVM
guest of libvirt host `hal9000` (`wuz@192.168.1.2`), reached via ProxyJump as
`hal9001@192.168.123.160`. It is the sole active B01 campaign guest; the steps
below follow the registered owner-ops procedure (P8-T13/P8-T15 pattern:
runtime root `/home/hal9001/p8t13-owner-ops/runtime`, daemon loopback
`127.0.0.1:48681`, native dsh panel `127.0.0.1:3080`) and change nothing else:
no snapshot, credential, or EVAL-root mutation; no formal B01 attempt.

Boundaries that stay in force while running it:

- do not touch retired `B01-Clean-Linux-001`, EVAL roots
  (`/home/hal9001/perfeval*`), closed EVAL ports
  (`48181`, `48284`, `48286`–`48300`, `48383`, `48386`–`48400`), other
  daemons, or the recorded hung helper PID `430838`;
- never run `secret-tool search`/`lookup`; no key material in argv, output,
  or logs;
- build only from a pushed exact Git revision on the jump host
  (`DEV-LINUX-NATIVE-01`); never copy an uncommitted tree.

The executable step sequence (blocks for the operator machine, the jump host,
and the guest) is recorded in the final session report and reproduced here in
condensed form:

1. **Route check** — `virsh -c qemu:///system list --all` on the host; SSH
   `-J wuz@192.168.1.2 hal9001@192.168.123.160`; confirm guest hostname
   `hal9001-Standard-PC-Q35-ICH9-2009` before any operation.
2. **Exact-revision build (jump host)** — resolve
   `git ls-remote https://github.com/agentkernel/cognitive-os.git refs/heads/main`,
   clone/checkout that exact revision under
   `~/agent-kernel-worktrees/run-<rev8>`, then
   `cargo build --locked --release -p kernel-server -p admin-cli`
   (produces `target/release/kernel-server` and `target/release/cognitive`);
   `scp` both binaries to guest `/home/hal9001/run-<rev8>/bin/`.
3. **Daemon replace (guest)** — `cognitive daemon stop` then
   `cognitive daemon start --runtime-root /home/hal9001/p8t13-owner-ops/runtime
   --bind 127.0.0.1:48681 --kernel-server /home/hal9001/run-<rev8>/bin/kernel-server`;
   verify `daemon status`, `doctor`, and `GET /personal/health` → 200.
4. **dsh agent** — `cognitive dsh status` (reuse the installed `dsh.json`:
   dsh root `/home/hal9001/p8t10-a17edfad/dsh`, pin
   `528c682e061696f5a160f363f236ecbf53cbd006`); `cognitive dsh web
   --host 127.0.0.1 --port 3080 --no-open`; `cognitive dsh apply`; probe
   `GET http://127.0.0.1:3080/` → 200 and `dsh status` `ACTIVE` /
   `process_alive=true`; optional bounded probe
   `cognitive dsh launch --print --path b --task <one-line prompt>`.
5. **pi agent** — `cognitive doctor` first; if `pi` is already configured,
   run the bounded probe `cognitive pi launch --print` with the prompt on
   stdin; if not configured, install pinned
   `@earendil-works/pi-coding-agent@0.81.1` into a cleanable
   `pi-runtime` prefix, build `packages/pi-cognitiveos` at the same exact
   revision, and `cognitive pi configure --executable
   <.../pi-coding-agent/dist/cli.js> --extension-entry
   <.../pi-cognitiveos/dist/index.js>`. `pi launch` is fail-closed on full
   doctor readiness (a `provider=blocked` projection after a dirty daemon
   replace blocks Pi but not dsh Path B).
6. **Health (secret-free)** — `curl http://127.0.0.1:48681/personal/health`;
   `cognitive status`; Personal Web UI `http://127.0.0.1:48681/ui/`
   (management-session gate: paste the daemon bootstrap secret from the
   runtime file, never a Provider key); native dsh panel
   `http://127.0.0.1:3080/`.

Source documents: `docs/plan/PERSONAL-TEST-ENVIRONMENTS.md` §7/§10,
`docs/checkpoints/20260823-personal-p8-t15-dsh-native-web-panel-report.md`
(operator start and post-closure replace records),
`docs/checkpoints/20260820-personal-c1-c2-b01-guest-procedure.md` §1 (control
route), `handbook/en/user/install-and-first-conversation.md` §3–§5, and
`handbook/en/reference/cli-cognitive.md`.

## 4. Unique next action

Owner: execute §2 (add the clients repository to the Cursor app
installation). Everything else — bundle import, client push, client Draft PR,
and flipping the P7-T05/D10 record out of `blocked` on the PR #266 branch —
follows from that single grant and needs no other owner decision.

## 5. 2026-08-24 post-fix verification (append-only)

The reconcile session re-verified every §1 fact after the owner reported the
permission as fixed:

| Fact | Verified result (2026-08-24, ~05:26–05:45 UTC) |
|---|---|
| Repository visibility | now **public** (`visibility: public`); previously private |
| Clients `main` | still `db563744` — no import happened |
| `GET /installation/repositories` (this run's token) | still **only** `agentkernel/cognitive-os` |
| Push probe `cursor/write-probe-8d2f` (twice) and real-branch push | all **HTTP 403** `denied to cursor[bot]` |
| Kernel-repo control probe (branch create+delete) | **pass** — denial is clients-repository-specific |
| Original bundle `93d35c39…ed6741` | **lost** with the original session VM |

Additional root-cause fact: this Cloud Agent environment registers only
`github.com/agentkernel/cognitive-os`, so its run tokens are minted for that
repository alone. Even after the §2 installation grant, publication must run
from an agent whose run/environment covers `agentkernel/cognitiveos-clients`
(e.g. an agent launched on that repository), or the owner imports the bundle
manually. The §2 recovery command remains valid and still has not been
executed, or has not reached this environment's tokens.

## 6. 2026-08-24 independent re-verification from a second run (append-only)

A separate Cloud Agent run (`bc-0c6f2f39-7484-5007-ad1f-f96a2cfd18be`,
environment `9a1980df-9f6c-11f1-a7d1-d6b4613131ce`) re-probed the boundary
while fixing the kernel development environment. Full record:
[Cloud Agent development environment and push/merge diagnosis](20260824-cloud-agent-dev-environment-and-push-diagnosis.md).

| Fact | Verified result (2026-08-24, ~06:20–06:30 UTC) |
|---|---|
| `GET /installation/repositories` | still `total_count: 1` → only `agentkernel/cognitive-os` |
| Clients read (`git ls-remote`) | **pass** — `db563744`, unchanged; public read needs no grant |
| Clients write (`git push --dry-run`, new branch) | **fail** — HTTP 403 `Permission to agentkernel/cognitiveos-clients.git denied to cursor[bot]` |
| Kernel control probe (branch push + delete) | **pass** — denial remains clients-specific |
| This run's `environment.repos` | `["github.com/agentkernel/cognitive-os"]` |
| Recovery bundle `02a0216f…641e` in this run's agent store | **absent** — `/cursor/stores/self/artifacts/` is empty; stores are per-run |

Refinement to §2: the installation grant is necessary but **not sufficient by
itself**. A run receives a token only for the repositories in its own
`environment.repos`, so publication additionally requires a run started from
an environment that lists `agentkernel/cognitiveos-clients` (an agent launched
on that repository, or this environment extended with it as a second repo).
The owner-side bundle import remains the alternative that needs neither.

Because the bundle does not survive across agent stores, whichever run
eventually publishes must rebuild the D10 client work again from the recorded
design source before pushing, and re-run the client test/build validation.

Recovery re-executed measurement-free of the block: the D10 diffs were
re-extracted from the original session's retained transcript, rebuilt as
clients `ba331b8` + `f257819` on `db563744`, revalidated (29/29 tests,
production build pass), and preserved as a new verified bundle in the
reconcile agent's store, SHA-256
`02a0216fd4611b88d904a1c481dc96b1f9ec06f62b99979511c45258250b641e`
(10,471 bytes, tip `f2578196`, base `db563744`). The §2 import example's
expected SHA-256 and checkout tip must be replaced by these values when the
new bundle is used.
