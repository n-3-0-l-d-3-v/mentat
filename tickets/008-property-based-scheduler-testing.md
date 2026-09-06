---
status: open
phase: 1
---

# 008 — Property-based testing of the scheduler and assembler

`docs/DEFINITION_OF_DONE.md` requires randomized/property-based tests for
any component with non-trivial state. The dependency graph and scheduler
currently only have hand-written unit/integration tests.

## Scope
- Generate random valid blocks (random register read/write patterns) and
  assert: the dependency graph is always acyclic; the scheduler always
  executes every instruction exactly once; final register values match an
  independent naive-sequential-order interpreter run on the same block
  (differential testing per project philosophy, section 31).
- Generate random valid multi-block programs (bounded size, guaranteed
  terminating via a decrementing counter) and assert two independent runs
  always produce identical traces.
- Wire into CI as a `cargo test` target (proptest or quickcheck — pick one
  and record the choice in an ADR).

Not started. Do not mark done until it clears `docs/DEFINITION_OF_DONE.md`.
