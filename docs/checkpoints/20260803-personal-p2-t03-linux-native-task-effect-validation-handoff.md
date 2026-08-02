# P2-T03 Linux-native task-to-Effect validation handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/linux-native-task-effect-validation` (closed)
- Branch: `main`
- Change class: implementation-only evidence closure
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Exact revision and environment

The already pushed revision
`a74ad74856b4cef6d05668acf42832ea18351b8a` was cloned from the remote into a
fresh `mktemp` worktree on the qualified Linux host. The remote command fetched
and detached at that exact hash, compared `git rev-parse HEAD` to the expected
revision, and registered a cleanup trap before any test ran. The command did
not use the stale non-Git `/home/wuz/agent-kernel` source snapshot.

## Validation result

Both focused commands passed:

```text
cargo test -p cognitive-store --test m5_intent_chain
cargo test -p kernel-server scheduler_authority::tests
```

The remote command exited 0 and its cleanup trap removed the disposable
worktree. This validates the durable task-to-Effect lookup on Linux without
running a Provider, Pi, service, secret, B01 guest, installation, or external
mutating operation.

## Scope and non-claims

This is `experimental-local-only` / `tested-local` implementation evidence for
P2-T03. It does not pass B02, B04, B05, B12, GMVP-LINUX, a release, or a
Profile. The task remains `in-progress`.

## Remaining work

- `blocked_paths`: none.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12 and GMVP-LINUX.
- owner: next P2-T03 Lane-RUN/CTR session.
- next action: bind one unambiguous durable Effect resolution into the concrete
  worker closure and exact owner/epoch-fenced scheduler release operation,
  while retaining pending reconciliation and independent Task verification.
