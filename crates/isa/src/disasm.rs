use crate::{Instruction, Opcode, Program};

/// Renders a program back into the assembler's textual syntax. Produces
/// output that reassembles to a structurally identical program (labels are
/// synthesized as `b<index>` since block index, not name, is what the
/// binary encoding actually preserves).
pub fn disassemble(program: &Program) -> String {
    let mut out = String::new();
    for (i, block) in program.blocks.iter().enumerate() {
        out.push_str(&format!("block b{i}:\n"));
        for ins in &block.instructions {
            out.push_str("    ");
            out.push_str(&disassemble_instruction(ins));
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

pub fn disassemble_instruction(ins: &Instruction) -> String {
    use Opcode::*;
    let m = ins.opcode.mnemonic();
    match ins.opcode {
        Nop | Halt | Ret => m.to_string(),
        LoadI => format!("{m} r{}, {}", ins.dst, ins.imm),
        Mov | Not => format!("{m} r{}, r{}", ins.dst, ins.src1),
        Add | Sub | Mul | Div | Mod | And | Or | Xor | Shl | Shr | CmpEq | CmpLt | CmpLe
        | CmpGt | CmpGe | CmpNe => format!("{m} r{}, r{}, r{}", ins.dst, ins.src1, ins.src2),
        Load => format!("{m} r{}, r{}, {}", ins.dst, ins.src1, ins.imm),
        Store => format!("{m} r{}, r{}, {}", ins.dst, ins.src1, ins.imm),
        Jmp | Call => format!("{m} b{}", ins.imm),
        Jz | Jnz => format!("{m} r{}, b{}", ins.src1, ins.imm),
        Syscall => format!("{m} {}", ins.imm),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, Instruction};

    #[test]
    fn disassembles_and_reassembles_shape() {
        let mut b = Block::new("entry");
        b.push(Instruction::new(Opcode::LoadI, 0, 0, 0, 42));
        b.push(Instruction::new(Opcode::Halt, 0, 0, 0, 0));
        let program = Program {
            blocks: vec![b],
            entry: 0,
        };
        let text = disassemble(&program);
        assert!(text.contains("loadi r0, 42"));
        assert!(text.contains("halt"));
    }
}
