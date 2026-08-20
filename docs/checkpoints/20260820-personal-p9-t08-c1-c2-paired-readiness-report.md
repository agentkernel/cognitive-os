# P9-T08 C1/C2 paired-benchmark readiness — running report

- Task: `P9-T08`
- Lease: `lease/personal/P9-T08/c1-c2-paired-readiness`
- Branch: `personal/P9-T08-c1-c2-paired-readiness`
- Claim ceiling: `hypothesis` / non-claim
- Navigation: [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md)

Per `TEST-REPORT-INCREMENTAL-01`, append each finished validation unit here
before starting the next.

## D01 programme amendment

1. **Owner instruction recovered against canonical sources — pass.**
   `PROGRESS.md` had closed packages 6–9 as assessment-only. Evaluation
   routing was OFF. Closed EVAL-002 and EVAL-004 through EVAL-011 are not
   resumed. Active lease table was empty.
2. **Programme rewritten — pass.** Definition of done is “ready to start B0
   on `B01-Desktop-Linux-002` for paired C1+C2”, not “a B0 may be requested”.
   Packages 6–14 are readiness delivery; 15–17 are the measurement campaign.
3. **Formal task registered — pass.** `P9-T08` is `in-progress` with slices
   D01–D04. Layer 1 counts 98 / 90 / 1 / 1 / 6 / 8.
4. **Isolation reserved, not activated — pass.** `PERSONAL-PERF-EVAL-012` is
   named only as a reserved future EVAL ID. The Owner-directed campaign row
   does not activate it.
5. **D01 durable — pass.** Commit `6ceb8d29` pushed; Draft PR
   [#247](https://github.com/agentkernel/cognitive-os/pull/247).

## D02 P-arm instruments

6. **Failure-first P-arm broker and fixture adapter — pass (local Node).**
   First landing: `node --test tools/test/c1_c2_paired_p_arm.test.mjs` **8/8
   pass**. Broker refuses non-loopback binds and secret-shaped argv/env,
   binds only a non-secret placeholder token, and exposes no
   Context/Memory/Task/retry/verify surface. Fixture adapter schemas match
   O-arm Workspace* names and parameter keys; C1 read/search and C2a
   write/patch run inside a temp fixture root; path escape and preimage
   mismatch fail closed.
7. **Broker HTTP inject + Secret Service fail-closed + freeze/fairness —
   pass (local Node).** Same focused file now **14/14 pass**. Loopback HTTP
   broker injects upstream `Authorization` in memory, records only
   `auth_present`/`auth_bytes`, and never returns material. Linux Secret
   Service `get` fails closed on Windows and on secret-shaped attributes.
   Fairness checker emits pass/fail on §2.3 axes (`b0: false`). Freeze
   ledger has disjoint B0/B1/B2 seeds (180 ids), `retry=0`, and secret-free
   corpus. Redactor refuses unredacted `sk-` evidence. Not B0. Not a counted
   sample. No live Pi Provider call.
8. **D04 documents authored — pass (docs).** B01 guest procedure (no
   mutation), secret bind runbook, cell overlay `cells.json`, and reserved
   EVAL-012 scaffolding
   (`docs/evaluation/personal-perf-eval-012-preregistration.md`). EVAL-012
   is not active. No B01 sample.
9. **Linux Secret Service get into the broker — pass
   (`DEV-LINUX-NATIVE-01`).** Exact pushed
   `7dc8c999729028ddb850ab858e88e2f1ba8d5bf9` at disposable Git clone
   `/home/wuz/p9-t08-c1-c2` (not a closed EVAL or P2-T37 root). Node v22.19.0,
   Python 3.10.12, `python3-dbus` import ok, session bus
   `unix:path=/run/user/1000/bus`, `rustc 1.97.1`, pnpm 10.33.2. Focused
   tests **14/14 pass**. `prove-linux-secret-get.mjs` **ok**: D-Bus get of a
   probe item (suffix `9`, not `/12`–`/19`), in-memory broker inject
   (`auth_present`, 56 bytes, digest match), C1 Read + C2a Write fixture
   calls, `secret_material_written: false`, `secret_tool_lookup: false`,
   `secret_tool_search: false`, probe cleared (`item_count` 0). Proof JSON
   was not secret-shaped. No live DeepSeek call. No B01 sample. No product
   P2 gap: Workspace* schemas cloned in the fixture adapter without daemon
   authority.
10. **Non-B01 fairness dry-run — pass.** Same revision:
    `fairness: pass`, `b0: false`, `retry: 0`, freeze `file_count: 17`.
11. **Leftover remote branch — pass.**
    `git push origin --delete personal/P2-T37-c2a-public-mutation-path`
    succeeded; `git ls-remote --heads origin` no longer lists it (was
    `837f9a4c`).

Next unit: keep EVAL-012 reserved/not active. No B0/B1/B2 samples. Draft PR
[#247](https://github.com/agentkernel/cognitive-os/pull/247) remains Draft
until required CI is green and P9-T08 ready/merge/lease/branch/main can
complete.
