//! End-to-end tests against real programs (not toy one-liners): a loop with
//! a function call, and a division-by-zero trap, exercising every
//! acceptance criterion from docs/design/ISA.md — loops, calls, memory,
//! exceptions, and deterministic replay.

use vm::{ExitReason, Trap, Vm};

fn sum_1_to_5_program() -> isa::Program {
    let src = r#"
        block entry:
            loadi r0, 0
            loadi r1, 1
            loadi r2, 5
            jmp loop_check

        block loop_check:
            cmpgt r3, r1, r2
            jnz r3, after_loop

        block loop_body:
            add r0, r0, r1
            loadi r4, 1
            add r1, r1, r4
            jmp loop_check

        block after_loop:
            call print_result

        block done:
            halt

        block print_result:
            syscall 0
            ret
    "#;
    asm::assemble(src).unwrap()
}

#[test]
fn loop_and_function_call_produce_correct_sum() {
    let program = sum_1_to_5_program();
    program.validate().unwrap();
    let mut vm = Vm::new(program);
    let exit = vm.run().unwrap();
    assert_eq!(exit, ExitReason::Halted);
    assert_eq!(vm.output, vec!["15".to_string()]);
    assert_eq!(vm.regs.get(0), 15);
}

#[test]
fn execution_is_deterministically_replayable() {
    let program = sum_1_to_5_program();
    let mut vm1 = Vm::new(program.clone());
    vm1.run().unwrap();
    let mut vm2 = Vm::new(program);
    vm2.run().unwrap();
    assert!(vm1.trace.first_divergence(&vm2.trace).is_none());
    assert!(!vm1.trace.steps.is_empty());
}

#[test]
fn division_by_zero_traps_instead_of_panicking() {
    let src = "block entry:\n  loadi r0, 10\n  loadi r1, 0\n  div r2, r0, r1\n  halt\n";
    let program = asm::assemble(src).unwrap();
    let mut vm = Vm::new(program);
    let err = vm.run().unwrap_err();
    assert_eq!(err, Trap::DivideByZero { block: 0, instr: 2 });
}

#[test]
fn out_of_bounds_memory_access_traps() {
    let src = "block entry:\n  loadi r0, 999999999\n  load r1, r0, 0\n  halt\n";
    let program = asm::assemble(src).unwrap();
    let mut vm = Vm::new(program).with_memory_size(1024);
    let err = vm.run().unwrap_err();
    matches!(err, Trap::OutOfBoundsMemory { .. });
}

#[test]
fn memory_store_then_load_round_trips_through_a_function_call() {
    // stores 77 at address 0, calls a function that loads and doubles it.
    let src = r#"
        block entry:
            loadi r0, 0
            loadi r1, 77
            store r0, r1, 0
            call doubler
        block done:
            halt
        block doubler:
            load r2, r0, 0
            add r2, r2, r2
            mov r0, r2
            syscall 0
            ret
    "#;
    let program = asm::assemble(src).unwrap();
    let mut vm = Vm::new(program);
    vm.run().unwrap();
    assert_eq!(vm.output, vec!["154".to_string()]);
}

#[test]
fn recursive_countdown_uses_the_call_stack_correctly() {
    // countdown(n): if n == 0 return; print n; countdown(n-1).
    // exercises repeated call/ret without an explicit stack discipline
    // beyond the VM's own call_stack of return-block indices.
    let src = r#"
        block entry:
            loadi r0, 3
            call countdown
        block done:
            halt

        block countdown:
            loadi r1, 0
            cmpeq r2, r0, r1
            jnz r2, base_case

        block recurse:
            syscall 0
            loadi r3, 1
            sub r0, r0, r3
            call countdown

        block countdown_tail:
            ret

        block base_case:
            ret
    "#;
    let program = asm::assemble(src).unwrap();
    program.validate().unwrap();
    let mut vm = Vm::new(program).with_step_budget(10_000);
    let exit = vm.run().unwrap();
    assert_eq!(exit, ExitReason::Halted);
    assert_eq!(
        vm.output,
        vec!["3".to_string(), "2".to_string(), "1".to_string()]
    );
}
