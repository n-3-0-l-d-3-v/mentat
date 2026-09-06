---
status: done
phase: 1
---

# 001 — Fixed-size instruction encoding + opcode table

Define the 8-byte instruction encoding, the opcode table (29 opcodes
covering arithmetic, comparison, memory, and control transfer), and the
block/program structural representation with load-time validation.

## Acceptance criteria
- [x] Encode/decode round-trips for all opcodes, including negative immediates.
- [x] Malformed bytecode (short buffer, unknown opcode) reported as `DecodeError`, not a panic.
- [x] `Program::validate` rejects: empty blocks, missing terminators, misplaced terminators, out-of-range registers, out-of-range block jump targets.
