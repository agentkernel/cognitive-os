# F-017 Platform / Channel Sandbox Matrix (M6)

> Status: **closed-for-release-claim-set** (2026-07-21 M6-EXIT Batch-E1).
> Broader channel coverage remains `not_tested` / `unsupported` and must not be
> silently claimed. Do **not** merge WSL2 guest results into a Windows-native claim.

## Claim freeze (v0.1 release declaration set)

Frozen by [M6-EXIT-PLAN.md](../plan/M6-EXIT-PLAN.md) WP-CLAIM. Only rows in this
table may appear in release claims.

| Platform | Channel | Claim | Evidence digest | Reproduce |
|---|---|---|---|---|
| `linux_native` | network (unmediated) | `denied_with_evidence` | `sha256:evidence-network` | see below |
| `linux_native` | secrets (`host://`) | `denied_with_evidence` | `sha256:evidence-secrets` | see below |
| `linux_native` | tool_proxy (unregistered) | `denied_with_evidence` | `sha256:evidence-tool_proxy` | see below |
| `linux_native` | remaining channels | `not_tested` | — | non-claim |
| `windows_wsl2_linux_guest` | * | `not_tested` | — | non-claim (Linux guest only if later tested) |
| `windows_native` | * | `unsupported` | — | no native containment backend |

### Reproduce commands (claimed deny rows)

```bash
# Unit negatives + stable matrix digests (Linux native reference)
cargo test -p cognitive-runtime --lib sandbox::tests -- --nocapture

# Behavior vector (Linux native bypass negatives; counted in runner pins)
cargo run -p cognitive-conformance --bin conformance-runner
# Expect AGENT-BYPASS-002 result=pass; report under
# artifacts/evidence/conformance/conformance-report.json
```

Digest strings are emitted by `SandboxGate::matrix_rows()` for channels in
`evidenced_denials` and pinned by `f017_claim_freeze_digests_are_stable`.
Cross-platform merge is refused by `refuse_cross_platform_merge`
(`AGENT_ADAPTER_BYPASS_DETECTED`).

## Claim discipline

| Platform row | Meaning | Merge rule |
|---|---|---|
| `linux_native` | Reference Linux containment | May claim deny/degrade only with evidence digest |
| `windows_wsl2_linux_guest` | Windows host running Linux guest via WSL2 | Represents **Linux guest** only |
| `windows_native` | Windows-native containment backend | Without native evidence → `unsupported` / `not-tested` |

## Closure criteria (from M6-PLAN `M6-F017` / M6-EXIT)

1. Every claimed deny/degrade row has a reproducible command + digest. **met** (claim freeze table)
2. Linux native reference negatives cover declared matrix dimensions used in release claims. **met**
3. WSL2 rows explicitly labeled Linux guest. **met** (`not_tested`)
4. Windows native remains `unsupported` until a native backend exists. **met**
5. Cross-platform merge refused (`refuse_cross_platform_merge`). **met** (unit)

Relative to the frozen release claim set, F-017 is **closed**. Expanding claims
beyond this table re-opens F-017 until new digests land.
