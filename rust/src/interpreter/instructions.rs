//! 字节码指令分发表。
//!
//! 约定：每个 handler 接收 `&mut Interpreter`，负责自身操作数的解码（通过
//! `interp.read_u8/read_u16/...`），返回 [`Flow`] 控制流。
//!
//! 阶段 (a) 仅实现纯算术静态方法需要的指令；其余槽位为 `nop` 占位。

use crate::interpreter::interpreter::{
    DStackSlot, Flow, Interpreter, Registers, ReturnValue, StackSlot,
};

pub type InsFnType = fn(&mut Interpreter) -> Flow;

// ── 通用 handler 模板 ────────────────────────────────────────────────

fn nop(_: &mut Interpreter) -> Flow {
    Flow::Continue
}

// ── 分发表 ───────────────────────────────────────────────────────────

/// 字节码 opcode 上界（JVM 定义 0..=255）。
const INSTRUCTION_COUNT: usize = 256;

/// 初始化后的指令分发表。运行时由 [`init_instruction_table`] 填充。
///
/// 采用延迟初始化而非 const 拼接，是为了避免手写 256 个槽位带来的脆弱性——
/// 新增指令只需在 `init_instruction_table` 里写一行 `table[opcode] = handler`。
static INSTRUCTION_TABLE_RAW: std::sync::OnceLock<[InsFnType; INSTRUCTION_COUNT]> =
    std::sync::OnceLock::new();

/// 返回全局指令表，必要时初始化。
pub fn instruction_table() -> &'static [InsFnType; INSTRUCTION_COUNT] {
    INSTRUCTION_TABLE_RAW.get_or_init(init_instruction_table)
}

fn init_instruction_table() -> [InsFnType; INSTRUCTION_COUNT] {
    let mut t = [nop as InsFnType; INSTRUCTION_COUNT];

    t
}
