/// A fault raised during execution. Every trap identifies the block and
/// instruction that raised it so the debugger can point straight at the
/// failure instead of a bare error message.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum Trap {
    #[error("division by zero at block {block} instruction {instr}")]
    DivideByZero { block: usize, instr: usize },
    #[error("memory access out of bounds: address {addr} + {len} bytes exceeds memory size {size} (block {block} instruction {instr})")]
    OutOfBoundsMemory {
        addr: u64,
        len: u8,
        size: usize,
        block: usize,
        instr: usize,
    },
    #[error("call stack overflow (depth {depth}) at block {block} instruction {instr}")]
    CallStackOverflow {
        depth: usize,
        block: usize,
        instr: usize,
    },
    #[error("return with empty call stack at block {block} instruction {instr}")]
    CallStackUnderflow { block: usize, instr: usize },
    #[error("unknown syscall number {number} at block {block} instruction {instr}")]
    UnknownSyscall {
        number: i32,
        block: usize,
        instr: usize,
    },
    #[error("step budget of {budget} exceeded (possible non-terminating program)")]
    StepBudgetExceeded { budget: u64 },
}
