//! Property-based tests (ticket 008): the dependency-driven scheduler must
//! behave identically to a conventional sequential (program-order)
//! reference implementation for any block whose only ordering requirement
//! is the data dependencies `vm::depgraph` encodes. This is differential
//! testing per the project philosophy (section 31 of the design doc): the
//! sequential executor is the "conventional system" control group, the
//! scheduler is the experiment, and they must agree.

use isa::{Block, Instruction, Opcode, Program};
use proptest::prelude::*;
use vm::{depgraph, Vm};

const NUM_TEST_REGS: u8 = 8;

/// A pure-arithmetic/logic instruction: no control flow, no memory, no
/// syscalls, no division (so no traps are possible), operating on a small
/// register window so writers/readers actually collide and exercise real
/// dependency chains instead of always being independent.
fn arb_instruction() -> impl Strategy<Value = Instruction> {
    let reg = 0..NUM_TEST_REGS;
    prop_oneof![
        (reg.clone(), any::<i32>()).prop_map(|(d, imm)| Instruction::new(
            Opcode::LoadI,
            d,
            0,
            0,
            imm
        )),
        (reg.clone(), reg.clone()).prop_map(|(d, s)| Instruction::new(Opcode::Mov, d, s, 0, 0)),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::Add,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::Sub,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::Mul,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::And,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::Or,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::Xor,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone()).prop_map(|(d, s)| Instruction::new(Opcode::Not, d, s, 0, 0)),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::CmpEq,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg.clone()).prop_map(|(d, a, b)| Instruction::new(
            Opcode::CmpLt,
            d,
            a,
            b,
            0
        )),
        (reg.clone(), reg.clone(), reg).prop_map(|(d, a, b)| Instruction::new(
            Opcode::CmpGt,
            d,
            a,
            b,
            0
        )),
    ]
}

fn arb_block(max_len: usize) -> impl Strategy<Value = Block> {
    prop::collection::vec(arb_instruction(), 1..max_len).prop_map(|mut instrs| {
        instrs.push(Instruction::new(Opcode::Halt, 0, 0, 0, 0));
        Block {
            label: "prop".into(),
            instructions: instrs,
        }
    })
}

proptest! {
    /// Every dependency edge points to a strictly earlier instruction
    /// index, for any block the generator can produce — so the graph is
    /// acyclic by construction, not just on the hand-picked unit-test
    /// examples.
    #[test]
    fn dependency_graph_is_always_acyclic(block in arb_block(40)) {
        let graph = depgraph::build(&block);
        for (i, node) in graph.iter().enumerate() {
            for &d in &node.deps {
                prop_assert!(d < i, "edge {d} -> {i} does not point strictly backward");
            }
        }
    }

    /// The scheduler executes every instruction in the block exactly once,
    /// regardless of how tangled the dependency graph is.
    #[test]
    fn scheduler_executes_every_instruction_exactly_once(block in arb_block(40)) {
        let n = block.instructions.len();
        let program = Program { blocks: vec![block], entry: 0 };
        let mut vm = Vm::new(program);
        vm.run().unwrap();
        prop_assert_eq!(vm.trace.steps.len(), n);
    }

    /// The core claim of ADR-001: dependency-driven scheduling and plain
    /// sequential (program-order) execution must reach identical final
    /// register state, because the dependency graph is built specifically
    /// to preserve program-order semantics under reordering.
    #[test]
    fn scheduled_execution_matches_sequential_reference(block in arb_block(30)) {
        let program = Program { blocks: vec![block], entry: 0 };

        let mut scheduled = Vm::new(program.clone());
        let scheduled_exit = scheduled.run().unwrap();

        let mut sequential = Vm::new(program);
        let sequential_exit = sequential.run_sequential().unwrap();

        prop_assert_eq!(scheduled_exit, sequential_exit);
        prop_assert_eq!(scheduled.regs.snapshot(), sequential.regs.snapshot());
    }
}
