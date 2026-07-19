# CognitiveOS Conformance Assets

This directory contains machine-readable conformance vectors. It is a data package, not an executable runner, and its presence does not claim that any implementation or Profile conforms.

## Nine test layers

1. **Wire/Schema** validates required fields, canonical encoding inputs, digests, versions, and fail-closed handling.
2. **State-machine** covers legal and illegal transitions, terminal states, replay, CAS, and conflicts.
3. **Effect/Recovery** covers idempotency, receipts, unknown outcomes, compensation, and the three mandatory crash points.
4. **Security negative** covers cross-tenant access, capability amplification, expired authority, schema drift, URI-as-authorization, injection, and egress violations.
5. **Context/Semantic** covers required-set completeness, trust labels, loss declarations, freshness, hard budgets, conflicts, and interference regressions.
6. **Harness/Loop** covers TaskContract gates, progress evidence, bounded retries, checkpoints, recovery, verifier reports, and evidence closure.
7. **Knowledge compilation** covers claim provenance, source invalidation, bounded recompilation, self-corroboration, deletion closure, and persistent poisoning.
8. **Performance contract** validates reproducibility manifests, units, denominators, confidence intervals, p50/p95/p99 disclosure, SLO binding, and mechanism/model latency separation.
9. **Management Shell** covers privileged-session binding and expiry, fail-closed authorization gates, untrusted-text isolation, independent approval, idempotent effects, crash reconciliation, and deterministic fallback.

## Manifests and status language

A conformance claim is represented by `specs/schemas/profile-manifest.schema.json`. A manifest pins the specification, requirement and schema bundle digests, encoding/canonicalization profile, suite digest, implementation version, results, degradations, and evidence. Profile values have distinct meanings:

- `implemented`: all applicable MUST requirements have passing evidence or an itemized documented degradation.
- `planned`: intended future work; excluded from conformance coverage.
- `experimental`: available for evaluation but excluded from conformance coverage.
- `unsupported`: no support is claimed.

The registry status `specified` means a normative requirement and its test mapping are defined. It does not mean a runner or implementation exists. A future execution system may additionally track `implemented` for runnable tests; it must not infer that state from these vectors.

## Intelligent Management Shell assets

`specs/schemas/privileged-management-session.schema.json`, `management-action-proposal.schema.json`, and `management-approval-decision.schema.json` define the signed session, proposal, and approval data contracts. The `management-*.json` vectors are declarative scenarios for the optional `intelligent_management_shell` profile, including negative authorization cases and deterministic fallback behavior. They do not provide or imply a Shell, Management API, CLI, test runner, or conforming implementation.

The management error codes used by expected denials are registered in `specs/registry/errors.yaml`. Every vector ID is mapped from its normative `REQ-MGMT-*` entries in `specs/registry/requirements.yaml`; a runner must verify those gates and observable outcomes rather than treating a schema-valid document as a pass.

## Performance and knowledge assets

`specs/schemas/performance-report.schema.json` defines the report contract. `vectors/performance-report-contract.json` is a declarative schema example; knowledge vectors exercise invalidation, poisoning isolation, and bounded maintenance. These files are evidence formats and test cases, not measured implementation results. A profile manifest uses `cognitiveos_conformance.performance_reports` to reference digest-pinned reports.

The current namespace is `cognitiveos.*` and the manifest root is `cognitiveos_conformance`. Legacy `agentos.*` or `agentos_conformance` documents require an explicit old schema/adapter and cannot be silently mixed.

## Running

There is currently no conformance runner in this repository. `vectors/*.json` are declarative inputs and expected outcomes for runner authors. At minimum, consumers can parse every JSON document, validate schemas against JSON Schema draft 2020-12, resolve every relative `$ref` from the containing schema, parse both YAML registries, and verify that each vector's `requirement_ids` and error codes exist in the registries.

Example validation with suitable Python packages installed:

```powershell
python -m pip install jsonschema PyYAML
python scripts-or-your-runner.py  # repository currently provides no runner
```

Do not report a vector as passed merely because the JSON parses. A runner must execute the stated input against an implementation, compare the observable result with `expected`, preserve evidence, and report pass, fail, not-applicable, or documented-degradation for each applicable requirement.

## Fourteen test layers
原九层继续有效，并增加：
10. **Agent installation/adapter**：package provenance、C0—C3 feature matrix、sandbox interception、Conversation isolation、completion candidate 与 recovery reconciliation。
11. **Governed memory**：candidate admission、跨 scope promotion、conflict preservation、retention/deletion、derived invalidation。
12. **Cognitive discovery**：manifest discover/read separation、candidate admission、delta scope/budget、stagnation 与 existence privacy。
13. **Operation catalog**：snapshot lifecycle、summary/descriptor binding、effect-class ambiguity、dry-run 与 catalog drift。
14. **Semantic mediation/CRB**：soft-signal isolation、deterministic envelope、fallback、egress、parent budget 和 hard-bound enforcement。

五个新 Profile 的 declarative vectors 仍只是场景数据，不表示安装器、Memory Service、Catalog、SMS、CRB 或测试 runner 已实现。每个新 vector 的 requirement/error 必须存在于 registry，profile manifest 必须分别声明 Profile 状态、C0—C3 feature matrix、semantic service level 与 degradation。
