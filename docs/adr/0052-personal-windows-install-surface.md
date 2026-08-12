# ADR-0052: Personal Windows Install Surface (Credential Backend, Installer/Service, B01-W)

- Status: Accepted for the credential-backend decision (§1); the
  installer/service (§2) and B01-W campaign (§3) sections are added by later
  P7-T07 slices in the same task before the task closes
- Date: 2026-08-12
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product platform decision for the P7-T07 Windows
  install surface. This ADR is not a registry REQ, schema, transition, vector,
  or Profile claim, and it makes no Windows install parity or B01-W claim
  (ADR-0025 / PERS-PR-021).

## Context

ADR-0025 decided Windows x86_64 as a first-release product platform, but the
entire Windows install surface (credential backend, installer/service, and the
dedicated B01-W gate) was deferred to P7-T07. Until B01-W is actually
executed, Windows remains a daemon/CLI product path only and no install parity
may be claimed.

The Personal secret boundary is already frozen: `SecretStore::{probe, put,
get, delete}` with opaque `SecretRef` (ADR-0018), Provider config holding only
`SecretRef` handles (ADR-0020), fail-closed behavior with no plaintext
fallback anywhere. The Linux production backend is a subprocess adapter over
the OS facility (`secret-tool`), keeping `cognitive-secret` a zero-dependency
crate under the workspace-wide `unsafe_code = "forbid"` policy.

The ADR-0018 local-native Provider secret development exception was Linux-only
and has expired (P2-T08/D02); it never applied to Windows and is not revived
by this ADR.

## §1 Decision: Windows credential store backend

1. The Windows production secret backend is the OS-native **Credential
   Manager** with **generic credentials**, implemented as
   `WindowsCredentialManagerStore` in `cognitive-secret`.
2. The adapter mirrors the Linux subprocess architecture instead of linking
   Win32 APIs from Rust: the Win32 calls
   (`CredWriteW`/`CredReadW`/`CredDeleteW`) run inside a **fixed, audited
   PowerShell helper script** compiled per invocation via `Add-Type`. This
   preserves the crate's zero external dependencies and the workspace
   `unsafe_code = "forbid"` policy.
3. The helper is always started from the **absolute system path**
   `%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe` with
   `-NoProfile -NonInteractive -EncodedCommand`; PATH lookup is never used, so
   a planted `powershell.exe` cannot intercept material.
4. Secret material transits **only child stdin/stdout as hex**, never argv,
   environment variables, config files, SQLite, logs, errors, or evidence.
   The script text is constant except for charset-validated identifiers
   (credential target derived from `SecretAttributes`; the label is
   hex-encoded before embedding). Tokens that could escape a single-quoted
   PowerShell literal are rejected before any spawn.
5. Credentials persist as `CRED_PERSIST_LOCAL_MACHINE` (local to the device);
   roaming persistence classes are never requested, so material cannot sync
   off-host through account roaming.
6. Fail-closed mapping is identical in shape to Linux: non-Windows hosts,
   missing system PowerShell, or a failing helper pipeline probe report
   `Unavailable` and every mutation fails closed; `ERROR_NOT_FOUND` maps to
   `NotFound`; helper input rejection and API failures map to non-secret
   `Backend` errors. There is no plaintext fallback.
7. The generic-credential blob ceiling (`CRED_MAX_CREDENTIAL_BLOB_SIZE`,
   2560 bytes) is enforced before any write; oversized material is rejected
   without a partial write.
8. Production selection (`select_production_secret_store`) prefers the native
   Credential Manager backend on Windows when its probe reports `Available`
   and otherwise stays `Unavailable`. The frozen P1-T02 selection contract is
   unchanged: the ephemeral test double is never selectable, and the
   Linux-signal override with `false` remains fail-closed.
9. Known cost boundary: each helper invocation starts PowerShell and compiles
   the P/Invoke surface (roughly a second per operation). Secret operations
   are rare (init, rotation, daemon Provider-key reads), so this is accepted
   for the MVP surface; any caching optimization is future hardening and must
   not weaken the fail-closed or no-plaintext boundaries.

### §1 Validation

- `CI-WINDOWS-MSVC-01` executes the real Credential Manager round-trip,
  rotation, delete, oversized-rejection, foreign/absent-ref, and redaction
  negatives (`crates/cognitive-secret/tests/p7_t07_windows_credential_store.rs`).
- `CI-UBUNTU-01` executes the non-Windows fail-closed negatives and compiles
  the full adapter.
- Per the environment registry, ordinary CI evidence is implementation
  evidence only; it does not claim a qualified native Windows product install,
  B01-W, install parity, release, or Profile.

## §2 Decision: inspectable installer/service surface

Added by the P7-T07 installer/service slice.

## §3 Decision: dedicated B01-W gate

Added by the P7-T07 B01-W authoring slice.

## Consequences

- The daemon, CLI init, readiness/doctor, and Provider proxy paths gain a real
  native Windows secret backend through the existing
  `select_production_secret_store` composition without any caller rewrites.
- Windows hosts without a usable helper pipeline stay exactly where they are
  today: fail-closed `Unavailable`, no plaintext fallback.
- No Windows install parity, B01-W, Gate, release, or Profile claim is created
  by implementing this backend.

## Rejected Alternatives

1. **`windows-sys`/`keyring` crates with direct Win32 FFI** — requires
   `unsafe` (forbidden workspace-wide) or new external dependencies in the
   deliberately zero-dependency secret crate.
2. **`cmdkey`** — cannot read credential blobs back and would put material in
   argv; both violate the frozen boundary.
3. **DPAPI-encrypted vault file via `ConvertFrom-SecureString`** — reinvents a
   file vault when the OS already provides a native credential store; weaker
   operator visibility than Credential Manager.
4. **WinRT `PasswordVault`** — unreliable on server-class validation hosts and
   historically tied to account roaming semantics; the local-only persistence
   guarantee of generic credentials is explicit and testable.
5. **Plaintext or SQLite fallback when the helper is unavailable** — violates
   the frozen ADR-0018/0020 boundary and Personal axioms.
