//! REVM Inspector for EVM tracing.

use alloc::string::String;
use revm::{
    context::JournalTr,
    context_interface::ContextTr,
    interpreter::{
        interpreter_types::{Jumps, MemoryTr, StackTr},
        CallInputs, CallOutcome, Interpreter, InterpreterTypes,
    },
    state::bytecode::opcode::OpCode,
    Inspector,
};

/// Inspector that logs EVM execution steps.
#[derive(Debug, Default)]
pub struct TracingInspector {
    /// Gas limit at the start of top-level call.
    call_gas_limit: u64,
}

impl TracingInspector {
    /// Creates a new [`TracingInspector`].
    pub fn new() -> Self {
        Self { call_gas_limit: 0 }
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

    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let depth = context.journal_mut().depth();
        if depth == 0 {
            self.call_gas_limit = inputs.gas_limit;
            info!(
                target: "tx_trace",
                caller = ?inputs.caller,
                target = ?inputs.target_address,
                gas_limit = inputs.gas_limit,
                "Transaction call started"
            );
        }
        None
    }

    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let depth = context.journal_mut().depth();
        if depth == 0 {
            let gas = outcome.gas();
            let gas_used = self.call_gas_limit.saturating_sub(gas.remaining());
            info!(
                target: "tx_trace",
                caller = ?inputs.caller,
                target = ?inputs.target_address,
                gas_limit = self.call_gas_limit,
                gas_used = gas_used,
                gas_refunded = gas.refunded(),
                success = ?outcome.instruction_result(),
                "Transaction call ended"
            );
        }
    }
}