# Golden fixtures (cross-language)

`canonical-json-fixtures.json` is the shared truth for the
`cognitiveos.canonical-json/0.1` encoding profile
(`docs/standards/canonical-encoding-and-digest.md` section 14, ADR-0004).

Consumers that must stay byte-identical:

- Rust: `crates/cognitive-contracts/tests/golden_fixtures.rs`
- TypeScript: `packages/contracts-ts/src/golden.test.ts`
- CI cross-language gate: both `emit_golden` programs must print identical
  canonical digest maps (`.github/workflows/ci.yml`).

Never edit the fixture JSON by hand. Regenerate deliberately with:

```powershell
pnpm --filter @cognitiveos/contracts-ts run build
node packages/contracts-ts/dist/dev/generate-fixtures.js > tests/golden/canonical-json-fixtures.json
cargo test -p cognitive-contracts   # Rust twin must re-verify before commit
```

Changing canonical bytes or digests is a breaking encoding-profile change
(ADR-0004): it requires a new profile version, not an in-place fixture edit.

M0 coverage note: the set covers ordering, escaping, Unicode distinction,
numbers, negative zero, timestamps, null-vs-missing, nesting, whitespace,
digest domain separation and one signature preimage. The remaining
section-14 items (digest projections, set manifests, unknown-critical
extensions) are Lane-CTR / M1 deliverables tracked in
`docs/plan/DEVELOPMENT-PLAN.md`.
