# P2-T03 Linux-native execution routing handoff

- Date: 2026-08-02
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/linux-native-execution-routing`
- Branch: `main`
- Change class: corrective
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Delivered routing correction

Linux daemon, Pi/sidecar, installer, user-service, native integration and
experimental deployment validation now route first to
`personal-linux-native-01` through non-interactive, no-secret SSH. Remote
validation requires a disposable Git worktree at an already pushed exact
revision; Windows static/formatting/documentation checks cannot substitute for
that result.

The SSH host was confirmed as a Linux x86_64 KVM/libvirt host. Its
`B01-Clean-Linux-001` and `B01-Desktop-Linux-002` guests remain reserved for
their preregistered B01 procedures and were not changed by this slice.

## Linux-native validation

The SSH host cloned the already pushed source into the disposable worktree
`/home/wuz/cognitiveos-personal-worktrees/8d7601d`. It checked out and printed
the exact reviewed P2-T03 revision:

```text
8d7601d9fc81c822149d786c1639ff3f2d6219de
```

The following non-secret focused test then passed:

```text
cargo test -p kernel-server scheduler_authority::tests
2 passed; 0 failed
```

This supersedes only the prior handoff's `not-run` reason for this focused
Linux test. It does not change any Task/Gate/release/Profile status.

## Not run

- No user-service deployment or service-manager action was performed.
- No privilege elevation, secret handling, external Provider request or B01
  guest action was performed.
- Full workspace tests, Clippy, protected CI and P2 Gates remain not-run for
  this slice.

## Remaining work

- `blocked_paths`: none for the recorded focused test; deployment requires a
  separate user confirmation before any user-service change.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX.
- owner: next P2-T03 Lane-CTR/KRN or Lane-RUN session.
- next action: establish Loop-scoped, fenced dispatch-disablement and scoped
  pending-Effect closure evidence, then wire scheduler ceiling outcomes to a
  durable STOP before any worker lease.

## Non-claims

This is `tested-local` Linux-native implementation evidence only. It does not
deliver worker dispatch, durable STOP handling, Effect closure, BoundedHarness
integration, a P2 Gate result, B01/B09 result, release or Profile conformance.
