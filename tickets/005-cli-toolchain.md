---
status: done
phase: 1
---

# 005 — `imc` CLI: assembler front-end, disassembler, runner, replay, debugger

## Acceptance criteria
- [x] `imc asm` / `imc disasm` round-trip through the shared `isa`/`asm` crates.
- [x] `imc run` executes a program, reports traps distinctly from normal exit,
      and can write a full execution trace to a file.
- [x] `imc replay` re-executes a program and diffs the resulting trace
      against a previously recorded one, reporting the first divergent
      step — proven both on a matching trace and a deliberately tampered one.
- [x] `imc debug` supports block-granular single-stepping (`step [n]`),
      register inspection (`regs`), and memory inspection (`mem <addr>`).

## Known limitations (tracked, not silently ignored)
- The debugger's `mem` inspector reports block/instruction 0 on trap
  regardless of where the stepping actually is — acceptable for Phase 1,
  revisit if it gets in the way of Phase 2/3 debugging.
- No standalone trace-viewer or instruction-profiler binary yet; `imc run
  --trace` plus reading the JSON is the current substitute. Follow-up
  ticket: 007.
