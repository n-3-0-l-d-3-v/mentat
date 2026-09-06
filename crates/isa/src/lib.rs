//! The Machine's instruction set architecture: fixed-size (8-byte) instruction
//! encoding, opcode table, and the block-structured program representation
//! that the VM schedules by data-dependency readiness rather than a
//! program counter. See `docs/design/ISA.md` in this repo for the full spec.

pub mod decode;
pub mod disasm;
pub mod encode;
pub mod opcode;
pub mod program;

pub use opcode::Opcode;
pub use program::{Block, Program};

/// Number of general-purpose architectural registers. Bounded, as required
/// by the "no conventional instruction pointer" constraint's corollary that
/// resources must be explicit and finite rather than assumed unbounded.
pub const NUM_REGISTERS: usize = 32;

/// A single fixed-size instruction: 1 byte opcode, 3 byte-sized operand
/// slots (dst/src1/src2, meaning depends on the opcode), and a 4-byte
/// immediate. Always exactly 8 bytes encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Instruction {
    pub opcode: Opcode,
    pub dst: u8,
    pub src1: u8,
    pub src2: u8,
    pub imm: i32,
}

impl Instruction {
    pub const ENCODED_LEN: usize = 8;

    pub fn new(opcode: Opcode, dst: u8, src1: u8, src2: u8, imm: i32) -> Self {
        Self {
            opcode,
            dst,
            src1,
            src2,
            imm,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("buffer too short: need {need} bytes, have {have}")]
    TooShort { need: usize, have: usize },
    #[error("unknown opcode byte 0x{0:02x}")]
    UnknownOpcode(u8),
}
