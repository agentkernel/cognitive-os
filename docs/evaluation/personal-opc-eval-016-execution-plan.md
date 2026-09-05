# PERSONAL-OPC-EVAL-016 — execution plan

- Campaign ID: `PERSONAL-OPC-EVAL-016`
- Status: **closed** (2026-09-05). Measurement-only Control Plane vs frozen prototype v9 / product journeys.
- Lease: `lease/personal/EVAL-20260905/prototype-gap-browser-test`
- Running report:
  [2026-09-05-personal-2.0.0-prototype-gap-browser-test.md](2026-09-05-personal-2.0.0-prototype-gap-browser-test.md)
- Claim ceiling: `hypothesis` / non-claim. Not Gate, release, Profile, B01, or Agent-benefit.
- Product pin (guest + local HEAD at test time): `711a5a7ce8e9f89c6aabcbb7f3d8d7ee098f8fd1`

## 1. Activation

Owner 2026-09-05 directed a live browser evaluation of Personal 2.0.0 Control Plane against the frozen design prototype (`personal-20-opc-e2e-optimized-v9`) and `personal/docs/product/user-journeys.md`. `PERSONAL-PERF-EVAL-015` remains **closed** and is not resumed.

## 2. Isolation

| Resource | Bound value |
|---|---|
| Guest | `B01-Desktop-Linux-002` (`hal9001@192.168.123.160` via ProxyJump `wuz@192.168.1.2`) |
| Control Plane | `127.0.0.1:48681` → `/ui/` (`kernel-server --personal --runtime-root /home/hal9001/p13-main-711a5a7c/runtime`) |
| dsh panel | `127.0.0.1:3080` (`DSH Local Build`; Path B overlay, not `/ui/`) |
| Git revision | `711a5a7ce8e9f89c6aabcbb7f3d8d7ee098f8fd1` (guest `REVISION` file matches local `main` HEAD) |
| Secret handling | runtime `local-bootstrap.secret` only; never Provider keys; never written to Git, report, canvas, or screenshots |

Do not change guest baseline, snapshots, or credential store. Do not modify product code, contracts, tests, or handbook generators.

## 3. Cells

1. Confirm Cursor `cursor-ide-browser` tools (navigate, snapshot, click, fill, CDP, tabs).
2. Confirm tunnel / daemon status / session gate without logging the bootstrap secret.
3. Execute every user journey in `user-journeys.md` §§1–12 against live `/ui/`, plus prototype v9 surfaces and live IA.
4. Probe dsh `3080` for role-confusion only.
5. Publish the running report and owner-facing canvas. No product-code changes.

## 4. Non-claims

This campaign does not promote Gate, release, Profile, B01, Agent-benefit, or Windows native chrome. Live clicks on `B01-Desktop-Linux-002` `/ui/` are measurement evidence only.
