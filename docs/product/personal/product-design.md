# CognitiveOS Personal Product Design

- Status: canonical stable product intent
- Initial release: Linux x86_64 `1.0.0` through `GMVP-LINUX`
- Architecture: [Personal architecture](../../architecture/personal/README.md)

## 1. Product statement

CognitiveOS Personal is an Agent Shell-led cognitive-resource management
system for one local owner. Users express goals and management intent in
natural language; a deterministic local daemon qualifies Agents and Tools,
binds models and governed data, enforces budgets and permissions, supervises
recoverable execution, and explains results through authority-backed state and
evidence.

It is an operating layer above Linux, not a hardware/driver kernel and not a
multi-Agent launcher that trusts each Agent independently.

## 2. Target users and jobs

### Primary user

A technically capable individual who uses coding or general-purpose Agents but
wants one local place to understand what is installed, what can act, what is
running, what it costs, what changed and whether a result is actually complete.

### Jobs to be done

1. Install Personal and reach a useful first conversation without manually
   distributing credentials among Agent runtimes.
2. Ask in natural language what Agents, models, Tasks, permissions and evidence
   exist, while seeing exact deterministic facts.
3. Turn a goal into a bounded Task with visible scope, budget, side effects and
   acceptance criteria.
4. Supervise, pause, resume, recover, upgrade or remove an Agent without losing
   auditability or duplicating external mutations.
5. Diagnose failure and restore service even when the model, Provider or Agent
   runtime is unavailable.

## 3. Product principles

### Shell first, deterministic fallback always

The Pi-hosted Agent Shell is the default experience. The `cognitive` CLI calls
the same daemon application services and remains available for exact commands,
automation, recovery and model-free operation.

### Preview before authority mutation

Natural language is compiled into a canonical preview containing exact targets,
versions, permissions, budget impact, external mutations and rollback
expectations. The user's admission binds to that digest; changed facts require
a new preview.

### One default approval, hard safety rails

Tier 0 operations are silent, Tier 1 uses a bounded first-use capability lease,
and Tier 2 always requires explicit confirmation. Budgets, epochs, catalog
metadata and state guards enforce policy without an approval chain.

### Installed does not mean permitted

Agent and Tool installation proves only package and compatibility facts.
Workspace, network, model, Tool, Memory and budget scopes are independently
granted and revocable.

### Results must be explainable

Users see authority state, pending/unknown Effects, evidence and verifier
results. A green process exit or fluent Agent response is never presented as
Task completion without acceptance evidence.

### Local by default, secrets stay native

The daemon binds loopback only. Provider/user secret material stays in the
native Secret Store and never enters Agent configuration, SQLite, argv, logs or
support evidence.

## 4. Primary surfaces

| Surface | Purpose | Authority boundary |
|---|---|---|
| Pi-hosted Agent Shell | natural-language goals, inspection, preview and watch | client only; all facts come from daemon |
| `cognitive` CLI | deterministic management and recovery | client only; same application services |
| daemon local API | task and management channels | authenticated front door, not a public network API |
| doctor/support bundle | explain readiness and failures | redacted facts/digests only |

Linux 1.0 has no required Web UI. A future UI remains a client of the same
authority projections.

## 5. Information architecture

The Shell presents six top-level spaces:

1. **Home**: readiness, active Task, blocked actions and health summary;
2. **Agents**: installations, instances, health, permissions and versions;
3. **Tasks**: goals, contracts, progress, budgets, Effects and verification;
4. **Resources**: models, Tools, Context, Memory and artifacts;
5. **Permissions**: capability leases, scope, expiry and revocation;
6. **Evidence**: event timeline, receipts, unknown outcomes and acceptance.

Natural-language requests resolve into this namespace. Names are conveniences;
the preview always shows stable identity and current version before mutation.

## 6. Interaction states

Every long-running operation exposes:

- proposed;
- awaiting clarification;
- awaiting explicit admission;
- accepted/queued;
- running or waiting;
- suspended/blocked;
- reconciling unknown outcome;
- verifying;
- completed, failed, cancelled or quarantined.

The Shell may optimistically acknowledge receipt but cannot fabricate later
states. Detaching stops observation; it does not cancel work.

## 7. Success measures

Release campaigns measure, without converting local samples into claims:

- clean install-to-first-conversation success and time;
- default-path human confirmation count;
- false-completion rate under deliberate wrong implementations;
- duplicate external mutation rate after restart/timeout;
- recovery success and time;
- Agent install/upgrade/rollback/uninstall correctness;
- secret/redaction critical failures;
- actionable doctor/support outcomes.

Formal thresholds and environment requirements remain owned by the development
plan and preregistered campaigns.

## 8. Linux 1.0 product shape

Pi is both the Shell host and the only qualified managed Agent, under separate
identities. Personal obtains the exact official Pi npm package after preview,
verifies and installs it into a private immutable root, registers and
supervises it, and exposes its lifecycle through the Shell and CLI.

The generic adapter framework is part of the engineering deliverable, but
OpenClaw, Hermes, Codex, WorkBuddy and other Agents remain unsupported until
their own qualification and release decisions complete.

See [Linux 1.0 scope](linux-1.0-scope.md) for the exact claim boundary.
