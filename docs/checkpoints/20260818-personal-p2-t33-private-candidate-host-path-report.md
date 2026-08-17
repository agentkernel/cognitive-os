# P2-T33 private-candidate host path (running)

- Task: `P2-T33`
- Branch: `personal/P2-T33-private-candidate-host-path`
- Lease: `lease/personal/P2-T33/private-candidate-host-path`
- Change class: `implementation-only`
- Claim ceiling: `hypothesis` / non-claim
- Document status: D01 implemented locally; Linux/CI validation pending

Owner 2026-08-18 after `PERSONAL-PERF-EVAL-009` close authorized product
changes. EVAL-008 nested `completion.sock` under a long `--runtime-root` and
skipped `private_completion_socket_could_not_be_created`. EVAL-009 used a
short root: the socket was created and the real adapter spawned, then skipped
`private_pi_candidate_adapter_rejected_the_request` because adapter stderr was
`/dev/null` and the private-candidate proxy required a one-key `content`-only
message.

## Discriminant

1. Bind the one-shot completion socket under a short parent
   (`$XDG_RUNTIME_DIR/cognitiveos/pc-<pid>-<seq>.sock`, then process temp,
   then `/tmp/cognitiveos`). Never `chmod` `/tmp`. Keep Pi work/config dirs
   under the long runtime root.
2. Pipe adapter stderr; retain a redacted diagnostic on `daemon.log` skip
   lines (`sk-` / `api_key=` / `token=`).
3. Linux candidate env allowlist after `env_clear()` includes `HOME` /
   locale / `XDG_RUNTIME_DIR` / TLS trust files. Never forward
   `DBUS_SESSION_BUS_ADDRESS` or Provider keys.
4. Private-candidate proxy strips `tools`/`tool_choice` and accepts one text
   choice that may include `role=assistant`. Empty or present `tool_calls`
   still fail closed.

Stub Workspace* on a long root is the focused public-launcher proof (same
composition as P2-T32). It is not C1/C2 真机 and does not promote EVAL, Gate,
release, Profile, B01, or Agent-benefit.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Status | Note |
|---|---|---|
| D01 `private_completion_socket_binds_under_host_path_limit_when_config_dir_is_long` | implemented; `not-run` locally | `RUST-LINK-DEV-WIN-GNU-01`; route to linux-002 / Ubuntu |
| D01 `adapter_rejection_diagnostic_is_redacted_and_bounded` | implemented; `not-run` locally | same |
| D01 `private_candidate_provider_response_requires_one_text_choice` (role+content Ok) | implemented; `not-run` locally | same |
| D01 `private_candidate_request_strips_tool_surfaces_before_forwarding` | implemented; `not-run` locally | same |
| D01 `rejecting_adapter_stderr_is_a_public_daemon_log_fact_on_a_long_runtime_root` | implemented; `not-run` locally | admin-cli focused; Unix only |
| D02 `public_launcher_on_a_long_runtime_root_still_acquires_a_stub_lease` | implemented; `not-run` locally | stub ≠ real adapter |
| Local `cargo fmt --all -- --check` | **pass** | this window, Windows GNU eligible |
| Local `pnpm run check:consistency` | **pass** | 275 requirements / leases verified |
| Local `check:handbook` / `generate-handbook --check` | **pass** | 54 docs × 2 locales; 18 generated pages byte-identical |
| Local `git diff --check` | **pass** | this window |
| Ubuntu required CI | `not-run` | after push |
| linux-002 focused `p2_t33` | `not-run` | exact pushed revision |
| Windows GNU cargo | `not-run` | `RUST-LINK-DEV-WIN-GNU-01` |

No Gate, release, Profile, B01, EVAL, or Agent-benefit claim.
