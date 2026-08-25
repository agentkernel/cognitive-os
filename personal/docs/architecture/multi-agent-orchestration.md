# Multi-Agent Orchestration (design)

- Status: informative Personal architecture design (mainline, fail-closed default)
- Related: [ADR-0044](../../adr/0044-personal-multi-agent-mainline.md),
  [agent-adapter-contract.md](./agent-adapter-contract.md)

## 1. Intent

Multiple independently qualified Agents may contribute **candidates** into one
owner-local Task under daemon arbitration. Personal remains the sole authority
writer. Multi-agent is design-mainline for 2.0; it is **not** a Linux 1.0 claim.

## 2. Roles

| Role | Authority |
|---|---|
| Owner / management session | enable/disable collaboration, budgets, scopes |
| Daemon scheduler | admit, fence, budget, WIA/continuation, Effect, verify |
| Agent A..N (via sidecars) | propose candidates / observations only |
| Independent verifier | completion criteria against durable facts |

## 3. Collaboration patterns (initial)

1. **Single primary + helpers:** one Agent holds the conversational shell; peers
   may be invoked for bounded sub-proposals.
2. **Parallel proposal race:** daemon admits at most one Effect path per
   idempotency key; losers become audited non-committed candidates.
3. **Sequential handoff:** continuation authority is daemon-issued; Agents never
   transfer leases to each other.

## 4. Isolation rules

- Separate sidecar sessions and process identities per AgentInstance.
- Shared Context/Memory/Skill access still revalidates authorization per body.
- Default posture: multi-agent collaboration **off** until owner enables and
  each participant is qualified.
- NO-GO for a specific Agent/campaign remains legitimate; it does not erase the
  architecture chapter.

## 5. Non-claims

No B11 pass, no multi-agent runtime in 1.0, no second authority plane.
