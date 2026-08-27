# 01 — Product model and JTBD

## Evidence boundary

This direction is based on the Owner's requirements, current implementation
audit, and informative source research. There are no five-or-more ICP
interviews, observed workaround/frequency data, behavior analytics, retention,
or willingness-to-pay evidence. Jobs and prioritization are owner hypotheses,
not market validation.

## Primary user and job

One local human Owner operates an OPC or individual development practice.

> When I delegate long-running work, I need one local business console to
> define Projects and digital employees, see what needs me, recover failures,
> control cost and scope, and verify outcomes without learning every Agent
> engine detail.

The daily jobs are:

1. create or revise a Project safely;
2. understand today and decide what needs the Owner;
3. supervise manager/member responsibility and evidence;
4. approve, narrow, reject, or recover consequential work;
5. maintain Knowledge, Provider routes, budgets, and managed runtime health.

## Product objects

```text
Owner
  -> Project
       -> Charter / Goal / Metric / Plan revision
       -> Role Blueprint -> Project Role Assignment
            -> Digital Employee -> Runtime + Conversation + Memory
       -> Routine / Trigger -> Task -> Attempt -> Effect / Artifact / Evidence
```

- Project is a governed long-term workspace, not a directory or chat.
- Role Blueprint is versioned responsibility/capability intent, not a Prompt
  or Agent.
- Assignment specializes a Blueprint inside one Project.
- Digital Employee is a long-lived identity, not a process.
- Runtime is disposable execution; DSH is the default managed Agent.
- Conversation is Personal-owned and non-authoritative.
- Routine defines recurring work; an occurrence/Attempt is one execution.
- Completion requires current independent evidence and daemon acceptance.

## Responsibilities

Every active Project has exactly one current manager. Only the base Project
Manager Blueprint is built in. The Personal Assistant proposes other roles.
Managers may adjust approved subgoals, Tasks, order, frequency, and member
responsibility. Primary goal, team, budget, Provider, tools, permissions, and
external-action rules require an Owner-confirmed revision.

## Terminology

| Advanced term | Default business language |
|---|---|
| Prompt | work instruction |
| Skill | work method |
| Tool | executable action |
| MCP | connected application and capability |
| Loop | work cycle |
| Harness | execution engine |

Technical identity/version/epoch/digest facts remain available in inspectors.

## Desired outcome and non-claims

The design outcome is a fixed-denominator Windows-local Project loop where
every scenario has an explicit state, next action, authority boundary, receipt,
and verification basis. It does not promise business results, full autonomy,
offline-host operation, browser reliability equal to an API, all-platform
publishing, or multi-Agent benefit.
