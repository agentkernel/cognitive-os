 # P2-T05 native Tool registry closure

 - Date: 2026-08-07
 - Classification: `implementation-only`
 - Task: `P2-T05`
 - Branch: `personal/P2-T05-native-tool-registry`
 - Lease: `lease/personal/P2-T05/native-tool-registry-validation`
 - Draft PR: [#159](https://github.com/agentkernel/cognitive-os/pull/159)
 - Acceptance checkpoint: `72a7e55e5a780827438bfb0fb42172cfd1e5bec1`

 ## Acceptance mapping

 P2-T05 requires a useful native Tool family for workspace read/search/write/
 patch, bounded process/check, and read-only HTTP fetch; each operation must
 bind immutable descriptor/version/digest/risk facts, while unknown, drifted,
 disabled, and quarantined Tools remain dispatch-ineligible.

 - `crates/cognitive-kernel/src/tool_registry.rs` defines the daemon-owned,
   static six-operation catalog. There is no runtime registration interface.
 - `resolve_native_tool` checks the operation ID, action, descriptor version,
   risk, availability, and canonical digest before returning a resolved value.
   The focused negatives cover unknown IDs, action/version/digest drift, and
   disabled/quarantined descriptors.
 - `resolve_persisted_native_descriptor` rejects drift between durable daemon
   descriptors and the catalog's executor, effect, and recovery facts before
   admission can consume the descriptor.
 - `validate_workspace_operation`, `validate_process_check`, and
   `validate_read_only_http_fetch` are bounded pre-executor validators. They
   reject path escape, unregistered processes, oversized inputs, mutable HTTP
   methods, non-HTTPS URLs, unregistered origins, queries/fragments, and
   invalid time limits without accessing a filesystem, spawning a process, or
   making a network request.
 - The private resource projection exposes catalog facts as read-only daemon
   observations; it is not a Tool executor or an authority bypass.

 ## Validation

 - Local Rust build/test/Clippy: `not-run`; `DEV-WIN-GNU-01` is an unsupported
   Rust linking host, so no known linker failure was repeated.
 - Exact native Linux `DEV-LINUX-NATIVE-01`: passed at immutable
   `72a7e55e5a780827438bfb0fb42172cfd1e5bec1` in disposable Git clone
   `/tmp/cognitiveos-p2-t05-72a7e55`:
   - `cargo test -p cognitive-kernel tool_registry` -- 7 passed;
   - `cargo fmt --all -- --check` -- passed.
 - Required ordinary supported CI: Ubuntu and Windows/MSVC verification jobs
   passed for PR #159, GitHub Actions run `31158456727`.

 ## Recovery provenance

 The owner selected dangling revision `1ced05936464f0256b6b461a9519be75a9169aa5`
 as the recovery source. Its P2-T05 implementation had already been subsumed
 by later merged Tool work. Compared with this closure checkpoint, its remaining
 `tool_registry.rs` delta only replaces a fail-closed digest initialization with
 an `expect` panic and shortens test setup; it does not restore a missing
 acceptance behavior. Its unrelated P2-T04 Pi/candidate files were deliberately
 excluded from this single-task P2-T05 branch.

 ## Non-claims and next dependency

 P2-T05 closes the pre-executor catalog and validator boundary only. It does
 not execute a Tool, access external I/O, mutate a workspace, create an
 Intent/Effect, reconcile an unknown outcome, create progress/evidence,
 verify or complete a Task, pass a Gate, establish a release, or establish a
 Profile claim. P2-T06 is the next owner for executor/supervisor, dispatch,
 persistence, and reconciliation behavior.
