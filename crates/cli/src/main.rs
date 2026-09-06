//! `imc` — the Machine's command-line toolchain: assembler, disassembler,
//! runner, deterministic replay checker, and a step debugger, all built on
//! the shared `isa`/`vm`/`asm` crates rather than duplicating logic.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use isa::Program;
use vm::Vm;

#[derive(Parser)]
#[command(name = "imc", about = "The Impossible Machine toolchain", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Assemble a `.imcs` text program into a `.imcp` JSON program file.
    Asm {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Disassemble a `.imcp` program file back into text.
    Disasm { input: PathBuf },
    /// Run a program to completion.
    Run {
        input: PathBuf,
        #[arg(long, default_value_t = 1 << 20)]
        mem_size: usize,
        #[arg(long, default_value_t = 10_000_000)]
        step_budget: u64,
        /// Write the full execution trace to this file.
        #[arg(long)]
        trace: Option<PathBuf>,
    },
    /// Re-run a program and check the resulting trace against a previously
    /// recorded one, proving (or disproving) deterministic replay.
    Replay { input: PathBuf, trace: PathBuf },
    /// Interactive step debugger.
    Debug { input: PathBuf },
    /// Human-readable rendering of a recorded trace: per-block step counts,
    /// hot instructions, and a timeline of the first/last steps.
    Trace {
        trace: PathBuf,
        #[arg(long, default_value_t = 10)]
        timeline: usize,
    },
    /// Runs a program and reports per-opcode and per-block execution
    /// counts plus wall-clock time, so "hot" parts of a program are
    /// measured rather than guessed at.
    Profile {
        input: PathBuf,
        #[arg(long, default_value_t = 1 << 20)]
        mem_size: usize,
        #[arg(long, default_value_t = 10_000_000)]
        step_budget: u64,
    },
}

fn load_program(path: &PathBuf) -> Result<Program> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if path.extension().and_then(|e| e.to_str()) == Some("imcs") {
        asm::assemble(&text).map_err(|e| anyhow::anyhow!(e))
    } else {
        serde_json::from_str(&text)
            .with_context(|| format!("parsing program JSON {}", path.display()))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Asm { input, output } => {
            let source = fs::read_to_string(&input)?;
            let program = asm::assemble(&source).map_err(|e| anyhow::anyhow!(e))?;
            program.validate().map_err(|e| anyhow::anyhow!(e))?;
            fs::write(&output, serde_json::to_string_pretty(&program)?)?;
            println!(
                "assembled {} block(s) -> {}",
                program.blocks.len(),
                output.display()
            );
            Ok(())
        }
        Command::Disasm { input } => {
            let program = load_program(&input)?;
            print!("{}", isa::disasm::disassemble(&program));
            Ok(())
        }
        Command::Run {
            input,
            mem_size,
            step_budget,
            trace,
        } => {
            let program = load_program(&input)?;
            program.validate().map_err(|e| anyhow::anyhow!(e))?;
            let mut vm = Vm::new(program)
                .with_memory_size(mem_size)
                .with_step_budget(step_budget);
            let result = vm.run();
            for line in &vm.output {
                println!("{line}");
            }
            if let Some(trace_path) = trace {
                fs::write(&trace_path, serde_json::to_string_pretty(&vm.trace)?)?;
            }
            match result {
                Ok(exit) => {
                    eprintln!("exit: {exit:?}");
                    Ok(())
                }
                Err(trap) => bail!("trapped: {trap}"),
            }
        }
        Command::Replay { input, trace } => {
            let program = load_program(&input)?;
            program.validate().map_err(|e| anyhow::anyhow!(e))?;
            let recorded: vm::Trace = serde_json::from_str(&fs::read_to_string(&trace)?)?;
            let mut fresh_vm = Vm::new(program);
            let _ = fresh_vm.run();
            match fresh_vm.trace.first_divergence(&recorded) {
                None => {
                    println!(
                        "REPLAY OK: {} steps match exactly",
                        fresh_vm.trace.steps.len()
                    );
                    Ok(())
                }
                Some(i) => bail!("REPLAY DIVERGED at step index {i}"),
            }
        }
        Command::Debug { input } => run_debugger(&input),
        Command::Trace { trace, timeline } => render_trace(&trace, timeline),
        Command::Profile {
            input,
            mem_size,
            step_budget,
        } => profile(&input, mem_size, step_budget),
    }
}

fn render_trace(trace_path: &PathBuf, timeline_len: usize) -> Result<()> {
    let trace: vm::Trace = serde_json::from_str(&fs::read_to_string(trace_path)?)?;
    println!("total steps : {}", trace.steps.len());
    println!("exit        : {}", trace.exit);
    println!("output      : {:?}", trace.output);
    println!();

    let mut by_block: std::collections::BTreeMap<(usize, String), usize> = Default::default();
    let mut by_mnemonic: std::collections::BTreeMap<String, usize> = Default::default();
    for step in &trace.steps {
        *by_block
            .entry((step.block, step.block_label.clone()))
            .or_insert(0) += 1;
        *by_mnemonic.entry(step.mnemonic.clone()).or_insert(0) += 1;
    }

    println!("steps per block:");
    for ((idx, label), count) in &by_block {
        println!("  block {idx} ({label}): {count}");
    }
    println!();

    let mut mnemonic_counts: Vec<(&String, &usize)> = by_mnemonic.iter().collect();
    mnemonic_counts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    println!("hot instructions (by mnemonic):");
    for (mnemonic, count) in &mnemonic_counts {
        println!("  {mnemonic}: {count}");
    }
    println!();

    println!("timeline (first {timeline_len}):");
    for step in trace.steps.iter().take(timeline_len) {
        println!(
            "  step {:>6}  block {:>3} ({})  instr {:>3}  {}  -> {:?}",
            step.step, step.block, step.block_label, step.instr_index, step.mnemonic, step.result
        );
    }
    if trace.steps.len() > timeline_len {
        println!("  ...");
        println!("timeline (last {timeline_len}):");
        for step in trace
            .steps
            .iter()
            .rev()
            .take(timeline_len)
            .collect::<Vec<_>>()
            .iter()
            .rev()
        {
            println!(
                "  step {:>6}  block {:>3} ({})  instr {:>3}  {}  -> {:?}",
                step.step,
                step.block,
                step.block_label,
                step.instr_index,
                step.mnemonic,
                step.result
            );
        }
    }
    Ok(())
}

fn profile(input: &PathBuf, mem_size: usize, step_budget: u64) -> Result<()> {
    let program = load_program(input)?;
    program.validate().map_err(|e| anyhow::anyhow!(e))?;
    let mut vm = Vm::new(program)
        .with_memory_size(mem_size)
        .with_step_budget(step_budget);

    let start = std::time::Instant::now();
    let result = vm.run();
    let elapsed = start.elapsed();

    let mut by_block: std::collections::BTreeMap<(usize, String), usize> = Default::default();
    let mut by_mnemonic: std::collections::BTreeMap<String, usize> = Default::default();
    for step in &vm.trace.steps {
        *by_block
            .entry((step.block, step.block_label.clone()))
            .or_insert(0) += 1;
        *by_mnemonic.entry(step.mnemonic.clone()).or_insert(0) += 1;
    }
    let total = vm.trace.steps.len().max(1);

    println!("wall time     : {elapsed:?}");
    println!("total steps   : {}", vm.trace.steps.len());
    if total > 0 {
        println!(
            "ns / step     : {:.1}",
            elapsed.as_nanos() as f64 / total as f64
        );
    }
    println!();

    println!("execution share by block:");
    let mut block_counts: Vec<_> = by_block.into_iter().collect();
    block_counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for ((idx, label), count) in &block_counts {
        println!(
            "  block {idx:>3} ({label:<20}) {count:>8}  ({:>5.1}%)",
            100.0 * *count as f64 / total as f64
        );
    }
    println!();

    println!("execution share by opcode:");
    let mut mnemonic_counts: Vec<_> = by_mnemonic.into_iter().collect();
    mnemonic_counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (mnemonic, count) in &mnemonic_counts {
        println!(
            "  {mnemonic:<10} {count:>8}  ({:>5.1}%)",
            100.0 * *count as f64 / total as f64
        );
    }

    match result {
        Ok(exit) => {
            println!();
            println!("exit: {exit:?}");
            Ok(())
        }
        Err(trap) => bail!("trapped: {trap}"),
    }
}

fn run_debugger(input: &PathBuf) -> Result<()> {
    let program = load_program(input)?;
    program.validate().map_err(|e| anyhow::anyhow!(e))?;
    let mut vm = Vm::new(program);
    println!("imc debugger — commands: step [n], regs, mem <addr> [count], continue, quit");
    let stdin = io::stdin();
    loop {
        print!("(imc) ");
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("regs") => {
                for (i, v) in vm.regs.snapshot().iter().enumerate() {
                    print!("r{i}={v} ");
                }
                println!();
            }
            Some("mem") => {
                if let Some(addr_s) = parts.next() {
                    let addr: u64 = addr_s.parse().unwrap_or(0);
                    let count: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                    for i in 0..count {
                        let a = addr + i * 8;
                        match vm.memory.load_u64(a, 0, 0) {
                            Ok(v) => println!("mem[{a}] = {v} (0x{v:016x})"),
                            Err(e) => {
                                println!("error: {e}");
                                break;
                            }
                        }
                    }
                } else {
                    println!("usage: mem <addr> [count]");
                }
            }
            Some("step") => {
                let n: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                for _ in 0..n {
                    match vm.step_block() {
                        Ok(Some(exit)) => {
                            println!("program finished: {exit:?}");
                            break;
                        }
                        Ok(None) => {
                            println!("-> block {}", vm.current_block);
                        }
                        Err(trap) => {
                            println!("trap: {trap}");
                            break;
                        }
                    }
                }
            }
            Some("continue") => match vm.run() {
                Ok(exit) => println!("program finished: {exit:?}"),
                Err(trap) => println!("trap: {trap}"),
            },
            Some("quit") | Some("q") => break,
            Some(other) => println!("unknown command: {other}"),
            None => {}
        }
    }
    Ok(())
}
