---
status: done
phase: 1
---

# 007 — Standalone trace viewer + instruction profiler

`docs/design/CONSTRAINTS.md` calls for a trace viewer, memory inspector,
and instruction profiler as first-class tools, not just JSON files a human
reads by hand.

## Acceptance criteria
- [x] `imc trace <trace.json>`: human-readable rendering of a trace —
      total steps, exit reason, output, per-block step counts, hot
      instructions ranked by mnemonic, and a head/tail timeline.
- [x] `imc profile <program>`: runs the program, measures wall-clock time,
      and reports per-block and per-opcode execution share (count and
      percentage) — a real instruction profiler grounded in the same
      `vm::Trace` data, not a guess.
- [x] Debugger's `mem` command extended to inspect a range (`mem <addr>
      [count]`), printing each word in decimal and hex.

## Known limitation (tracked, not silently ignored)
- `imc profile`'s per-step timing is wall-clock-for-the-whole-run divided
  by step count (an average), not per-instruction instrumentation — real
  per-opcode timing would need instrumenting `Vm::execute` itself, which
  risks skewing the numbers it's trying to measure. Revisit if Phase 2+
  needs finer-grained profiling.
