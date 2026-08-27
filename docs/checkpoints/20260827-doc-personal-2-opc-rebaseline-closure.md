# DOC-PERSONAL-2.0-OPC closure handoff

- Status at handoff: acceptance satisfied; final Git closure in progress
- Formal anchor: `P11-T01/D01` documentation-only
- Branch: `personal/DOC-PERSONAL-2.0-OPC-rebaseline`
- Content revision: `33a35f9fdc81625911efb279490e5b1e42baf826`
- Upstream: `origin/personal/DOC-PERSONAL-2.0-OPC-rebaseline`
- Pull request: [#280](https://github.com/agentkernel/cognitive-os/pull/280)
  (Draft at this handoff)
- Lease: `lease/personal/P11-T01/opc-doc-rebaseline` (active until merge
  closure)
- Change class: `product-semantic` with architecture/planning follow-through
- Claim ceiling: `non-claim`

## Delivered

1. Accepted
   [ADR-0059](../adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
   records Windows-first OPC scope, Project/Role/Employee/Routine/Conversation/
   Memory boundaries, managed DSH, Pi Assistant, Provider/budget separation,
   X connector constraints, 2.1 deferral, and exact ADR-0056/0058 partial
   supersession.
2. Replaced the canonical Personal 2.0 product corpus with OPC semantics and
   added focused Project, Knowledge/Memory/Vault, long-running-operation, and
   exact OSS-reference chapters.
3. Preserved the former Personal target under
   `personal/docs/product/legacy-agent-stewardship-20260827/`; Linux 1.0
   remains finalized and unchanged except a forward non-claim pointer.
4. Archived all 44 previous client design/audit/planning documents under
   `clients/docs/design/legacy-control-plane-20260827/` and published one
   current twelve-document `clients/docs/design/opc-2.0/` interaction corpus.
5. Added the external interactive Canvas
   `C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-2-opc-product-prototype.canvas.tsx`
   (39,984 bytes; SHA-256
   `e368e6ed309ac9d6a3f15b1981cb3e2bb2074a1c6afad433ec40c51976dc1b94`).
6. Reworked Personal architecture and added Project/Role/Employee,
   Conversation/Memory/Vault, Windows host/background, and
   Routine/Trigger/missed-run chapters.
7. Preserved P10-T01/T02 done facts; cancelled P10-T03..T18 before
   implementation with explicit successors; registered P11-T01..T15 with
   typed dependencies, vertical slices, detailed task cards, negatives,
   validation routes, and claim ceilings.
8. Updated trace PERS-PR-039..047, support matrix, Windows environment route,
   Current snapshot, bilingual handbook, glossary, source map, coverage, and
   30 source fingerprints. Generated pages remain byte-identical.

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| product + scope + final Installed Agent semantics | canonical product index/design/scope/Agent/Knowledge/operations/Provider/resource docs; ADR-0059 |
| current/frozen migration with one active definition | both frozen indexes plus current product/client indexes; consistency/link review |
| interaction design and non-happy states | twelve-document OPC corpus; all required state classes; Requires-backend matrix |
| interactive prototype | external Canvas with scene/state/composer controls; Cursor TypeScript no-error result |
| architecture and ADR supersession | ADR-0059 and architecture index/chapters; ADR-0058 `0.1` non-reinterpretation |
| formal Phase 11 implementation-ready plan | formal task/typed dependency/slice tables; detailed `plan.md` cards; trace PERS-PR-039..047 |
| support/environment honesty | support matrix and `DEV-WINDOWS-NATIVE-OPC-01` not-provisioned route; B01-W separate |
| handbook/docs sync | 58 × 2 handbook pass; 18 generated locale pages byte-identical; new OPC source-map/coverage routes |
| local documentation validation | running report R12–R24 |
| required CI | [run 33085916115](https://github.com/agentkernel/cognitive-os/actions/runs/33085916115) SUCCESS on content revision `33a35f9f` |

## Validation

- `pnpm run check:consistency`: pass
- `pnpm run check:handbook`: pass (58 documents × 2 locales)
- `node tools/src/generate-handbook.mjs --check`: pass (18 pages)
- `node tools/src/fill-handbook-fingerprints.mjs`: final 0 drift
- `node tools/src/docs-sync-gate.mjs --staged`: pass
- Markdown links/anchors/fences/terminology: pass after 41 archived-link
  repairs
- `git diff --cached --check`: pass
- IDE diagnostics: no errors
- Canvas TypeScript check: no errors
- Required Ubuntu/Windows CI: pass on content revision
- Local Rust build/test/Clippy/link: not-run by policy on `DEV-WIN-GNU-01`

Full incremental results, including the retained initial consistency failure,
are in
[the running report](20260827-doc-personal-2-opc-rebaseline-report.md).

## Non-claims

No product implementation, public contract, negative vector, test code,
Windows host, DSH artifact/sandbox, Personal Assistant, Project, Conversation,
Vault, Routine, Provider enforcement, UI, connector, or remote capability was
implemented or qualified. No support, Gate, release, Profile, B01-W,
containment, market, usability, performance, 24/7, business-outcome, or
Agent-benefit claim follows.

## Remaining deterministic Git closure

1. commit and push this closure update on the same branch/PR;
2. require final-head Ubuntu/Windows CI;
3. change PR #280 from Draft to ready and merge normally;
4. on merged `main`, close the lease and mark P11-T01/D01/task done;
5. delete the safely deletable remote/local branch, fast-forward local `main`,
   and verify only owner-owned untracked `.cursor/skills/**` remains.

No P11 implementation task may be claimed as part of that closure.
