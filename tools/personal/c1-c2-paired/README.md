# C1/C2 paired measurement instruments (P9-T08)

Measurement-only. These files are not a second authority writer. They must not
create Intent/Effect, Task, Context, Memory, Skill, verification, or acceptance
state. They do not promote Gate, release, Profile, B01, or Agent-benefit.

- `pure-pi-broker.mjs` — campaign-only loopback credential broker for arm `P`
  (execution plan §2.2 option 2). Loopback HTTP, placeholder token
  `campaign-broker-nonsecret-token`, in-memory upstream auth, `retry=0`.
- `linux-secret-service.mjs` / `linux-secret-get-helper.py` — Secret Service
  `get` via D-Bus `SearchItems` + `GetSecret`. Never `secret-tool lookup` or
  `search`. Probe store uses `secret-tool store` stdin only.
- `workspace-fixture-adapter.mjs` — equivalent WorkspaceRead/Search/Write/Patch
  schemas for arm `P`, executing only inside a fixture root.
- `fairness-checker.mjs` / `paired-runner.mjs` / `freeze.mjs` / `redactor.mjs`
  — frozen seeds (`retry=0`, disjoint B0/B1/B2), §2.3 fairness observability,
  mechanical redaction. Not B0 and not a counted sample.
- `prove-linux-secret-get.mjs` — non-B01 Linux proof; prints redacted JSON only.
- Focused tests: `tools/test/c1_c2_paired_p_arm.test.mjs`.
- `frozen-system-task-prompt.txt` — shared UTF-8 prompt whose byte length is
  the dry-run `system_task_prompt_bytes` observation (P9-T09). Live P/O
  launches must pass the same file through `--append-system-prompt`
  (P9-T10): `pi --print --append-system-prompt <absolute-file>` and
  `cognitive pi launch --print --append-system-prompt <absolute-file>`.

Closed EVAL brokers, ports `48286`–`48298` / `48386`–`48398` / `48383`, and
SecretStore items `/12`–`/19` are not reused. B01 samples are forbidden until
packages 6–14 complete and a new EVAL is activated.
