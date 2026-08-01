use crate::{
    engine::{
        exec_error::{ExecError, ExecResult},
        interpreter::instructions::{
            branches::*, calls::*, constants::*, control::*, loads::*, math::*, stack::*, stores::*,
        },
        outcome::StepOutcome,
    },
    runtime::java_thread::JavaThread,
};

#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn execute_one(&mut self, thread: &mut JavaThread) -> ExecResult<StepOutcome> {
        let frame = thread
            .stack_mut()
            .current_interpreter_mut()
            .map_err(ExecError::Stack)?;

        let opcode = frame.fetch_opcode()?;

        match opcode {
            // Constants and immediate values.
            0x00 => nop(frame),
            0x01 => aconst_null(frame),
            0x02 => iconst_n::<-1>(frame),
            0x03 => iconst_n::<0>(frame),
            0x04 => iconst_n::<1>(frame),
            0x05 => iconst_n::<2>(frame),
            0x06 => iconst_n::<3>(frame),
            0x07 => iconst_n::<4>(frame),
            0x08 => iconst_n::<5>(frame),
            0x09 => lconst_n::<0>(frame),
            0x0a => lconst_n::<1>(frame),
            0x0b => fconst_n::<0>(frame),
            0x0c => fconst_n::<1>(frame),
            0x0d => fconst_n::<2>(frame),
            0x0e => dconst_n::<0>(frame),
            0x0f => dconst_n::<1>(frame),
            0x10 => bipush(frame),
            0x11 => sipush(frame),
            0x12 => ldc(frame),
            0x13 => ldc_w(frame),
            0x14 => ldc2_w(frame),

            // Indexed loads.
            0x15 => iload(frame),
            0x16 => lload(frame),
            0x17 => fload(frame),
            0x18 => dload(frame),
            0x19 => aload(frame),

            // Fixed-index loads.
            0x1a => iload_n::<0>(frame),
            0x1b => iload_n::<1>(frame),
            0x1c => iload_n::<2>(frame),
            0x1d => iload_n::<3>(frame),
            0x1e => lload_n::<0>(frame),
            0x1f => lload_n::<1>(frame),
            0x20 => lload_n::<2>(frame),
            0x21 => lload_n::<3>(frame),
            0x22 => fload_n::<0>(frame),
            0x23 => fload_n::<1>(frame),
            0x24 => fload_n::<2>(frame),
            0x25 => fload_n::<3>(frame),
            0x26 => dload_n::<0>(frame),
            0x27 => dload_n::<1>(frame),
            0x28 => dload_n::<2>(frame),
            0x29 => dload_n::<3>(frame),
            0x2a => aload_n::<0>(frame),
            0x2b => aload_n::<1>(frame),
            0x2c => aload_n::<2>(frame),
            0x2d => aload_n::<3>(frame),

            // Indexed stores.
            0x36 => istore(frame),
            0x37 => lstore(frame),
            0x38 => fstore(frame),
            0x39 => dstore(frame),
            0x3a => astore(frame),

            // Fixed-index stores.
            0x3b => istore_n::<0>(frame),
            0x3c => istore_n::<1>(frame),
            0x3d => istore_n::<2>(frame),
            0x3e => istore_n::<3>(frame),
            0x3f => lstore_n::<0>(frame),
            0x40 => lstore_n::<1>(frame),
            0x41 => lstore_n::<2>(frame),
            0x42 => lstore_n::<3>(frame),
            0x43 => fstore_n::<0>(frame),
            0x44 => fstore_n::<1>(frame),
            0x45 => fstore_n::<2>(frame),
            0x46 => fstore_n::<3>(frame),
            0x47 => dstore_n::<0>(frame),
            0x48 => dstore_n::<1>(frame),
            0x49 => dstore_n::<2>(frame),
            0x4a => dstore_n::<3>(frame),
            0x4b => astore_n::<0>(frame),
            0x4c => astore_n::<1>(frame),
            0x4d => astore_n::<2>(frame),
            0x4e => astore_n::<3>(frame),

            // Operand stack manipulation.
            0x57 => pop(frame),
            0x58 => pop2(frame),
            0x59 => dup(frame),
            0x5a => dup_x1(frame),
            0x5b => dup_x2(frame),
            0x5c => dup2(frame),
            0x5d => dup2_x1(frame),
            0x5e => dup2_x2(frame),
            0x5f => swap(frame),

            // Arithmetic, shifts, bitwise operations, and local increment.
            0x60 => iadd(frame),
            0x61 => ladd(frame),
            0x62 => fadd(frame),
            0x63 => dadd(frame),
            0x64 => isub(frame),
            0x65 => lsub(frame),
            0x66 => fsub(frame),
            0x67 => dsub(frame),
            0x68 => imul(frame),
            0x69 => lmul(frame),
            0x6a => fmul(frame),
            0x6b => dmul(frame),
            0x6c => idiv(frame),
            0x6d => ldiv(frame),
            0x6e => fdiv(frame),
            0x6f => ddiv(frame),
            0x70 => irem(frame),
            0x71 => lrem(frame),
            0x72 => frem(frame),
            0x73 => drem(frame),
            0x74 => ineg(frame),
            0x75 => lneg(frame),
            0x76 => fneg(frame),
            0x77 => dneg(frame),
            0x78 => ishl(frame),
            0x79 => lshl(frame),
            0x7a => ishr(frame),
            0x7b => lshr(frame),
            0x7c => iushr(frame),
            0x7d => lushr(frame),
            0x7e => iand(frame),
            0x7f => land(frame),
            0x80 => ior(frame),
            0x81 => lor(frame),
            0x82 => ixor(frame),
            0x83 => lxor(frame),
            0x84 => iinc(frame),

            // Integer comparisons and control flow.
            0x99 => ifeq(frame),
            0x9a => ifne(frame),
            0x9b => iflt(frame),
            0x9c => ifge(frame),
            0x9d => ifgt(frame),
            0x9e => ifle(frame),
            0x9f => if_icmpeq(frame),
            0xa0 => if_icmpne(frame),
            0xa1 => if_icmplt(frame),
            0xa2 => if_icmpge(frame),
            0xa3 => if_icmpgt(frame),
            0xa4 => if_icmple(frame),
            0xa7 => goto(frame),

            // Method invocation.
            0xb8 => invokestatic(frame),

            // Method returns.
            0xac => ireturn(frame),
            0xad => lreturn(frame),
            0xae => freturn(frame),
            0xaf => dreturn(frame),
            0xb0 => areturn(frame),
            0xb1 => return_void(frame),

            unsupported => Err(ExecError::UnsupportedOpcode {
                opcode: unsupported,
                bci: frame.last_pc(),
            }),
        }
    }
}
