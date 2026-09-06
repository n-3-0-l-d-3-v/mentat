use isa::NUM_REGISTERS;
use serde::{Deserialize, Serialize};

/// One executed instruction, in the order the scheduler actually fired it
/// (which, within a block, is dependency order — not program order).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: u64,
    pub block: usize,
    pub block_label: String,
    pub instr_index: usize,
    pub mnemonic: String,
    pub result: Option<i64>,
}

/// A complete, replayable record of one execution: every step fired plus
/// the final register/memory/exit state. Because the scheduler is
/// deterministic (ties broken by lowest ready instruction index), replaying
/// the same program from the same initial state must reproduce this trace
/// exactly — that equality check is what `imc replay` verifies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub steps: Vec<TraceStep>,
    pub final_registers: [i64; NUM_REGISTERS],
    pub final_memory_hash: u64,
    pub exit: String,
    pub output: Vec<String>,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            final_registers: [0; NUM_REGISTERS],
            final_memory_hash: 0,
            exit: String::new(),
            output: Vec::new(),
        }
    }

    /// Compares two traces step-by-step and returns the index of the first
    /// divergence, if any. Used by `imc replay` to prove (or disprove)
    /// determinism rather than merely asserting it.
    pub fn first_divergence(&self, other: &Trace) -> Option<usize> {
        for (i, (a, b)) in self.steps.iter().zip(other.steps.iter()).enumerate() {
            if a != b {
                return Some(i);
            }
        }
        if self.steps.len() != other.steps.len() {
            return Some(self.steps.len().min(other.steps.len()));
        }
        if self.final_registers != other.final_registers
            || self.final_memory_hash != other.final_memory_hash
            || self.exit != other.exit
        {
            return Some(self.steps.len());
        }
        None
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple FNV-1a hash over memory contents, used to compare final memory
/// state between two runs without shipping the entire memory image in every
/// trace file.
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
