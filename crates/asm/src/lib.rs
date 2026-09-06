//! A small two-pass assembler for the Machine's textual assembly format.
//!
//! Syntax:
//! ```text
//! ; comments start with a semicolon
//! block entry:
//!     loadi r0, 5
//!     loadi r1, 1
//!     jmp loop
//!
//! block loop:
//!     cmple  r2, r0, r1
//!     jz     r2, done
//!     add    r1, r1, r1
//!     jmp    loop
//!
//! block done:
//!     syscall 0
//!     halt
//! ```
//!
//! Each `block <label>:` starts a new basic block; the block ends at the
//! next `block` header or end of file, and must end with a terminator
//! instruction (`halt`, `jmp`, `jz`, `jnz`, `call`, `ret`) — this is checked
//! by `Program::validate`, not by the assembler itself, so a malformed
//! program still assembles and fails at load time with a precise error.

use isa::{Block, Instruction, Opcode, Program};

#[derive(Debug, thiserror::Error)]
pub enum AsmError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },
    #[error("line {line}: unknown label '{label}'")]
    UnknownLabel { line: usize, label: String },
    #[error("line {line}: unknown mnemonic '{mnemonic}'")]
    UnknownMnemonic { line: usize, mnemonic: String },
    #[error("no blocks defined")]
    NoBlocks,
}

struct RawInstr {
    line: usize,
    mnemonic: String,
    args: Vec<String>,
}

struct RawBlock {
    label: String,
    instrs: Vec<RawInstr>,
}

pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let raw_blocks = parse_raw(source)?;
    if raw_blocks.is_empty() {
        return Err(AsmError::NoBlocks);
    }

    let label_index: std::collections::HashMap<&str, usize> = raw_blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.label.as_str(), i))
        .collect();

    let mut blocks = Vec::with_capacity(raw_blocks.len());
    for raw in &raw_blocks {
        let mut block = Block::new(raw.label.clone());
        for ri in &raw.instrs {
            block.push(to_instruction(ri, &label_index)?);
        }
        blocks.push(block);
    }

    Ok(Program { blocks, entry: 0 })
}

fn parse_raw(source: &str) -> Result<Vec<RawBlock>, AsmError> {
    let mut blocks: Vec<RawBlock> = Vec::new();

    for (lineno, raw_line) in source.lines().enumerate() {
        let line_no = lineno + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("block ") {
            let label = rest.trim().trim_end_matches(':').trim();
            if label.is_empty() {
                return Err(AsmError::Syntax {
                    line: line_no,
                    message: "block label cannot be empty".into(),
                });
            }
            blocks.push(RawBlock {
                label: label.to_string(),
                instrs: Vec::new(),
            });
            continue;
        }

        let block = blocks.last_mut().ok_or_else(|| AsmError::Syntax {
            line: line_no,
            message: "instruction appears before any 'block <label>:' header".into(),
        })?;

        let (mnemonic, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let args: Vec<String> = if rest.trim().is_empty() {
            Vec::new()
        } else {
            rest.split(',').map(|s| s.trim().to_string()).collect()
        };
        block.instrs.push(RawInstr {
            line: line_no,
            mnemonic: mnemonic.to_string(),
            args,
        });
    }

    Ok(blocks)
}

fn strip_comment(line: &str) -> &str {
    match line.find(';') {
        Some(i) => &line[..i],
        None => line,
    }
}

fn parse_reg(s: &str, line: usize) -> Result<u8, AsmError> {
    let s = s.trim();
    let digits = s.strip_prefix('r').ok_or_else(|| AsmError::Syntax {
        line,
        message: format!("expected register like 'r0', got '{s}'"),
    })?;
    digits.parse::<u8>().map_err(|_| AsmError::Syntax {
        line,
        message: format!("invalid register number in '{s}'"),
    })
}

fn parse_imm(s: &str, line: usize) -> Result<i32, AsmError> {
    let s = s.trim();
    let parsed = if let Some(hex) = s.strip_prefix("0x") {
        i32::from_str_radix(hex, 16)
    } else {
        s.parse::<i32>()
    };
    parsed.map_err(|_| AsmError::Syntax {
        line,
        message: format!("invalid immediate '{s}'"),
    })
}

fn resolve_label(
    s: &str,
    line: usize,
    labels: &std::collections::HashMap<&str, usize>,
) -> Result<i32, AsmError> {
    labels
        .get(s.trim())
        .map(|&i| i as i32)
        .ok_or_else(|| AsmError::UnknownLabel {
            line,
            label: s.trim().to_string(),
        })
}

fn to_instruction(
    ri: &RawInstr,
    labels: &std::collections::HashMap<&str, usize>,
) -> Result<Instruction, AsmError> {
    let line = ri.line;
    let a = |i: usize| {
        ri.args
            .get(i)
            .map(String::as_str)
            .ok_or_else(|| AsmError::Syntax {
                line,
                message: format!("'{}' expects an operand at position {}", ri.mnemonic, i + 1),
            })
    };

    macro_rules! rrr {
        ($op:expr) => {{
            Ok(Instruction::new(
                $op,
                parse_reg(a(0)?, line)?,
                parse_reg(a(1)?, line)?,
                parse_reg(a(2)?, line)?,
                0,
            ))
        }};
    }
    macro_rules! rr {
        ($op:expr) => {{
            Ok(Instruction::new(
                $op,
                parse_reg(a(0)?, line)?,
                parse_reg(a(1)?, line)?,
                0,
                0,
            ))
        }};
    }

    match ri.mnemonic.as_str() {
        "nop" => Ok(Instruction::new(Opcode::Nop, 0, 0, 0, 0)),
        "halt" => Ok(Instruction::new(Opcode::Halt, 0, 0, 0, 0)),
        "loadi" => Ok(Instruction::new(
            Opcode::LoadI,
            parse_reg(a(0)?, line)?,
            0,
            0,
            parse_imm(a(1)?, line)?,
        )),
        "mov" => rr!(Opcode::Mov),
        "add" => rrr!(Opcode::Add),
        "sub" => rrr!(Opcode::Sub),
        "mul" => rrr!(Opcode::Mul),
        "div" => rrr!(Opcode::Div),
        "mod" => rrr!(Opcode::Mod),
        "and" => rrr!(Opcode::And),
        "or" => rrr!(Opcode::Or),
        "xor" => rrr!(Opcode::Xor),
        "not" => rr!(Opcode::Not),
        "shl" => rrr!(Opcode::Shl),
        "shr" => rrr!(Opcode::Shr),
        "cmpeq" => rrr!(Opcode::CmpEq),
        "cmplt" => rrr!(Opcode::CmpLt),
        "cmple" => rrr!(Opcode::CmpLe),
        "cmpgt" => rrr!(Opcode::CmpGt),
        "cmpge" => rrr!(Opcode::CmpGe),
        "cmpne" => rrr!(Opcode::CmpNe),
        "load" => Ok(Instruction::new(
            Opcode::Load,
            parse_reg(a(0)?, line)?,
            parse_reg(a(1)?, line)?,
            0,
            parse_imm(a(2)?, line)?,
        )),
        "store" => Ok(Instruction::new(
            Opcode::Store,
            parse_reg(a(0)?, line)?,
            parse_reg(a(1)?, line)?,
            0,
            parse_imm(a(2)?, line)?,
        )),
        "jmp" => Ok(Instruction::new(
            Opcode::Jmp,
            0,
            0,
            0,
            resolve_label(a(0)?, line, labels)?,
        )),
        "jz" => Ok(Instruction::new(
            Opcode::Jz,
            0,
            parse_reg(a(0)?, line)?,
            0,
            resolve_label(a(1)?, line, labels)?,
        )),
        "jnz" => Ok(Instruction::new(
            Opcode::Jnz,
            0,
            parse_reg(a(0)?, line)?,
            0,
            resolve_label(a(1)?, line, labels)?,
        )),
        "call" => Ok(Instruction::new(
            Opcode::Call,
            0,
            0,
            0,
            resolve_label(a(0)?, line, labels)?,
        )),
        "ret" => Ok(Instruction::new(Opcode::Ret, 0, 0, 0, 0)),
        "syscall" => Ok(Instruction::new(
            Opcode::Syscall,
            0,
            0,
            0,
            parse_imm(a(0)?, line)?,
        )),
        other => Err(AsmError::UnknownMnemonic {
            line,
            mnemonic: other.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_simple_countdown() {
        let src = r#"
            block loop:
                syscall 0
                loadi r1, 1
                sub r0, r0, r1
                cmpgt r2, r0, r1
                jnz r2, loop
            block done:
                halt
        "#;
        let program = assemble(src).unwrap();
        assert_eq!(program.blocks.len(), 2);
        program.validate().unwrap();
    }

    #[test]
    fn resolves_multi_block_labels() {
        let src = r#"
            block entry:
                loadi r0, 0
                jmp done
            block done:
                halt
        "#;
        let program = assemble(src).unwrap();
        assert_eq!(program.blocks.len(), 2);
        program.validate().unwrap();
        assert_eq!(program.blocks[0].instructions[1].imm, 1);
    }

    #[test]
    fn unknown_label_errors() {
        let src = "block entry:\n  jmp nowhere\n";
        let err = assemble(src).unwrap_err();
        matches!(err, AsmError::UnknownLabel { .. });
    }

    #[test]
    fn unknown_mnemonic_errors() {
        let src = "block entry:\n  frobnicate r0\n";
        let err = assemble(src).unwrap_err();
        matches!(err, AsmError::UnknownMnemonic { .. });
    }
}
