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
    }
}

fn run_debugger(input: &PathBuf) -> Result<()> {
    let program = load_program(input)?;
    program.validate().map_err(|e| anyhow::anyhow!(e))?;
    let mut vm = Vm::new(program);
    println!("imc debugger — commands: step [n], regs, mem <addr>, continue, quit");
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
                    match vm.memory.load_u64(addr, 0, 0) {
                        Ok(v) => println!("mem[{addr}] = {v}"),
                        Err(e) => println!("error: {e}"),
                    }
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
