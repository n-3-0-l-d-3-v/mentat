//! Benchmarks the dependency-driven scheduler's overhead against program
//! size, so "no program counter" is measured, not just claimed.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use isa::{Block, Instruction, Opcode, Program};
use vm::Vm;

/// A single block computing a long chain of independent + dependent
/// arithmetic, sized by `n` instructions, ending in `halt`.
fn chain_program(n: usize) -> Program {
    let mut block = Block::new("bench");
    for i in 0..n {
        let r = (i % 30) as u8;
        block.push(Instruction::new(Opcode::LoadI, r, 0, 0, i as i32));
        block.push(Instruction::new(Opcode::Add, r, r, r, 0));
    }
    block.push(Instruction::new(Opcode::Halt, 0, 0, 0, 0));
    Program {
        blocks: vec![block],
        entry: 0,
    }
}

fn bench_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_run");
    for size in [10usize, 100, 1_000, 10_000] {
        let program = chain_program(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &program, |b, program| {
            b.iter(|| {
                let mut vm = Vm::new(program.clone());
                black_box(vm.run().unwrap());
            });
        });
    }
    group.finish();
}

fn bench_recursive_countdown(c: &mut Criterion) {
    let src = r#"
        block entry:
            loadi r0, 200
            call countdown
        block done:
            halt
        block countdown:
            loadi r1, 0
            cmpeq r2, r0, r1
            jnz r2, base_case
        block recurse:
            loadi r3, 1
            sub r0, r0, r3
            call countdown
        block countdown_tail:
            ret
        block base_case:
            ret
    "#;
    let program = asm::assemble(src).unwrap();
    c.bench_function("vm_recursive_countdown_200", |b| {
        b.iter(|| {
            let mut vm = Vm::new(program.clone()).with_step_budget(1_000_000);
            black_box(vm.run().unwrap());
        });
    });
}

criterion_group!(benches, bench_run, bench_recursive_countdown);
criterion_main!(benches);
