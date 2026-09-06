# ADR-001: Block-level dependency scheduling instead of a program counter

## Status
Accepted

## Context

The Machine's constraint (`docs/design/CONSTRAINTS.md`) is that it must not
use a conventional instruction-pointer-driven execution model, while still
being able to execute real programs: loops, function calls, memory access,
and exceptions, with deterministic, replayable execution.

A pure token-passing dataflow machine (every instruction is its own node,
control flow is itself data) is the most doctrinaire reading of the
constraint, but it makes ordinary control constructs (loops, recursion)
disproportionately expensive to express and verify for a first phase.

## Decision

Programs are organized into **blocks**. Execution *within* a block is
scheduled purely by data dependency (see `vm::depgraph` and
`docs/design/ISA.md`), with no instruction ordering assumption beyond
"reads see the last write in program order." Control transfer *between*
blocks is resolved by each block's terminator instruction and is the only
place anything resembling a program counter (the "current block index")
still exists.

## Alternatives Considered

1. **Pure instruction-level dataflow across the whole program**, with
   explicit token/continuation passing for every branch. Most faithful to
   the constraint, but effectively requires solving the compiler's
   control-flow-graph-to-dataflow-graph problem as a prerequisite to
   running any program with a loop — too large for Phase 1 to deliver to
   the project's Definition of Done (real workload, tested, benchmarked).
2. **Conventional PC with a fetch-decode-execute loop**, rejected outright
   — it does not attack the constraint at all.
3. **Block-level dataflow with an explicit current-block index** (chosen).

## Consequences

- **Easier:** loops, recursion, and memory access have simple, checkable
  semantics; the dependency graph is small (bounded by block size) and
  cheap to build per block; determinism is trivial to state and test
  (`Trace::first_divergence`).
- **Harder / honest limitation:** the project cannot claim there is *no*
  program counter anywhere in the system — there is a current-block index
  that advances by explicit control transfer rather than by `+1`. This is
  documented plainly in `docs/design/ISA.md` rather than glossed over.
- **Future work:** Phase 3 (`chakobsa`) will need a real
  control-flow-graph-to-block lowering; Phase 9 (`shai-hulud`) may
  revisit finer-grained dataflow if block-level scheduling proves too
  coarse to be interesting there.
