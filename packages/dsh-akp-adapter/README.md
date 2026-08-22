# `@cognitiveos/dsh-akp-adapter`

This package is the client-side bridge for DeepSeek Harness (`dsh`). It keeps
the dsh plugin API at the edge and sends only bounded, candidate-only events to
the CognitiveOS AKP boundary. It does not hold a daemon bearer in plugin
payloads, a Provider key, an authority writer, or Task-completion capability.
HTTP bearers are supplied by the harness constructor and are never logged.

Pinned identity:

- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- bridge protocol: `cognitiveos.dsh-akp/0.1`

The wire format is snake_case JSON so the Rust daemon can parse it. Frames are
capped at 1 MiB. `JsonlAkpTransport` is the long-lived child-process transport;
`HttpAkpTransport` posts to authenticated `POST /task/akp/dsh`. HTTP sessions
must be activated after daemon start; a restart empties the process-local
session table and fails closed.

Workspace* candidates are mapped by the daemon onto the existing public
candidate admission path using the native catalog (`native.workspace.read` /
`search` / `write` / `patch`). WorkspaceRead is parameter-free; Search needs a
query; Write/Patch need canonical `input_b64` and `preimage`. Observation and
lifecycle events are accepted without authority writes. A dsh response is never
Task completion.

The adapter records serialization, transport, and total durations separately.
`scripts/linux002-e2e.mjs` drives attachDshCordisPlugin over HTTP against a
live Personal daemon (shim host, not the dsh CLI) and waits for Task
`COMPLETED`. `src/plugin.ts` is the Cordis `apply` entry for `dsh --patch`.
`scripts/dsh-real-process.mjs` boots pinned headless dsh via
`apps/cli/lib/bin.js` when host `build:lib` outputs are present (the published
CLI entry). If those compiled files are absent it falls back to
`node --import tsx/esm apps/cli/src/bin.ts`. It does not call `pnpm dsh`, which
is not portable on a guest without git. On a 2 vCPU guest the tsx-from-source
path spends ~10 s bootstrapping the harness before any Provider byte.
Installed Path B loads `plugin.bundle.cjs` (CommonJS
bundle of `src/plugin.ts`) because Node 22.23 `require()` of `dist/plugin.js` fails with
`ERR_REQUIRE_CYCLE_MODULE`. Regenerate the bundle with
`npx esbuild src/plugin.ts --bundle --platform=node --format=cjs --outfile=plugin.bundle.cjs`
when `src/plugin.ts` or `src/index.ts` changes. The helper admits disposable Workspace* Tasks, submits those candidates as plugin `startupEvents`, activates
`POST /task/akp/dsh` with a task-channel bearer file, and points `llm-deepseek`
at `POST /provider/v1/chat/completions` with a management-channel credential
ref. dsh always requests SSE; the public daemon proxy forwards `stream:true` as
HTTP/1.1 SSE (no SSE-to-unary bridge on Path B). The helper pins Flash
`thinking: disabled`, `reasoningEffort: off`, and `maxTokens: 256`.
The Provider key stays in SecretStore. Timing fields are measurement hooks, not a zero-overhead claim.
Live linux-002 and jump-host samples are implementation evidence, not a Gate,
release, Profile, B01, or Agent-benefit claim.

Product install is `cognitive dsh configure` then `cognitive dsh launch`
(Path B). Configure writes `{dsh_root}/.cognitiveos-dsh-revision` and
`config/cognitiveos/dsh.json` (pin, adapter root, candidate-only digest). The
digest is not SQLite-durable daemon adapter state. Launch fail-closes `--path a`;
same-host Path A vs Path B measurement uses `scripts/paired-path.mjs` with
`--api-key-file` (stdin `-` or a 0600 file). A dsh response is never Task
completion.
