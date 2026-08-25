# ADR-0052: Personal Windows Install Surface (Credential Backend, Installer/Service, B01-W)

- Status: Accepted (§1 credential backend, §2 installer/service surface,
  §3 B01-W gate policy; B01-W execution itself remains preregistered and
  unexecuted)
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
  negatives (`personal/crates/cognitive-secret/tests/p7_t07_windows_credential_store.rs`).
- `CI-UBUNTU-01` executes the non-Windows fail-closed negatives and compiles
  the full adapter.
- Per the environment registry, ordinary CI evidence is implementation
  evidence only; it does not claim a qualified native Windows product install,
  B01-W, install parity, release, or Profile.

## §2 Decision: inspectable installer/service surface

1. The Windows install surface reuses the Linux bootstrap contract instead of
   inventing a second trust design. `personal/deploy/windows/install.ps1` is a
   source-controlled **template** with the same `@COGNITIVEOS_*@` placeholder
   policy surface as `personal/deploy/linux/install.sh`; release automation renders it
   into a reviewed, version-specific script. The unrendered template fails
   closed (exit 64) before any network or filesystem action.
2. Rendered-policy validation is identical in shape to Linux: version charset,
   HTTPS-only object directory without user-info/query/fragment, restricted
   redirect host, and `sha256:<64 hex>` installer digest, all checked before
   any download.
3. Downloads run only through the **absolute** `%SystemRoot%\System32\curl.exe`
   with the same bounded flags as Linux (`--proto '=https'`, connect/transfer
   timeouts, retry budget, `--max-filesize`, explicit single-redirect policy
   against the rendered redirect host). PATH lookup is never used. Partial
   files live in a private owned temporary directory that is always cleaned.
4. The bootstrap delegates only to the digest-verified downloaded
   `cognitiveos-windows-bundle-installer.exe` with the same argument surface
   as the Linux installer (bundle directory, expected release/Pi pins, keyring
   material). The bootstrap itself never touches secret material, never
   registers services, and never elevates.
5. The daemon start model is a **per-user, least-privilege scheduled task**
   (`personal/deploy/windows/cognitiveos-personal-task.xml`): logon trigger,
   `InteractiveToken`, `LeastPrivilege`, restart-on-failure, and the same
   rendered `@COGNITIVEOS_RELEASE_ROOT@`/`@COGNITIVEOS_PERSONAL_HEALTH_PORT@`/
   `@COGNITIVEOS_RUNTIME_ROOT@` daemon arguments as the Linux user service.
   A Windows service (admin, service wrapper, SYSTEM surface) is rejected for
   the per-user product path.
6. Validation boundary: required CI executes the static required/forbidden
   fragment checks everywhere and the behavioral unrendered, version-mismatch,
   malformed-digest, non-HTTPS, and extra-argument rejections plus the
   least-privilege task-XML parse on Windows. The download/delegation path
   stays inspectable-only until B01-W executes with real rendered artifacts;
   CI evidence is implementation evidence, not an install claim.

## §3 Decision: dedicated B01-W gate

1. B01-W is a dedicated Windows first-install/first-conversation gate. It is
   defined by the fixed campaign policy in the preregistration checkpoint
   `docs/checkpoints/20260812-personal-p7-t07-b01-w-preregistration.md`,
   mirroring the ADR-0039 B01 successor rule: fixed denominator **N=6**
   counted clean outcomes, at least **5/6** successes, **zero critical safety
   failures**, a complete aggregate report, and an affirmative independent
   verifier disposition before `pass`.
2. Each attempt is a clean-reset Windows VM journey: rendered bootstrap
   download, verified install, per-user scheduled-task activation, daemon
   readiness, graphical **hidden-input** Provider credential entry into the
   Windows Credential Manager backend (§1), one bounded first conversation,
   and secret cleanup. Credential material never enters argv, files, logs, or
   evidence.
3. Authoring this gate creates no execution. Execution has registered
   prerequisites that do not exist yet and are enumerated in the
   preregistration: a provisioned clean Windows campaign VM
   (`B01-W-DESKTOP-001`), Windows release artifacts (bundle, installer
   executable, signatures) from the release pipeline, and an operator for the
   graphical credential step. Until B01-W actually executes, no Windows
   install parity may be claimed (ADR-0025 / PERS-PR-021), and B01-W cannot
   borrow B01 (Linux), CI, fixture, or local evidence.

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
