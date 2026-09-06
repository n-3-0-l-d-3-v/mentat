use crate::Instruction;

/// Encodes an instruction into its fixed 8-byte on-disk/in-memory form:
/// `[opcode][dst][src1][src2][imm:i32 little-endian]`.
pub fn encode(ins: &Instruction) -> [u8; Instruction::ENCODED_LEN] {
    let mut buf = [0u8; Instruction::ENCODED_LEN];
    buf[0] = ins.opcode.to_byte();
    buf[1] = ins.dst;
    buf[2] = ins.src1;
    buf[3] = ins.src2;
    buf[4..8].copy_from_slice(&ins.imm.to_le_bytes());
    buf
}

pub fn encode_program(instructions: &[Instruction]) -> Vec<u8> {
    let mut out = Vec::with_capacity(instructions.len() * Instruction::ENCODED_LEN);
    for ins in instructions {
        out.extend_from_slice(&encode(ins));
    }
    out
}
