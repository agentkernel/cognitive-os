# Personal Assistant, Pi, managed DSH, and Agent lifecycle

- Status: informative current/target alignment
- Preserved decisions:
  [ADR-0035](../../../docs/adr/0035-personal-pi-shell-and-managed-agent-role-separation.md)
  and [ADR-0043](../../../docs/adr/0043-personal-universal-agent-adapter.md)
- Current decision:
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)

## 1. Current boundary

Linux 1.0 uses Pi as Shell host and only qualified managed Agent; these remain
distinct identities under ADR-0035. The delivered P8 adapter framework and dsh
Path B remain current post-1.0 facts. They do not qualify Windows managed DSH,
the OPC Personal Assistant, Personal-owned Conversations, or another adapter.

## 2. Personal Assistant and Pi

The Personal Assistant is the global product identity. Pi may run behind it as
a pinned client/sidecar/default-deny engine:

```text
OPC client -> daemon session/application service -> Pi client engine
```

Pi receives bounded Context and deterministic read/proposal operations. It
produces candidates only. It owns no authority, Project/Task, archive,
Conversation, long-term Memory, secret, Tool grant, or completion.

Pi is hidden from the ordinary Installed Agents list. Advanced diagnostics may
show its exact package/product pin, health, and limitations.

## 3. Preinstalled managed DSH Agent

```text
admitted official artifact
  -> immutable installation slot
  -> Personal-managed isolated child
  -> bounded stdio broker
  -> Task/Attempt execution
```

DSH is visible in Settings > Installed Agents with exact source/version/digest,
license/provenance, adapter/broker compatibility, Windows qualification,
health, capability, active/rollback slot, and employee/Task bindings.

DSH is not:

- linked in-process with the daemon;
- a vendored fork;
- a native UI or native Conversation source for Personal;
- an authority/Memory/Secret owner;
- allowed env/plaintext credentials, native MCP/base tools, HMR, or home patch.

Provider traffic is daemon-proxied. DSH receives only bounded Context and
returns candidate/observation output. Personal owns employee Conversation,
archive, Memory, Task/Attempt, Effect, evidence, and acceptance.

## 4. Strict identity chain

| Identity | Owner | Not equivalent to |
|---|---|---|
| artifact/package | upstream + daemon admission | installation, trust, permission |
| installation slot | daemon lifecycle | active runtime or qualification |
| Installed Agent record | daemon management | employee, process, Conversation |
| digital employee | Project domain | Agent package or process |
| runtime instance | daemon runtime authority | employee or Task |
| adapter/broker session | daemon-supervised integration | authority or Conversation |
| Task execution | daemon scheduler binding | process/engine session |
| OS process | host observation | success or completion |
| Personal Conversation | Personal archive scope | engine/native session or Task |
| Personal Assistant session | client interaction | Pi authority or Project employee |

Co-location never merges credential, channel, epoch, budget, or completion.

## 5. Lifecycle operations

Artifact acquisition/admission, install/activate, launch, attach, health,
pause/recover, update, rollback, disconnect, and uninstall remain separate
typed operations. Installation grants no runtime permission. Process kill is
not Task cancel. Runtime restart establishes a fresh epoch and requires Effect
reconcile/context reauthorization before work resumes.

Disconnecting a runtime preserves employee, Conversation, Memory, Task/
Attempt, artifact, and evidence. Uninstalling a managed Agent requires impact,
open Effect, rollback, and retention review.

## 6. Future adapters

Hermes, Codex, Cursor, and other products are future independently qualified
adapter candidates. Each needs exact artifact/license/protocol/capability,
secret, sandbox, lifecycle, platform, and campaign evidence. DSH/Pi evidence
does not transfer. No native conversation synchronization is promised.

## 7. Contract and claim boundary

The current adapter contract remains the foundation. Managed DSH supply-chain/
runtime and Pi Assistant composition are **Requires-backend**; Windows sandbox/
host validation also **Requires-environment**. Any new public machine shape
requires Lane-CTR. This chapter creates no support, qualification, Gate,
release, Profile, or Agent-benefit claim.
