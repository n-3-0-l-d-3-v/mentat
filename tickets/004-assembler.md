---
status: done
phase: 1
---

# 004 — Text assembler

Two-pass assembler (`crates/asm`): label resolution, then instruction
lowering. See module docs in `crates/asm/src/lib.rs` for the syntax.

## Acceptance criteria
- [x] Multi-block programs with forward and backward label references.
- [x] Clear syntax errors with line numbers for unknown mnemonics, unknown
      labels, and malformed operands (not a generic parse failure).
- [x] Round-trips through `Program::validate`.
