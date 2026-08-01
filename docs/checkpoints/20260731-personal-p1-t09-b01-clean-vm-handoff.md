# P1-T09 B01 clean-VM allocation handoff

- Date: 2026-07-31
- Task: P1-T09 install-to-first-conversation route
- Campaign: `B01-clean-linux-first-install-first-conversation-001`
- Active lease: `lease/personal/P1-T09/b01-clean-vm-execution`
- Task status: `in-progress`
- Gate status: `not-run`
- Claim scope: B01, GMVP-LINUX, release, and Profile remain non-claim

## Allocated environment

A new, dedicated KVM guest was created on the owner-provided host. It is not
`personal-linux-native-01`, and no existing domain was reused.

| Field | Value |
|---|---|
| Libvirt domain | `B01-Clean-Linux-001` |
| Guest operating system | Ubuntu Server 24.04 LTS cloud image |
| Guest architecture | `x86_64` |
| Virtual resources | 2 vCPU, 4 GiB memory, 16 GiB independent QCOW2 overlay disk |
| Network | Isolated libvirt default NAT network; non-public guest endpoint retained only as operator access information |
| Access | `b01operator` SSH-key access; password and root SSH disabled |
| Image integrity | Ubuntu official SHA-256 manifest matched the downloaded image |
| Reset point | `b01-pre-install-baseline` libvirt snapshot |

The pre-install qualification passed for x86_64, Ubuntu 24.04, native
user-systemd, no CognitiveOS CLI/state, and no Pi CLI/state. No CognitiveOS
artifact was selected or installed, no Pi runtime was installed or configured,
and no Provider or user credential was entered.

## Native Secret Service qualification result

The Ubuntu Server cloud image did not initially include `secret-tool`. The
guest was provisioned with `gnome-keyring`, `libsecret-tools`, and its native
user service. The native Secret Service user units are active, and a
non-sensitive sentinel passed set/get/delete in the transient `session`
collection. The sentinel was removed.

This is **not** sufficient to qualify the current Product SecretStore path:
the Product calls `secret-tool store` without `--collection`, and therefore
requires an unlocked persistent default/login collection. This fresh,
SSH-key-only server user has no such collection. A default collection write
failed closed with a fixed missing-login-collection class; no Provider material
was involved or retained.

Accordingly, B01 attempt 1 has **not** started. The original clean snapshot
remains historical qualification evidence only: reverting or rebooting from it
would remove the later native Secret Service preparation.

The runner initially attempted the standard FreeDesktop
`Secret.Service.CreateCollection` call for the `default` alias. The service
returned a `login` alias path, but querying that path failed with the fixed
`object-does-not-exist` class because this cloud image has no prompt agent.
The temporary GUI route was also unsuitable: QXL, VirtIO, standard VGA, and
VMware SVGA all failed under this guest's Xorg mode handling, so no graphical
secret was entered.

Under the owner's approved disposable-agent path, the runner instead used the
native GNOME Keyring encrypted-collection API with a cryptographically random
one-time master value held only in the guest process memory and cleared before
the probe. It created an encrypted persistent collection, made it the default,
and passed the Product-compatible `secret-tool store` / lookup / clear cycle
without `--collection`. Only fixed non-sensitive sentinels were used and all
were cleared. LightDM was disabled afterwards; the same default-collection
probe passed in the restored headless user-systemd session. No Provider or user
credential was entered, printed, logged, persisted outside the native encrypted
collection, or made available to the runner.

## Assigned human roles

- Operator A: hidden-input Provider credential opt-in only, after all start
  gate items pass.
- Verifier B: independent verifier; must not receive, enter, or inspect the
  Provider credential.

## Dedicated Desktop B01 candidate (2026-08-01)

The cloud-image guest's graphical stack failed across QXL, VirtIO, VGA/VESA,
and VMware SVGA, so the runner replaced it with a dedicated Ubuntu Desktop
candidate from the official Canonical image instead of retrying display
drivers on the same guest.

| Field | Value |
|---|---|
| Libvirt domain | `B01-Desktop-Linux-002` |
| Guest operating system | Ubuntu Desktop 24.04.4 LTS (`ubuntu-24.04.4-desktop-amd64.iso`) |
| ISO digest | `3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e`, matched the official `releases.ubuntu.com/24.04/SHA256SUMS` manifest |
| Guest architecture | `x86_64`; non-WSL; `pid 1 = systemd`; user systemd `running` |
| Guest user | `hal9001` (created during desktop install with a password held only by the operator) |
| Network | libvirt default NAT; guest endpoint `192.168.123.160` (non-public access info) |
| Access | SSH enabled with a dedicated `b01_desktop_guest` ed25519 key; password SSH also configured by the desktop install |
| Clean state | no `cognitive` CLI, no Pi CLI or `~/.pi`, no `~/.config/cognitiveos`, no `~/.local/share/cognitiveos` |
| Secret Service | native `secret-tool` present; encrypted login keyring created by the operator's desktop login password |
| Keyring probe | Product-compatible `secret-tool store` / lookup / clear **without** `--collection` passed with fixed non-sensitive sentinel `b01-keyring-sentinel`; post-clear not-found verified; sentinel removed |
| Reset point | libvirt snapshot `b01-platform-qualified-baseline` (shutoff, taken after install, first login, keyring creation, and SSH provisioning) |

This resolves the prior reset-capable keyring blocker: the operator's desktop
login password is the recoverable encrypted keyring master, and the baseline
snapshot is the preregistered clean/reset checkpoint. No CognitiveOS artifact,
Pi runtime, Provider credential, or B01 attempt has entered this guest.

## Remaining start-gate work (owner + runner)

- Select and independently verify an immutable reviewed-`main` campaign
  artifact (source commit, SHA-256, signature, trusted-key/version, Pi `0.81.1`
  pin) before installation.
- Name the operator and independent verifier in the campaign record.
- Record the workload, timeout, attempt ledger location, redacted evidence
  collector, and cleanup procedure; review the B01 runner as a formal Gate
  runner.
- Operator A then performs the hidden-input Provider credential opt-in through
  the approved SecretStore flow on this guest only; no credential is copied to
  chat, argv, config, logs, evidence, or Git.

## Non-claims

No formal B01 attempt, installer execution, Pi configuration, Pi route,
Provider request, Task/Effect/Verification transition, release claim, or
Profile claim has occurred in this slice.
