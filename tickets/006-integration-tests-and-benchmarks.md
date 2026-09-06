---
status: done
phase: 1
---

# 006 — Real-program integration tests + scheduler benchmarks

## Acceptance criteria
- [x] A loop + function call program computing a real result (sum 1..5),
      not a "hello world."
- [x] A recursive program exercising repeated call/ret through the bounded
      call stack (recursive countdown).
- [x] Memory store/load round-tripping through a function call.
- [x] Deterministic-replay test: two independent runs of the same program
      produce byte-identical traces.
- [x] Trap tests for division-by-zero and out-of-bounds memory (trap, not panic).
- [x] Criterion benchmark of scheduler overhead vs. block size (10 to
      10,000 instructions) and of a realistic recursive workload.
