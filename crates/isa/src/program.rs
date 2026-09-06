use crate::Instruction;
use serde::{Deserialize, Serialize};

/// A basic block: a sequence of instructions with no internal PC-driven
/// order guarantee — the VM schedules them by data-dependency readiness —
/// followed by exactly one terminator instruction that transfers control to
/// another block (or halts). Blocks, not individual instructions, are the
/// unit of control transfer in this machine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Block {
    pub label: String,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, ins: Instruction) -> &mut Self {
        self.instructions.push(ins);
        self
    }

    pub fn terminator(&self) -> Option<&Instruction> {
        self.instructions.last()
    }
}

/// A whole program: a list of blocks plus the entry block index. Block
/// indices are the machine's only notion of "address" — there is no flat
/// instruction memory with a program counter into it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Program {
    pub blocks: Vec<Block>,
    pub entry: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("block {0} ('{1}') is empty")]
    EmptyBlock(usize, String),
    #[error("block {0} ('{1}') does not end with a terminator instruction")]
    MissingTerminator(usize, String),
    #[error("block {0} ('{1}') has a terminator before its last instruction")]
    TerminatorNotLast(usize, String),
    #[error("block {0} ('{1}') references out-of-range register {2} (max {3})")]
    RegisterOutOfRange(usize, String, u8, usize),
    #[error("block {0} ('{1}') jumps to out-of-range block {2} (have {3} blocks)")]
    BlockOutOfRange(usize, String, i32, usize),
    #[error("entry block {0} is out of range (have {1} blocks)")]
    EntryOutOfRange(usize, usize),
}

impl Program {
    /// Structural validation performed once at load time, analogous to a
    /// linker/loader check on a conventional machine: every block is
    /// well-formed before the VM ever schedules an instruction out of it.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.entry >= self.blocks.len() {
            return Err(ValidationError::EntryOutOfRange(
                self.entry,
                self.blocks.len(),
            ));
        }
        for (bi, block) in self.blocks.iter().enumerate() {
            if block.instructions.is_empty() {
                return Err(ValidationError::EmptyBlock(bi, block.label.clone()));
            }
            let last_idx = block.instructions.len() - 1;
            for (ii, ins) in block.instructions.iter().enumerate() {
                if ins.opcode.is_terminator() && ii != last_idx {
                    return Err(ValidationError::TerminatorNotLast(bi, block.label.clone()));
                }
                for reg in ins.opcode.reads(ins) {
                    if reg as usize >= crate::NUM_REGISTERS {
                        return Err(ValidationError::RegisterOutOfRange(
                            bi,
                            block.label.clone(),
                            reg,
                            crate::NUM_REGISTERS,
                        ));
                    }
                }
                if let Some(reg) = ins.opcode.writes(ins) {
                    if reg as usize >= crate::NUM_REGISTERS {
                        return Err(ValidationError::RegisterOutOfRange(
                            bi,
                            block.label.clone(),
                            reg,
                            crate::NUM_REGISTERS,
                        ));
                    }
                }
                use crate::Opcode::*;
                if matches!(ins.opcode, Jmp | Jz | Jnz | Call)
                    && (ins.imm as usize) >= self.blocks.len()
                {
                    return Err(ValidationError::BlockOutOfRange(
                        bi,
                        block.label.clone(),
                        ins.imm,
                        self.blocks.len(),
                    ));
                }
            }
            if !block.instructions[last_idx].opcode.is_terminator() {
                return Err(ValidationError::MissingTerminator(bi, block.label.clone()));
            }
        }
        Ok(())
    }
}
