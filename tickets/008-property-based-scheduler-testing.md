---
status: done
phase: 1
---

# 008 — Property-based testing of the scheduler and assembler

`docs/DEFINITION_OF_DONE.md` requires randomized/property-based tests for
any component with non-trivial state. See ADR-002 for the proptest choice.

## Acceptance criteria
- [x] Random valid blocks (arithmetic/logic instructions over a small
      register window, so dependency chains actually form) generate a
      dependency graph that is acyclic by construction — every edge points
      strictly backward in instruction index.
- [x] The scheduler executes every instruction in a randomly generated
      block exactly once, regardless of dependency shape.
- [x] Differential test against a reference sequential (program-order)
      executor (`Vm::step_block_sequential`/`run_sequential`, added to
      `vm::Vm` for this purpose): dependency-driven scheduling and plain
      sequential execution reach byte-identical final register state for
      any generated block — the central claim of ADR-001, now proven
      across generated cases instead of only the hand-written examples.
- [x] Wired into `cargo test --workspace` / CI via the existing `ci.yml`
      (no separate CI job needed — proptest runs as a normal test binary).

## Deferred (explicitly out of scope for this ticket, tracked as follow-up)
- Property generation is currently limited to single-block, control-flow-
  free programs (see ADR-002 consequences). Multi-block/branching program
  generation is real future work, not silently dropped — revisit once
  Phase 2/3 produce more assembler-generated test corpora to draw on.
