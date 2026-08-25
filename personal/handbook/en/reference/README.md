---
doc_id: ref.index
locale: en
kind: navigation
audience: [user, developer, ai]
generated: false
---

# Reference

Machine-derived references are **generated** by
`node tools/src/generate-handbook.mjs` from implementation sources and registered
contracts — never hand-edited (CI enforces byte equality):

- [`cognitive` CLI](./cli-cognitive.md) — product CLI usage, verbatim from the binary
- [`admin-cli`](./cli-admin.md) — management fallback usage
- [HTTP API](./http-api.md) — every daemon route with method/channel
- [Error codes](./errors.md) — all 55 registered codes
- [Configuration and state files](./config-files.md)
- [Environment variables](./environment-variables.md)
- [State transitions](./state-transitions.md) — the five registered state machines
- [JSON Schemas](./schemas.md) — all machine schemas by `$id`
- [Native Tool catalog](./tool-catalog.md)

Hand-maintained, fingerprint-guarded:

- [Capability status matrix](./capability-status.md) — implemented / partial /
  designed / unavailable across the whole product surface
- [Compatibility](./compatibility.md) — platforms, pins, and support boundaries
