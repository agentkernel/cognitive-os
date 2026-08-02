# CognitiveOS Personal User Journeys

- Status: target Linux 1.0 journeys
- Product boundary: [Linux 1.0 scope](linux-1.0-scope.md)
- Authority behavior: [Personal architecture](../../architecture/personal/README.md)

Each journey separates visible interaction from daemon authority facts. The
Shell may explain or propose; the daemon resolves identities, authorizes,
persists, dispatches and verifies.

## 1. Install Personal and reach first conversation

1. User verifies and runs the Linux installer.
2. Rust verifies the signed bundle, stages immutable bytes, installs the one
   user service and confirms loopback liveness/readiness.
3. `cognitive init` creates data layout, probes native Secret Service and asks
   for Provider key through hidden input.
4. Daemon discovers Provider models and persists a non-secret selected-model
   snapshot.
5. Product previews official Pi acquisition, including package/version/source,
   network use and no automatic runtime capability.
6. User admits acquisition; daemon verifies exact identity/SRI/digests,
   qualifies Pi, commits the installation/registry binding and activates the
   Shell host.
7. Pi mints bounded local sessions and sends completion through the daemon
   Provider proxy.

**Authority objects:** product installation receipt, Provider binding,
acquisition lock, Agent installation/registry/instance facts.

**Failure exits:** invalid signature, missing Secret Service, incompatible Node,
package drift, failed health or Provider probe all fail closed with no active
Pi installation receipt.

**Secret/evidence:** key material stays in native Secret Store; evidence holds
only redacted status, versions and digests.

## 2. Inspect cognitive resources in natural language

User asks: “Which Agents and Tasks are active, what can they access, and what
is blocked?”

1. Pi emits an interpretation candidate.
2. Daemon resolves this as read-only management queries.
3. Shell obtains a management-channel projection and renders stable IDs,
   versions, health, permission scopes, budgets and blockers.
4. Unknown fields remain explicit; Pi does not fill them from conversation.

No confirmation is required for authorized Tier 0 inspection. The management
bearer is separate from the Task bearer.

## 3. Create and supervise a governed Task

1. User states a goal and desired outcome.
2. Daemon durably records raw intent before Pi interpretation.
3. Shell presents a canonical preview: exact workspace, model, Pi instance,
   safe Tool, deadline, retries, steps, cost ceiling, permissions and acceptance
   criteria.
4. User admits the exact digest.
5. Daemon creates Task/Loop and an epoch-fenced Pi `AgentExecution`.
6. Shell watches progress, budget, Effects and evidence projections.
7. Independent verification decides completion.

Changed scope or stale versions require a new preview. Provider success or Pi
`agent_end` never skips verification.

## 4. Pause and resume work

User asks: “Pause this Task after a safe checkpoint.”

1. Shell resolves whether the target is Task, execution or Agent instance and
   shows the distinction.
2. Daemon disables new dispatch under a fresh control epoch.
3. Worker checkpoints bounded state and reconciles pending Effects.
4. Projection reports suspended, blocked or reconciliation-needed.
5. Resume rechecks current permission, budget, package/adapter identity and
   Context, then starts a fresh execution epoch.

Killing the process is not the pause protocol and cannot produce a suspended
authority state by itself.

## 5. Recover after daemon, Agent or network failure

1. User reconnects the Shell or runs `cognitive doctor`.
2. Daemon reloads durable Task, scheduler, installation and Effect facts.
3. Old leases/executions are fenced.
4. Unknown Effects are queried/reconciled with their original idempotency keys.
5. Current policy is reauthorized and compatible checkpoint state restored.
6. Daemon resumes, replaces, suspends or quarantines with an explicit reason.

The Shell retains/resumes its watch cursor when valid and never blindly
resubmits a mutating request.

## 6. Upgrade or roll back Pi

1. User asks to upgrade Pi.
2. Preview shows old/new exact versions, npm source, acquisition digests,
   compatibility result, affected instances/Tasks, permissions and rollback.
3. Daemon acquires a new immutable installation without changing the active
   binding.
4. Health and adapter checks run with no inherited Provider secret.
5. Daemon fences/migrates affected executions and atomically activates the new
   binding.
6. Failed activation restores the prior complete binding; incomplete rollback
   remains a visible durable failure.

The upgrade does not silently grant new Tools or workspace scope.

## 7. Uninstall Pi while retaining Personal data

1. User asks to remove Pi.
2. Preview lists active instances, running Tasks, pending Effects, package
   bytes, capability leases and retained Task/evidence data.
3. Daemon disables new dispatch, suspends/fences Pi and reconciles Effects.
4. Package bytes and active binding are removed; installation becomes removed.
5. Task history, evidence, Provider configuration and secrets remain unless a
   separate explicitly confirmed purge applies.

If pending Effects cannot be safely closed/quarantined, uninstall reports a
blocked or incomplete result rather than a success receipt.

## 8. Upgrade, roll back or uninstall Personal

Product lifecycle is distinct from Agent lifecycle.

- Product upgrade verifies a production artifact, stages it, restarts the
  canonical service, confirms pointer/unit/process/liveness consistency and
  then issues a receipt.
- Failure restores prior binary, unit, pointer and database compatibility.
- Product uninstall previews service, binaries and retained data separately;
  deleting user data is a Tier 2 operation.

Removing Personal never prints or exports secret material.

## 9. Backup and restore

1. `cognitive backup` previews included data and explicitly excludes secrets.
2. Daemon quiesces required writes and produces versioned, digest-bound backup
   metadata.
3. Restore verifies integrity, compatibility and migrations before dispatch is
   re-enabled.
4. User rebinds required secrets through native Secret Store prompts.

An archive containing Provider keys is a critical failure, not a convenience.

## 10. Diagnose and obtain support

`cognitive doctor --bundle` gathers redacted platform, service, database,
Secret Service readiness, Provider model, Pi installation/instance, Task,
Effect and recovery facts. It provides stable error codes, evidence digests and
next actions without prompts, raw Provider traffic, key material, SecretRefs or
sensitive SQLite content.

Support output describes what is known, unknown and not-run. It never converts
a local smoke result into a release or Profile claim.
