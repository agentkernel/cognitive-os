# P5-T05/D04 B09 execution evidence (ADR-0047)

- Task: `P5-T05`
- Slice: `P5-T05/D04`
- Campaign: `B09-managed-pi-sidecar/1`
- Policy: ADR-0047 MVP fixed denominator
- Exact revision: `548f138da25db93ef13aff891dc043ffaf2d4678`
- Environment: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`) via git bundle
  `/tmp/p5-t05-548f138.bundle` (GitHub HTTPS fetch failed with HTTP2 framing)
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/183
- Date: 2026-08-11

## Matrix results (all pass)

| Observation | Result |
|---|---|
| process_bound_on_activate | pass |
| unbound_registered_health | pass |
| pause_stop_clear_binding | pass |
| stale_epoch_preserves_binding | pass |
| process_bound_blocks_upgrade | pass |
| process_bound_blocks_uninstall | pass |
| pin_drift_rejects_activation | pass |
| stop_then_uninstall | pass |
| install_neq_permission | pass |
| identity_separation | pass |
| orphan_no_reattach | pass |
| Non-claim harness (`tools/test/b09-managed-pi-gate.test.mjs`) | pass 2/2 |

Focused Linux commands at `548f138`:

- `cargo test -p cognitive-runtime --test p5_t05_process_bound --test p5_t05_upgrade_fencing --test p5_t05_identity_recover` → 11/11
- `cargo clippy -p cognitive-runtime -p cognitive-store --all-targets -- -D warnings` → pass
- `node --test tools/test/b09-managed-pi-gate.test.mjs` → 2/2

## Non-claim report digests

- `suite_digest`: `sha256:1b9a72d1e0e6bee190208bae6a4b24f5d13a650c1a8df76835dfc4564a1aeddb`
- `trace_digest`: `sha256:7ec593e6f6a23acf610d0f32317d0b3da59b124876ade0eab313d878735c7dbe`
- `report_digest`: `sha256:3248ff142fe8672ce8fdacce1284762e8c79ff07d785a36e92220eb7c23cd091`
- `claim_scope`: `non-claim` (evaluator cannot set Gate state)

## Required CI

- Required Ubuntu/Windows CI run `31423464703` passed for PR #183 head
  `ed1d1a99d90e853c9abd7d55e82b5e13a62570e7` (includes matrix revision
  `548f138` plus evidence/docs sync).

## Disposition

Owner session reply `affirm B09` (2026-08-11) records B09 MVP `pass` under
ADR-0047. See closure checkpoint
`docs/checkpoints/20260811-personal-p5-t05-b09-managed-pi-closure.md`.

## Non-claims

No live production process supervision, non-Pi adapter qualification,
GMVP-LINUX, release, or Profile claim.
