//! REVM Inspector for EVM tracing.

use alloc::string::String;
use revm::{
    context::JournalTr,
    context_interface::ContextTr,
    interpreter::{
        interpreter_types::{Jumps, MemoryTr, StackTr},
        Interpreter, InterpreterTypes,
    },
    state::bytecode::opcode::OpCode,
    Inspector,
};

/// Inspector that logs EVM execution steps.
#[derive(Debug, Default)]
pub struct TracingInspector;

impl TracingInspector {
    /// Creates a new [`TracingInspector`].
    pub fn new() -> Self {
        Self
    }

    /// Formats the stack for logging (decimal format).
    fn format_stack<S: StackTr>(stack: &S) -> String {
        let data = stack.data();
        let mut result = String::new();
        for (i, value) in data.iter().rev().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&alloc::format!("{}", value));
        }
        result
    }
}

impl<CTX, INTR> Inspector<CTX, INTR> for TracingInspector
where
    CTX: ContextTr,
    INTR: InterpreterTypes,
{
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let pc = interp.bytecode.pc();
        let opcode = interp.bytecode.opcode();
        let gas = interp.gas.remaining();
        let refunded = interp.gas.refunded();
        let depth = context.journal_mut().depth();
        let data_size = interp.memory.size();

        let op_name = OpCode::new(opcode).map_or("UNKNOWN", |op| op.as_str());
        let stack_str = Self::format_stack(&interp.stack);

        info!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {}:({}) refund:{:#x}({}) Stack:[{}], Data size:{}",
            depth, pc, gas, gas, op_name, opcode, refunded, refunded, stack_str, data_size
        );
    }
}