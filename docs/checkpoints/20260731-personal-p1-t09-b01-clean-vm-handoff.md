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

Accordingly, the native Secret Service start-gate item is `incomplete`, B01
attempt 1 has **not** started, and the existing reset snapshot remains the
only valid pre-install reset point.

The runner additionally attempted the standard FreeDesktop
`Secret.Service.CreateCollection` call for the `default` alias. The service
returned a `login` alias path, but querying that path failed with the fixed
`object-does-not-exist` class. This is the expected headless-prompt state: a
native GUI Secret Service prompt agent must approve collection creation before
the persistent collection exists. The runner did not retry, approve an
unprotected collection, use a session collection, or enter any Provider
material.

## Assigned human roles

- Operator A: hidden-input Provider credential opt-in only, after all start
  gate items pass.
- Verifier B: independent verifier; must not receive, enter, or inspect the
  Provider credential.

## Required owner action

Use a graphical Secret Service prompt agent on the B01 guest to initialize and
unlock an encrypted persistent default/login collection for `b01operator`,
then confirm that a non-sensitive Product-compatible `secret-tool store` /
lookup / clear cycle succeeds **without** specifying `--collection`. Do not
enter a Provider credential during this preparation.

After that confirmation, the runner can re-run the redacted platform probe,
take a new platform-qualified reset snapshot, register the immutable reviewed
artifact, and complete the remaining B01 start-gate checks before attempt 1.

## Non-claims

No formal B01 attempt, installer execution, Pi configuration, Pi route,
Provider request, Task/Effect/Verification transition, release claim, or
Profile claim has occurred in this slice.
