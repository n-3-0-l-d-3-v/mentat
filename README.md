# impossible-machine — THE MACHINE

> A dependency-driven virtual machine with no instruction pointer.

Part of **[The Impossible Computer](https://github.com/n-3-0-l-d-3-v/impossible-computer)** — a constrained computing
ecosystem built by removing assumptions ordinary computers depend on. This
repository is developed standalone and mirrored into the combined ecosystem
repo commit-for-commit.

## Status

**Phase 1 — COMPLETE** (tickets 001–008 all done; see [tickets/](tickets/)).
The execution model is real and tested end-to-end: ISA encoding, the
dependency-graph scheduler, bounds-checked memory, traps, a two-pass
assembler, and an `imc` CLI (assemble/disassemble/run/replay/debug/trace/
profile) all work against real programs (loops, recursive function calls,
memory-through-calls) — see `examples/`, `crates/vm/tests/integration.rs`,
and the property-based differential tests in `crates/vm/tests/property.rs`
that prove the scheduler matches a reference sequential executor across
generated programs, not just hand-picked ones. See
[docs/design/](docs/design/) for the ISA spec, constraints, and architecture
decision records (ADR-001 on the scheduling design, ADR-002 on the
property-testing approach). Next: Phase 2 (`impossible-vault`).

## The constraint

The machine does not use a conventional instruction-pointer-driven execution model. Instructions execute when their operand dependencies become ready, not in program-counter order.

## What the constraint forces

Fixed-size instruction encoding, bounded physical registers, explicit operand dependency tracking, deterministic scheduling, and deterministic replay.

## Research question

> How much of conventional instruction sequencing is actually necessary for useful general-purpose computation?

## Sibling repositories

- [impossible-language](https://github.com/n-3-0-l-d-3-v/impossible-language) — THE LANGUAGE (QUEUED)
- [impossible-kernel](https://github.com/n-3-0-l-d-3-v/impossible-kernel) — THE KERNEL (QUEUED)
- [impossible-vault](https://github.com/n-3-0-l-d-3-v/impossible-vault) — THE VAULT (QUEUED)
- [impossible-database](https://github.com/n-3-0-l-d-3-v/impossible-database) — THE DATABASE (QUEUED)
- [impossible-wire](https://github.com/n-3-0-l-d-3-v/impossible-wire) — THE WIRE (QUEUED)
- [impossible-colony](https://github.com/n-3-0-l-d-3-v/impossible-colony) — THE COLONY (QUEUED)
- [impossible-history](https://github.com/n-3-0-l-d-3-v/impossible-history) — THE HISTORY (QUEUED)
- [impossible-artifact](https://github.com/n-3-0-l-d-3-v/impossible-artifact) — THE ARTIFACT (STRETCH)

## Development

This is a real, tested, benchmarked systems component — not a demo. See
[docs/DEFINITION_OF_DONE.md](docs/DEFINITION_OF_DONE.md) for the acceptance
bar every piece of this repo must clear before it is considered complete.

```bash
cargo build
cargo test
cargo bench
```
