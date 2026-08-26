# P7-T06 RC、文档、支持矩阵与声明范围内 B01–B12 — closure

- Task: `P7-T06` / slices `P7-T06/D01`–`D04`
- Status: `done`
- Branch: `personal/P7-T06-rc-docs-support-matrix` (deleted after merge)
- PR: [#276](https://github.com/agentkernel/cognitive-os/pull/276) **merged** at `main@712a517d`
- Content head: `c75ffcd9` (RC binder, runbooks, composition)
- Required-CI content head: `c75ffcd9`
- Required CI: [32954890567](https://github.com/agentkernel/cognitive-os/actions/runs/32954890567) **SUCCESS** (ubuntu, windows, required-ci)
- Lease: closed `lease/personal/P7-T06/rc-docs-support-matrix`
- Change class: documentation + tooling freeze
- Claim ceiling: `hypothesis`
- Non-claims: does not set Gate state; no Profile; no production GitHub Release or production signing ceremony; no Windows B01-W; no Multi-Agent enablement; no B10/MCP in Linux RC claim; no Web UI in Linux RC claim; no B06/B07 benefit or Gate pass; does not mutate `B01-Desktop-Linux-002`.

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Clean Linux VM suite bound | Composition of B01 successor `002` (six-attempt waiver/closure) plus P7-T01/P7-T02 update/rollback/uninstall authority path — not a new B01 guest campaign. [D03](20260826-personal-p7-t06-rc-composition-report.md) |
| All release claims point at evidence digests | Binder `tools/src/personal-rc-gate.mjs`; every evidence observation has `sha256:` binding. `report_digest` `sha256:2c36e4594c4318fa64bfd7017299b7cd858f1e3e33c8f57ae3d99d601acc62c3` |
| `implemented` stays applicable-MUST only; Personal release must not impersonate Profile | Forbidden keys in binder; explicit non-claims; support-matrix honesty |
| Publish install/init/provider/Pi/task/recovery/update/uninstall runbooks | Bilingual `user.rc-and-support` + operations/limitations honesty. No invented public `cognitive update` / `cognitive uninstall` |
| Open critical risks for this RC = 0 **or** explicit NO-GO | `open_critical_risks_for_this_rc: 0`; P6 recorded `disabled-nogo` for this RC (P6-T01..T04 stay not-started) |
| P6 may be explicit NO-GO/disabled and does not block RC | Claim freeze + binder disposition `p6_disabled_nogo` |
| Declared-scope B01–B12 + CI/SBOM/lifecycle/support matrix | [D01 claim set](20260826-personal-p7-t06-rc-claim-set.md); [D03 composition](20260826-personal-p7-t06-rc-composition-report.md) |
| Draft PR → required CI → merge → lease/branch/main | PR [#276](https://github.com/agentkernel/cognitive-os/pull/276) merged at `main@712a517d`; required CI `32954890567` at `c75ffcd9` |

## Slice evidence

| Slice | Outcome |
|---|---|
| `P7-T06/D01` | Claim freeze; binder + failure-first tests **2/2**; P6 `disabled-nogo`; support-matrix honesty; RC-scope critical risks = 0 |
| `P7-T06/D02` | Bilingual runbooks `personal/handbook/{en,zh-CN}/user/rc-and-support.md`; operations/limitations/Web UI honesty after ADR-0054 |
| `P7-T06/D03` | Digest-bound declaration; `suite_digest` `sha256:7edaa50a8da7304b64195c2030d012bf501e3a19e879bf764b2a481a84036cf3`; `trace_digest` `sha256:cd36dd58f12cc9fe12853c66c1d4b01afde825a6765ce92011a0730e9fcb3e23`; `report_digest` `sha256:2c36e4594c4318fa64bfd7017299b7cd858f1e3e33c8f57ae3d99d601acc62c3` |
| `P7-T06/D04` | Required CI green; ready/merge; lease closed; branches deleted; local `main` = `origin/main` |

## Validation

| Unit | Environment | Revision | Result |
|---|---|---|---|
| `node --test tools/test/personal-rc-gate.test.mjs` | local Windows (allowed GNU subset) | `c75ffcd9` | **pass** 2/2 |
| `pnpm -C tools test` | local Windows | `c75ffcd9` | **pass** |
| `pnpm run check:consistency` | local Windows | `c75ffcd9` | **pass** |
| `check-handbook` / `generate-handbook --check` / `docs-sync-gate` | local Windows | `c75ffcd9` | **pass** |
| `git diff --check` | local Windows | `c75ffcd9` | **pass** |
| resolve validation route | GitHub Actions | `c75ffcd9` | **pass** (run `32954890567`) |
| verify (ubuntu-latest) | GitHub Actions | `c75ffcd9` | **pass** (run `32954890567`) |
| verify (windows-latest) | GitHub Actions | `c75ffcd9` | **pass** (run `32954890567`) |
| required-ci | GitHub Actions | `c75ffcd9` | **pass** (run `32954890567`) |
| Local Windows GNU Rust | `DEV-WIN-GNU-01` | — | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Unique next action

Merged PR [#276](https://github.com/agentkernel/cognitive-os/pull/276) at `main@712a517d`. Local and remote task branches deleted; local `main` matches `origin/main`. **Do not auto-claim P6.** Remaining Layer 1 items: blocked `P7-T07` (owner prerequisites for B01-W) and owner-deferred P6 (`disabled-nogo` for this RC; P6-T01..T04 stay not-started). Wait for a fresh owner delivery instruction.
