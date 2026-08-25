# Golden fixtures (cross-language)

Two fixture files are the shared truth for the
`cognitiveos.canonical-json/0.1` encoding profile and its schema-bound
layers (`docs/standards/canonical-encoding-and-digest.md` section 14,
ADR-0004):

1. `canonical-json-fixtures.json` — encoding layer: key ordering, escaping,
   Unicode distinction, numbers, negative zero, timestamps pass-through,
   null-vs-missing, nesting, whitespace, digest domain separation, signature
   preimage; negatives for duplicate keys, BOM, invalid UTF-8, unsafe
   integer, NaN and Infinity literals, lone surrogate, control characters,
   trailing content.
2. `digest-and-projection-fixtures.json` — schema-bound layers: digest
   projection (declared `digest_excluded` paths only, pointer escaping,
   absent-path no-op) with self-digest verification; negatives for wrong
   self-field inclusion/exclusion, wrong domain, inserted defaults, missing
   digest; set/bundle manifests (exact canonical bytes, sorted order, domain
   separation, duplicate-id and empty rejection); canonical RFC 3339 UTC
   timestamp FORM negatives (offset, local, lowercase, leap second, trailing
   zeros, zero fraction); digest string form negatives (uppercase,
   truncated, wrong label); unknown-critical-extension gate; pinned schema
   digest verification with an altered-schema negative.

Consumers that must stay byte-identical:

- Rust: `crates/cognitive-contracts/tests/golden_fixtures.rs`,
  `tests/projection_fixtures.rs`
- TypeScript: `packages/contracts-ts/src/golden.test.ts`,
  `src/projection.test.ts`
- CI cross-language gate: both `emit_golden` programs must print identical
  canonical digest maps, including the LIVE schema-bundle manifest digest of
  `specs/schemas/` (`.github/workflows/ci.yml`).

Never edit fixture JSON by hand. Regenerate deliberately (byte-faithful
write, no shell redirection on Windows PowerShell — it adds BOM/CRLF):

```powershell
pnpm --filter @cognitiveos/contracts-ts run build
node -e "const {execFileSync} = require('child_process'); const fs = require('fs'); for (const [arg, file] of [['canonical','tests/golden/canonical-json-fixtures.json'],['projection','tests/golden/digest-and-projection-fixtures.json']]) { fs.writeFileSync(file, execFileSync(process.execPath, ['packages/contracts-ts/dist/dev/generate-fixtures.js', arg])); }"
cargo test -p cognitive-contracts   # Rust twin must re-verify before commit
```

Changing canonical bytes or digests of EXISTING fixtures is a breaking
encoding-profile change (ADR-0004): it requires a new profile version, not
an in-place fixture edit. Adding fixtures is additive (bump the fixture-set
version, as 0.1.0 -> 0.2.0 did for `infinity-literal`).
