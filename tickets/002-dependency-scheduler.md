---
status: done
phase: 1
---

# 002 — Dependency-graph builder + deterministic scheduler

Build the intra-block dependency DAG from register data-dependencies (no
program counter), and a deterministic list-scheduling executor over it.
See `docs/design/ISA.md` and `docs/design/decisions/ADR-001-*.md`.

## Acceptance criteria
- [x] DAG is acyclic by construction (property-tested: every dependency
      edge points to a strictly earlier instruction index).
- [x] Reads resolve to the correct in-block last-writer (or block-entry
      register-file value if none).
- [x] Write-after-write ordering preserved (last write in program order
      wins, even though physical execution order can differ).
- [x] Terminator depends on the whole block (block-level dataflow barrier).
- [x] Scheduler is deterministic (fixed tie-break: lowest ready index).
