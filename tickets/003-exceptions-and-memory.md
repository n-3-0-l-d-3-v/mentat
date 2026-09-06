---
status: done
phase: 1
---

# 003 — Bounds-checked memory + trap handling

## Acceptance criteria
- [x] Linear memory with checked 8-byte load/store; out-of-bounds access
      raises `Trap::OutOfBoundsMemory`, never a host panic or UB.
- [x] Division/modulo by zero raise `Trap::DivideByZero` with the faulting
      block/instruction.
- [x] Call stack is bounded (4096 frames); overflow/underflow are traps,
      not crashes.
- [x] A runaway program (no halt reachable) is caught by a step budget
      (`Trap::StepBudgetExceeded`) rather than hanging the host forever.
