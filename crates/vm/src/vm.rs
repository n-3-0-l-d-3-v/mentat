use std::collections::BTreeSet;

use isa::{Instruction, Opcode, Program};

use crate::depgraph;
use crate::memory::{Memory, WORD_LEN};
use crate::regfile::RegisterFile;
use crate::trace::{hash_bytes, Trace, TraceStep};
use crate::trap::Trap;

const DEFAULT_MEMORY_SIZE: usize = 1 << 20; // 1 MiB
const DEFAULT_STEP_BUDGET: u64 = 10_000_000;
const MAX_CALL_DEPTH: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitReason {
    Halted,
    SyscallExit(i64),
}

pub struct Vm {
    pub program: Program,
    pub regs: RegisterFile,
    pub memory: Memory,
    pub call_stack: Vec<usize>,
    pub current_block: usize,
    pub step_budget: u64,
    steps_taken: u64,
    pub trace: Trace,
    /// Records every value ever printed via `syscall 0`, for testing.
    pub output: Vec<String>,
    pending_exit: Option<ExitReason>,
}

impl Vm {
    pub fn new(program: Program) -> Self {
        let entry = program.entry;
        Self {
            program,
            regs: RegisterFile::default(),
            memory: Memory::new(DEFAULT_MEMORY_SIZE),
            call_stack: Vec::new(),
            current_block: entry,
            step_budget: DEFAULT_STEP_BUDGET,
            steps_taken: 0,
            trace: Trace::new(),
            output: Vec::new(),
            pending_exit: None,
        }
    }

    pub fn with_memory_size(mut self, size: usize) -> Self {
        self.memory = Memory::new(size);
        self
    }

    pub fn with_step_budget(mut self, budget: u64) -> Self {
        self.step_budget = budget;
        self
    }

    /// Runs the program to completion (Halt or syscall-exit) or until a
    /// trap fires or the step budget is exhausted.
    pub fn run(&mut self) -> Result<ExitReason, Trap> {
        loop {
            if let Some(exit) = self.step_block()? {
                self.trace.final_registers = self.regs.snapshot();
                self.trace.final_memory_hash = hash_bytes(self.memory.as_slice());
                self.trace.exit = format!("{exit:?}");
                self.trace.output = self.output.clone();
                return Ok(exit);
            }
        }
    }

    /// Executes exactly one block via the dependency-driven scheduler, then
    /// returns `Some(exit)` if the program halted/exited, or `None` if
    /// control transferred to another block and execution should continue.
    /// Public so a debugger can single-step block-by-block instead of
    /// running the whole program in one call.
    pub fn step_block(&mut self) -> Result<Option<ExitReason>, Trap> {
        let block_idx = self.current_block;
        let block = self.program.blocks[block_idx].clone();
        let graph = depgraph::build(&block);
        let n = block.instructions.len();

        let mut indegree: Vec<usize> = graph.iter().map(|node| node.deps.len()).collect();
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (i, node) in graph.iter().enumerate() {
            for &d in &node.deps {
                successors[d].push(i);
            }
        }

        // Deterministic ready set: always fire the lowest-index ready
        // instruction. This is the scheduler's only tie-breaking rule and
        // is what makes execution reproducible without a program counter.
        let mut ready: BTreeSet<usize> = BTreeSet::new();
        for (i, &deg) in indegree.iter().enumerate() {
            if deg == 0 {
                ready.insert(i);
            }
        }

        let mut executed = 0usize;
        let mut terminator_target: Option<usize> = None;

        while let Some(&i) = ready.iter().next() {
            ready.remove(&i);
            self.steps_taken += 1;
            if self.steps_taken > self.step_budget {
                return Err(Trap::StepBudgetExceeded {
                    budget: self.step_budget,
                });
            }

            let ins = block.instructions[i];
            let result = self.execute(&ins, block_idx, i)?;

            if ins.opcode.is_terminator() {
                terminator_target = Some(self.resolve_terminator(&ins, block_idx, i)?);
            }

            self.trace.steps.push(TraceStep {
                step: self.steps_taken,
                block: block_idx,
                block_label: block.label.clone(),
                instr_index: i,
                mnemonic: ins.opcode.mnemonic().to_string(),
                result,
            });

            executed += 1;

            if let Some(exit) = self.pending_exit.take() {
                return Ok(Some(exit));
            }

            for &succ in &successors[i] {
                indegree[succ] -= 1;
                if indegree[succ] == 0 {
                    ready.insert(succ);
                }
            }
        }

        debug_assert_eq!(
            executed, n,
            "dependency graph must be a DAG covering the whole block"
        );

        match self.program.blocks[block_idx].instructions[n - 1].opcode {
            Opcode::Halt => Ok(Some(ExitReason::Halted)),
            _ => {
                self.current_block = terminator_target.expect("terminator must set a target");
                Ok(None)
            }
        }
    }

    /// Reference implementation: executes the current block in plain
    /// program order (index 0, 1, 2, ...) instead of scheduling by
    /// dependency readiness — i.e. exactly what a conventional
    /// program-counter machine would do. This exists purely so property
    /// tests can differentially check that dependency-driven scheduling
    /// (`step_block`) produces identical final state to sequential
    /// execution for any block whose only ordering requirement is the data
    /// dependencies `depgraph::build` encodes. A divergence between the two
    /// would mean the dependency graph is missing an edge somewhere.
    pub fn step_block_sequential(&mut self) -> Result<Option<ExitReason>, Trap> {
        let block_idx = self.current_block;
        let block = self.program.blocks[block_idx].clone();
        let n = block.instructions.len();
        let mut terminator_target = None;

        for (i, ins) in block.instructions.iter().enumerate() {
            self.steps_taken += 1;
            if self.steps_taken > self.step_budget {
                return Err(Trap::StepBudgetExceeded {
                    budget: self.step_budget,
                });
            }
            let result = self.execute(ins, block_idx, i)?;
            if ins.opcode.is_terminator() {
                terminator_target = Some(self.resolve_terminator(ins, block_idx, i)?);
            }
            self.trace.steps.push(TraceStep {
                step: self.steps_taken,
                block: block_idx,
                block_label: block.label.clone(),
                instr_index: i,
                mnemonic: ins.opcode.mnemonic().to_string(),
                result,
            });
            if let Some(exit) = self.pending_exit.take() {
                return Ok(Some(exit));
            }
        }

        match self.program.blocks[block_idx].instructions[n - 1].opcode {
            Opcode::Halt => Ok(Some(ExitReason::Halted)),
            _ => {
                self.current_block = terminator_target.expect("terminator must set a target");
                Ok(None)
            }
        }
    }

    /// Runs to completion using `step_block_sequential` instead of the
    /// dependency-driven scheduler. See `step_block_sequential` for why.
    pub fn run_sequential(&mut self) -> Result<ExitReason, Trap> {
        loop {
            if let Some(exit) = self.step_block_sequential()? {
                self.trace.final_registers = self.regs.snapshot();
                self.trace.final_memory_hash = hash_bytes(self.memory.as_slice());
                self.trace.exit = format!("{exit:?}");
                self.trace.output = self.output.clone();
                return Ok(exit);
            }
        }
    }

    /// Applies one instruction's semantics. Returns the value written to
    /// `dst`, if any, purely for trace/debug readability.
    fn execute(
        &mut self,
        ins: &Instruction,
        block: usize,
        instr: usize,
    ) -> Result<Option<i64>, Trap> {
        use Opcode::*;
        let a = || self.regs.get(ins.src1);
        let b = || self.regs.get(ins.src2);
        let result = match ins.opcode {
            Nop => None,
            Halt => None,
            LoadI => {
                let v = ins.imm as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Mov => {
                let v = a();
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Add => {
                let v = a().wrapping_add(b());
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Sub => {
                let v = a().wrapping_sub(b());
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Mul => {
                let v = a().wrapping_mul(b());
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Div => {
                let (x, y) = (a(), b());
                if y == 0 {
                    return Err(Trap::DivideByZero { block, instr });
                }
                let v = x.wrapping_div(y);
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Mod => {
                let (x, y) = (a(), b());
                if y == 0 {
                    return Err(Trap::DivideByZero { block, instr });
                }
                let v = x.wrapping_rem(y);
                self.regs.set(ins.dst, v);
                Some(v)
            }
            And => {
                let v = a() & b();
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Or => {
                let v = a() | b();
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Xor => {
                let v = a() ^ b();
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Not => {
                let v = !a();
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Shl => {
                let v = a().wrapping_shl(b() as u32);
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Shr => {
                let v = a().wrapping_shr(b() as u32);
                self.regs.set(ins.dst, v);
                Some(v)
            }
            CmpEq => {
                let v = (a() == b()) as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            CmpLt => {
                let v = (a() < b()) as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            CmpLe => {
                let v = (a() <= b()) as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            CmpGt => {
                let v = (a() > b()) as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            CmpGe => {
                let v = (a() >= b()) as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            CmpNe => {
                let v = (a() != b()) as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Load => {
                let addr = (a() + ins.imm as i64) as u64;
                let v = self.memory.load_u64(addr, block, instr)? as i64;
                self.regs.set(ins.dst, v);
                Some(v)
            }
            Store => {
                let addr = (self.regs.get(ins.dst) + ins.imm as i64) as u64;
                let v = self.regs.get(ins.src1) as u64;
                self.memory.store_u64(addr, v, block, instr)?;
                None
            }
            Jmp | Jz | Jnz | Call | Ret => None,
            Syscall => {
                self.do_syscall(ins.imm, block, instr)?;
                None
            }
        };
        let _ = WORD_LEN;
        Ok(result)
    }

    fn do_syscall(&mut self, number: i32, block: usize, instr: usize) -> Result<(), Trap> {
        match number {
            0 => {
                // print r0 as a signed integer
                self.output.push(self.regs.get(0).to_string());
                Ok(())
            }
            1 => {
                self.pending_exit = Some(ExitReason::SyscallExit(self.regs.get(0)));
                Ok(())
            }
            other => Err(Trap::UnknownSyscall {
                number: other,
                block,
                instr,
            }),
        }
    }

    fn resolve_terminator(
        &mut self,
        ins: &Instruction,
        block: usize,
        instr: usize,
    ) -> Result<usize, Trap> {
        use Opcode::*;
        match ins.opcode {
            Jmp => Ok(ins.imm as usize),
            Jz => {
                if self.regs.get(ins.src1) == 0 {
                    Ok(ins.imm as usize)
                } else {
                    Ok(block + 1)
                }
            }
            Jnz => {
                if self.regs.get(ins.src1) != 0 {
                    Ok(ins.imm as usize)
                } else {
                    Ok(block + 1)
                }
            }
            Call => {
                if self.call_stack.len() >= MAX_CALL_DEPTH {
                    return Err(Trap::CallStackOverflow {
                        depth: self.call_stack.len(),
                        block,
                        instr,
                    });
                }
                self.call_stack.push(block + 1);
                Ok(ins.imm as usize)
            }
            Ret => self
                .call_stack
                .pop()
                .ok_or(Trap::CallStackUnderflow { block, instr }),
            Halt => Ok(block),
            _ => unreachable!("non-terminator passed to resolve_terminator"),
        }
    }
}
