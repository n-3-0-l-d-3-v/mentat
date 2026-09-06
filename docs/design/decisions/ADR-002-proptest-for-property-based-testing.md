# ADR-002: proptest for property-based testing

## Status
Accepted

## Context

Ticket 008 requires property-based tests proving the dependency-driven
scheduler behaves correctly for arbitrary (not hand-picked) programs, per
`docs/DEFINITION_OF_DONE.md`'s "randomized testing" requirement. Rust's
ecosystem offers two mainstream choices: `proptest` and `quickcheck`.

## Decision

Use `proptest`.

## Alternatives Considered

1. **quickcheck** — simpler API, but shrinking (minimizing a failing case)
   is less configurable and its `Arbitrary` trait is more awkward to
   compose for a structured type like `Block` where instructions must stay
   within a valid register range. Also less actively maintained.
2. **proptest** (chosen) — `Strategy`/`prop_oneof!`/`prop_map` compose
   naturally for generating constrained `Instruction`/`Block` values (see
   `crates/vm/tests/property.rs`), shrinking is automatic and effective at
   finding minimal failing blocks, and it integrates with plain `#[test]`
   functions via the `proptest! { ... }` macro without a separate test
   harness.
3. **Hand-rolled fuzzing** — rejected; reinvents shrinking and case
   generation for no benefit over an established crate.

## Consequences

- New dev-dependency (`proptest = "1"`) on the `vm` crate only.
- The differential test (`scheduled_execution_matches_sequential_reference`)
  depends on `Vm::step_block_sequential`/`Vm::run_sequential`, a reference
  sequential executor added to `vm::Vm` specifically to make this
  comparison possible — see the doc comment on `step_block_sequential` in
  `crates/vm/src/vm.rs`. This is now a real, reusable "conventional PC
  machine" reference mode, not test-only scaffolding, and could be exposed
  through `imc` later (e.g. `imc run --sequential`) if that becomes useful
  for teaching or debugging.
- Property tests currently cover pure arithmetic/logic instructions only
  (no control flow, memory, or division within the generated block) to keep
  the differential comparison well-defined; extending generation to
  multi-block programs with branches is a natural follow-up once Phase 2/3
  need heavier assembler-generated test corpora.
