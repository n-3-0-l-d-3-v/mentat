use isa::{Block, NUM_REGISTERS};

/// One node in a block's dependency graph: the indices (within the block)
/// of every instruction that must have already executed before this one
/// becomes ready. Built once per block from register data-dependencies —
/// this is the thing that stands in for a program counter.
#[derive(Debug, Clone, Default)]
pub struct DepNode {
    pub deps: Vec<usize>,
}

/// Builds the intra-block dependency DAG.
///
/// Algorithm: walk the block's instructions in their encoded (program)
/// order — this order is used only to determine, for a source register
/// read, which prior write in the block last defined it (an SSA-style
/// "last writer wins" resolution). It is never used to decide execution
/// order; that is left entirely to the scheduler, which fires instructions
/// purely by dependency readiness.
///
/// - A read of register `r` depends on the most recent in-block writer of
///   `r`, if any (if none, the value comes from the register file at block
///   entry, which is always ready).
/// - A write to register `r` also depends on the most recent prior writer
///   of `r` (write-after-write ordering), so that "last write wins"
///   matches program order even though physical execution order can
///   differ.
/// - The block's terminator additionally depends on every other
///   instruction in the block, guaranteeing the whole block's dataflow has
///   drained before control transfers elsewhere.
pub fn build(block: &Block) -> Vec<DepNode> {
    let n = block.instructions.len();
    let mut nodes: Vec<DepNode> = vec![DepNode::default(); n];
    let mut last_writer: [Option<usize>; NUM_REGISTERS] = [None; NUM_REGISTERS];

    for (i, ins) in block.instructions.iter().enumerate() {
        let mut deps = Vec::new();
        for r in ins.opcode.reads(ins) {
            if let Some(w) = last_writer[r as usize] {
                deps.push(w);
            }
        }
        if let Some(w) = ins.opcode.writes(ins) {
            if let Some(prev) = last_writer[w as usize] {
                deps.push(prev);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        nodes[i].deps = deps;

        if let Some(w) = ins.opcode.writes(ins) {
            last_writer[w as usize] = Some(i);
        }
    }

    if n > 0 {
        let last = n - 1;
        let mut all: Vec<usize> = (0..last).collect();
        all.extend(nodes[last].deps.iter().copied());
        all.sort_unstable();
        all.dedup();
        nodes[last].deps = all;
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use isa::{Instruction, Opcode};

    fn block_of(instrs: Vec<Instruction>) -> Block {
        Block {
            label: "b".into(),
            instructions: instrs,
        }
    }

    #[test]
    fn independent_instructions_have_no_deps() {
        let b = block_of(vec![
            Instruction::new(Opcode::LoadI, 0, 0, 0, 1),
            Instruction::new(Opcode::LoadI, 1, 0, 0, 2),
            Instruction::new(Opcode::Halt, 0, 0, 0, 0),
        ]);
        let g = build(&b);
        assert!(g[0].deps.is_empty());
        assert!(g[1].deps.is_empty());
        // terminator depends on everything before it
        assert_eq!(g[2].deps, vec![0, 1]);
    }

    #[test]
    fn read_depends_on_last_writer() {
        // r0 = 1; r0 = 2; r1 = r0 + r0; halt
        let b = block_of(vec![
            Instruction::new(Opcode::LoadI, 0, 0, 0, 1),
            Instruction::new(Opcode::LoadI, 0, 0, 0, 2),
            Instruction::new(Opcode::Add, 1, 0, 0, 0),
            Instruction::new(Opcode::Halt, 0, 0, 0, 0),
        ]);
        let g = build(&b);
        assert!(g[0].deps.is_empty());
        assert_eq!(g[1].deps, vec![0]); // WAW on r0
        assert_eq!(g[2].deps, vec![1]); // reads r0, last writer is instr 1
        assert_eq!(g[3].deps, vec![0, 1, 2]); // terminator barrier
    }

    #[test]
    fn dag_is_acyclic_by_construction() {
        // Every dependency edge points strictly backward in program order,
        // so a topological execution always exists.
        let b = block_of(vec![
            Instruction::new(Opcode::LoadI, 0, 0, 0, 5),
            Instruction::new(Opcode::LoadI, 1, 0, 0, 7),
            Instruction::new(Opcode::Add, 2, 0, 1, 0),
            Instruction::new(Opcode::Mul, 3, 2, 2, 0),
            Instruction::new(Opcode::Halt, 0, 0, 0, 0),
        ]);
        let g = build(&b);
        for (i, node) in g.iter().enumerate() {
            for &d in &node.deps {
                assert!(d < i, "dependency {d} of instruction {i} must precede it");
            }
        }
    }
}
