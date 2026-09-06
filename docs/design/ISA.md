# The Machine — ISA Specification

## Overview

The Machine has no program counter. A conventional CPU fetches the
instruction at `PC`, executes it, and sets `PC` to the next address (or a
branch target). This machine instead organizes a program into **blocks**,
and within a block, instructions fire when their **operand dependencies are
ready** — not in the order they appear in the file. Control only ever
transfers *between* blocks, and that transfer is explicit dataflow, not an
incrementing counter into a flat instruction array.

## Instruction encoding

Every instruction is exactly **8 bytes**, little-endian:

```
byte 0     : opcode
byte 1     : dst   (register index, meaning depends on opcode)
byte 2     : src1  (register index)
byte 3     : src2  (register index)
bytes 4..8 : imm   (i32, little-endian)
```

Fixed-size encoding means there is no variable-length instruction decoding
loop to design around — a design requirement carried over from the fact that
there is no PC advancing through a byte stream in the first place.

## Registers

32 general-purpose 64-bit signed integer registers, `r0`..`r31`. There is no
hardwired zero register. By convention (not enforced by the machine),
`r0` carries the first function argument / return value.

## Memory

A flat, byte-addressable region (default 1 MiB, configurable). `load`/
`store` operate on 8-byte little-endian words. Every access is bounds
checked; a violation raises `Trap::OutOfBoundsMemory` rather than
undefined behavior.

## Blocks and control transfer

A block is a sequence of instructions ending in exactly one **terminator**:
`halt`, `jmp`, `jz`, `jnz`, `call`, or `ret`. `Program::validate` rejects a
block that is empty, lacks a terminator, or has a terminator anywhere but
the last position.

- `jmp <block>` — unconditional transfer.
- `jz <reg>, <block>` / `jnz <reg>, <block>` — transfer if `reg` is
  zero/nonzero; otherwise **fall through to the next block by index**. This
  fallthrough convention is why loop bodies are laid out as
  `loop_check` (index *i*) immediately followed by `loop_body` (index
  *i+1*) in the examples.
- `call <block>` — pushes `current_block_index + 1` onto the call stack
  (bounded at 4096 frames) and transfers to `<block>`. The callee returns to
  the block laid out immediately after the call site — the same "fallthrough
  by physical layout" convention used by conditional jumps.
- `ret` — pops the call stack and transfers there; popping an empty stack
  raises `Trap::CallStackUnderflow`.
- `halt` — stops execution.

## Why no instruction pointer, concretely

Within a block, the VM builds a dependency DAG (`vm::depgraph`) before
executing anything:

- Walking instructions in their *encoded* order (used only to resolve
  "which prior write defines this read", i.e. SSA-style last-writer
  resolution — never to decide execution order), each read of register `r`
  depends on the most recent in-block writer of `r`, if any.
- Each write to `r` also depends on the previous writer of `r`
  (write-after-write ordering), so "last write wins" still matches program
  order even when physical execution order differs.
- The terminator additionally depends on **every other instruction** in the
  block — a block-level dataflow barrier ensuring the whole block's
  computation has drained before control leaves it.

The scheduler (`Vm::step_block`) then runs a **deterministic list
scheduling** algorithm: at each step, of all instructions whose
dependencies are satisfied, it fires the one with the lowest instruction
index. This is the machine's only ordering rule beyond data dependencies,
and it is what makes execution — and therefore tracing and replay — fully
reproducible without any notion of "the next instruction."

## Traps

Every fault (`crates/vm/src/trap.rs`) carries the block and instruction
index that raised it: `DivideByZero`, `OutOfBoundsMemory`,
`CallStackOverflow`, `CallStackUnderflow`, `UnknownSyscall`, and
`StepBudgetExceeded` (a runaway-program guard, not a "real" architectural
trap).

## Syscalls

`syscall 0` prints `r0` as a signed decimal integer. `syscall 1` exits with
`r0` as the exit code. This is intentionally minimal — Phase 1's job is the
execution model, not an OS ABI (that's `impossible-kernel`, Phase 4).

## What Phase 1 does *not* claim

- This is not superscalar/out-of-order hardware; the scheduler executes one
  instruction at a time. The dependency graph proves which orders are
  *valid*; determinism is achieved by always picking the same one, not by
  claiming free parallelism.
- Block-to-block transfer is still, honestly, a coarse-grained analog of a
  program counter (there is an active block index). What is eliminated is
  fine-grained sequential fetch *within* a block — the claim is scoped
  precisely to that, per `docs/design/CONSTRAINTS.md`.
