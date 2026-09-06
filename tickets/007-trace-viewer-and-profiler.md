---
status: open
phase: 1
---

# 007 — Standalone trace viewer + instruction profiler

`docs/design/CONSTRAINTS.md` calls for a trace viewer, memory inspector,
and instruction profiler as first-class tools, not just JSON files a human
reads by hand. Currently only `imc run --trace <file>` + `imc replay` exist.

## Scope
- `imc trace <trace.json>`: human-readable rendering of a trace (per-block
  step counts, hot instructions, timeline), reusing `vm::Trace`.
- `imc profile <program>`: runs the program and reports per-opcode and
  per-block execution counts and wall-clock share — a real instruction
  profiler, not just a step counter.
- Extend the debugger's `mem` command to inspect a range, not one word.

Not started. Do not mark done until it clears `docs/DEFINITION_OF_DONE.md`.
