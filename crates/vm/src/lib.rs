//! The Machine's execution core: register file, linear memory, the
//! dependency-graph builder that replaces a program counter within a block,
//! the deterministic scheduler, trap handling, and execution tracing.

pub mod depgraph;
pub mod memory;
pub mod regfile;
pub mod trace;
pub mod trap;
pub mod vm;

pub use memory::Memory;
pub use regfile::RegisterFile;
pub use trace::{Trace, TraceStep};
pub use trap::Trap;
pub use vm::{ExitReason, Vm};
