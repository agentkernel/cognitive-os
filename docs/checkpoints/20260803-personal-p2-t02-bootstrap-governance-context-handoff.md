# P2-T02 bootstrap governance-context decision

- Task / slice: `P2-T02/D01`, `lease/personal/P2-T02/bootstrap-governance-context`
  (`in-progress`)
- Change class: `normative-semantic` Personal authority decision
- Branch: `lane/ctr-p2-t02-bootstrap-governance-context`

## Decision

For the first authenticated Task-chain mutation of a local Personal principal,
the daemon may bootstrap a canonical, immutable, persisted governance-root
context. It supplies owner, authority, and ResourceScope anchors to the kernel
and binds them to the authenticated principal. This removes the P2-T02
pre-existing-external-governance-object prerequisite.

## Invariants retained

- bootstrap is daemon-owned, not request-supplied;
- the daemon derives the anchor identities and canonical digests;
- it persists the binding before any Task authority mutation;
- later mutations reload and verify the same principal-bound context;
- missing, corrupt, ambiguous, or principal-mismatched context fails closed;
- clients still cannot provide governance facts, object IDs, actor chains, or
  writer leases; and
- authorization, purpose, CAS, fencing, preview-digest and acceptance checks
  remain required.

## Next implementation

After supported validation and merge, a distinct Lane-RUN slice will persist
and resolve this context from the daemon, obtain server leases, and implement
authenticated intent record/interpret → preview → admit plus process-lifetime
watch. No Task completion, cross-restart watch replay, Gate, release, or
Profile claim is implied.
