# MENTAT — THE MACHINE

> A dependency-driven virtual machine with no instruction pointer.

## Why "MENTAT"

After the Butlerian Jihad outlawed "thinking machines" ("thou shalt not make a machine in the likeness of a human mind"), humans trained themselves into Mentats — living computers that process information without any conventional computing architecture underneath them. Mentats exist *because* the normal machine was forbidden and had to compute anyway, using an entirely different internal model. That is precisely what a dependency-scheduled VM with no program counter is doing.

Part of **[ARRAKIS](https://github.com/n-3-0-l-d-3-v/arrakis)** — a constrained computing
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
property-testing approach). Next: Phase 2 (`sietch`).

## The constraint

The machine does not use a conventional instruction-pointer-driven execution model. Instructions execute when their operand dependencies become ready, not in program-counter order.

## What the constraint forces

Fixed-size instruction encoding, bounded physical registers, explicit operand dependency tracking, deterministic scheduling, and deterministic replay.

## Research question

> How much of conventional instruction sequencing is actually necessary for useful general-purpose computation?

## Sibling repositories

- [chakobsa](https://github.com/n-3-0-l-d-3-v/chakobsa) — THE LANGUAGE (QUEUED)
- [muaddib](https://github.com/n-3-0-l-d-3-v/muaddib) — THE KERNEL (QUEUED)
- [sietch](https://github.com/n-3-0-l-d-3-v/sietch) — THE VAULT (ACTIVE)
- [choam](https://github.com/n-3-0-l-d-3-v/choam) — THE DATABASE (QUEUED)
- [distrans](https://github.com/n-3-0-l-d-3-v/distrans) — THE WIRE (QUEUED)
- [landsraad](https://github.com/n-3-0-l-d-3-v/landsraad) — THE COLONY (QUEUED)
- [ghola](https://github.com/n-3-0-l-d-3-v/ghola) — THE HISTORY (QUEUED)
- [shai-hulud](https://github.com/n-3-0-l-d-3-v/shai-hulud) — THE ARTIFACT (STRETCH)

## Development

This is a real, tested, benchmarked systems component — not a demo. See
[docs/DEFINITION_OF_DONE.md](docs/DEFINITION_OF_DONE.md) for the acceptance
bar every piece of this repo must clear before it is considered complete.

```bash
cargo build
cargo test
cargo bench
```
