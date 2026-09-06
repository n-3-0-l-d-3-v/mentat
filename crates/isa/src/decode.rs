use crate::{DecodeError, Instruction, Opcode};

pub fn decode(buf: &[u8]) -> Result<Instruction, DecodeError> {
    if buf.len() < Instruction::ENCODED_LEN {
        return Err(DecodeError::TooShort {
            need: Instruction::ENCODED_LEN,
            have: buf.len(),
        });
    }
    let opcode = Opcode::from_byte(buf[0])?;
    let dst = buf[1];
    let src1 = buf[2];
    let src2 = buf[3];
    let imm = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    Ok(Instruction {
        opcode,
        dst,
        src1,
        src2,
        imm,
    })
}

pub fn decode_program(buf: &[u8]) -> Result<Vec<Instruction>, DecodeError> {
    let mut out = Vec::with_capacity(buf.len() / Instruction::ENCODED_LEN);
    let mut i = 0;
    while i < buf.len() {
        out.push(decode(&buf[i..])?);
        i += Instruction::ENCODED_LEN;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::encode;

    #[test]
    fn roundtrip() {
        let ins = Instruction::new(Opcode::Add, 1, 2, 3, 0);
        let bytes = encode(&ins);
        assert_eq!(bytes.len(), 8);
        let back = decode(&bytes).unwrap();
        assert_eq!(ins, back);
    }

    #[test]
    fn negative_immediate_roundtrip() {
        let ins = Instruction::new(Opcode::LoadI, 5, 0, 0, -12345);
        let back = decode(&encode(&ins)).unwrap();
        assert_eq!(back.imm, -12345);
    }

    #[test]
    fn too_short() {
        let err = decode(&[0x04, 0x01]).unwrap_err();
        assert_eq!(err, DecodeError::TooShort { need: 8, have: 2 });
    }

    #[test]
    fn unknown_opcode() {
        let err = decode(&[0xFF, 0, 0, 0, 0, 0, 0, 0]).unwrap_err();
        assert_eq!(err, DecodeError::UnknownOpcode(0xFF));
    }
}
