# Learning Loop (design)

- Status: informative Personal architecture design
- Related: AXIOMS P3, [ADR-0042](../../../docs/adr/0042-personal-three-pillar-engineering.md),
  P4-T01/P4-T04/P4-T05, P8-T06

## 1. Intent

Close the loop across episodes: Agents may propose Skill/Memory candidates from
success and failure experience (Reflexion-family), and only the daemon's
deterministic admission path may create durable objects.

## 2. Flow

```text
episode outcomes / verifier facts
        → Agent or harness emits Skill/Memory proposal (candidate)
        → daemon policy admission (append-only decision)
        → immutable Memory object or Skill revision
        → later Context retrieval under authorization
```

## 3. Constraints

- No self-authorization: proposals never write authority SQLite directly.
- Failure lessons are explainable, forgettable/revocable governed facts.
- Learning never relaxes A1–A8 (especially A2/A4/A6).

## 4. Non-claims

Design only until P8-T06. No automatic Profile or Gate claim from learning
metrics.
