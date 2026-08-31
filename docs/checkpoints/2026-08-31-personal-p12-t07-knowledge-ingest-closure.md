# P12-T07 Knowledge ingest — closure

- Task: `P12-T07` / slice `P12-T07/D01`
- Branch: `personal/P12-T07-knowledge-ingest`
- Lease: `lease/personal/P12-T07/knowledge-ingest` → §3.1
- PR: [#300](https://github.com/agentkernel/cognitive-os/pull/300)
- Content: `736fcbcb`; docs-head `fefd6872`
- Required CI: [33401268090](https://github.com/agentkernel/cognitive-os/actions/runs/33401268090) **SUCCESS** at `fefd6872` (resolve 3s, ubuntu 3m22s, windows 12m53s, required-ci 3s)
- Change class: `implementation-only`
- Claim ceiling: `hypothesis`

Knowledge ingest lives on daemon `/ui/` Knowledge. Owner-paste POSTs `/management/project/v1/vault.import` then `vault.index.rebuild`. Why this fragment reads GET `vault.index` inject_order + excerpts. Import failure keeps the original fields. Files are not Project authority (`vault.apply-authority` stays off the client whitelist). Secret-shaped paste is not POSTed. Obsidian is not bundled. Dual Track TS: personal-web-ui **391/391**. GNU cargo **not-run**. NVDA/200%/host-theme **not-run**. Host FS E2E **not-run**. Native UI E2E **not-run**. Product origin is daemon `/ui/`. Not T08 connections. Not T15.

Unique next: merge #300, then claim `P12-T08` (Settings connections + don't-ask-again + CloseBackground).
