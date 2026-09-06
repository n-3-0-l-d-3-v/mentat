use crate::DecodeError;

/// Every opcode the machine understands. Values are stable and part of the
/// on-disk encoding (`docs/design/ISA.md`), so existing bytecode must keep
/// decoding the same way across versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x00,
    Halt = 0x01,
    LoadI = 0x02,
    Mov = 0x03,
    Add = 0x04,
    Sub = 0x05,
    Mul = 0x06,
    Div = 0x07,
    Mod = 0x08,
    And = 0x09,
    Or = 0x0A,
    Xor = 0x0B,
    Not = 0x0C,
    Shl = 0x0D,
    Shr = 0x0E,
    CmpEq = 0x0F,
    CmpLt = 0x10,
    CmpLe = 0x11,
    CmpGt = 0x12,
    CmpGe = 0x13,
    CmpNe = 0x14,
    Load = 0x15,
    Store = 0x16,
    Jmp = 0x17,
    Jz = 0x18,
    Jnz = 0x19,
    Call = 0x1A,
    Ret = 0x1B,
    Syscall = 0x1C,
}

impl Opcode {
    pub fn from_byte(b: u8) -> Result<Self, DecodeError> {
        use Opcode::*;
        Ok(match b {
            0x00 => Nop,
            0x01 => Halt,
            0x02 => LoadI,
            0x03 => Mov,
            0x04 => Add,
            0x05 => Sub,
            0x06 => Mul,
            0x07 => Div,
            0x08 => Mod,
            0x09 => And,
            0x0A => Or,
            0x0B => Xor,
            0x0C => Not,
            0x0D => Shl,
            0x0E => Shr,
            0x0F => CmpEq,
            0x10 => CmpLt,
            0x11 => CmpLe,
            0x12 => CmpGt,
            0x13 => CmpGe,
            0x14 => CmpNe,
            0x15 => Load,
            0x16 => Store,
            0x17 => Jmp,
            0x18 => Jz,
            0x19 => Jnz,
            0x1A => Call,
            0x1B => Ret,
            0x1C => Syscall,
            other => return Err(DecodeError::UnknownOpcode(other)),
        })
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Mnemonic used by the assembler and disassembler.
    pub fn mnemonic(self) -> &'static str {
        use Opcode::*;
        match self {
            Nop => "nop",
            Halt => "halt",
            LoadI => "loadi",
            Mov => "mov",
            Add => "add",
            Sub => "sub",
            Mul => "mul",
            Div => "div",
            Mod => "mod",
            And => "and",
            Or => "or",
            Xor => "xor",
            Not => "not",
            Shl => "shl",
            Shr => "shr",
            CmpEq => "cmpeq",
            CmpLt => "cmplt",
            CmpLe => "cmple",
            CmpGt => "cmpgt",
            CmpGe => "cmpge",
            CmpNe => "cmpne",
            Load => "load",
            Store => "store",
            Jmp => "jmp",
            Jz => "jz",
            Jnz => "jnz",
            Call => "call",
            Ret => "ret",
            Syscall => "syscall",
        }
    }

    pub fn from_mnemonic(s: &str) -> Option<Self> {
        use Opcode::*;
        Some(match s {
            "nop" => Nop,
            "halt" => Halt,
            "loadi" => LoadI,
            "mov" => Mov,
            "add" => Add,
            "sub" => Sub,
            "mul" => Mul,
            "div" => Div,
            "mod" => Mod,
            "and" => And,
            "or" => Or,
            "xor" => Xor,
            "not" => Not,
            "shl" => Shl,
            "shr" => Shr,
            "cmpeq" => CmpEq,
            "cmplt" => CmpLt,
            "cmple" => CmpLe,
            "cmpgt" => CmpGt,
            "cmpge" => CmpGe,
            "cmpne" => CmpNe,
            "load" => Load,
            "store" => Store,
            "jmp" => Jmp,
            "jz" => Jz,
            "jnz" => Jnz,
            "call" => Call,
            "ret" => Ret,
            "syscall" => Syscall,
            _ => return None,
        })
    }

    /// True if this opcode may only appear as the final instruction of a
    /// block (it transfers control between blocks rather than producing a
    /// value consumed within the block).
    pub fn is_terminator(self) -> bool {
        matches!(
            self,
            Opcode::Halt | Opcode::Jmp | Opcode::Jz | Opcode::Jnz | Opcode::Call | Opcode::Ret
        )
    }

    /// Registers this instruction reads, given its operand fields.
    pub fn reads(self, ins: &crate::Instruction) -> Vec<u8> {
        use Opcode::*;
        match self {
            Nop | Halt | LoadI | Jmp | Ret | Syscall => vec![],
            Mov | Not => vec![ins.src1],
            Add | Sub | Mul | Div | Mod | And | Or | Xor | Shl | Shr | CmpEq | CmpLt | CmpLe
            | CmpGt | CmpGe | CmpNe => vec![ins.src1, ins.src2],
            Load => vec![ins.src1],
            Store => vec![ins.dst, ins.src1],
            Jz | Jnz => vec![ins.src1],
            Call => vec![],
        }
    }

    /// Register this instruction writes, if any.
    pub fn writes(self, ins: &crate::Instruction) -> Option<u8> {
        use Opcode::*;
        match self {
            LoadI | Mov | Add | Sub | Mul | Div | Mod | And | Or | Xor | Not | Shl | Shr
            | CmpEq | CmpLt | CmpLe | CmpGt | CmpGe | CmpNe | Load => Some(ins.dst),
            _ => None,
        }
    }
}
