//! 字节码指令分发表。
//!
//! 约定：每个 handler 接收 `&mut Interpreter`，负责自身操作数的解码（通过
//! `interp.read_u8/read_u16/...`），返回 [`Flow`] 控制流。
//!
//! 阶段 (a) 仅实现纯算术静态方法需要的指令；其余槽位为 `nop` 占位。

#![allow(clippy::erasing_op)]

use crate::interpreter::interpreter::{DStackSlot, Flow, Interpreter, ReturnValue, StackSlot};
use crate::oops::cp_entry::CPEntry;

pub type InsFnType = fn(&mut Interpreter) -> Flow;

// ── 通用 handler 模板 ────────────────────────────────────────────────

fn nop(_: &mut Interpreter) -> Flow {
    Flow::Continue
}

// ── 局部变量 load/store 辅助 ─────────────────────────────────────────
//
// 约定：handler 进入时 pc 已越过 opcode（由 run_loop 推进 1）。
// 宽变体（iload/lload/...）需要再读 1 字节索引；短变体（iload_0..3）索引已固定。

#[inline]
fn load_local(interp: &mut Interpreter, idx: usize) -> Flow {
    let v = unsafe { *interp.regs().bp.add(idx) };
    interp.push_slot(v);
    Flow::Continue
}

#[inline]
fn store_local(interp: &mut Interpreter, idx: usize) -> Flow {
    let v = interp.pop_slot();
    unsafe {
        *interp.regs().bp.add(idx) = v;
    }
    Flow::Continue
}

// long / double 在局部变量区同样占 2 槽，布局与栈一致：高 32 位在低地址。
#[inline]
fn load_local_long(interp: &mut Interpreter, idx: usize) -> Flow {
    let hi = unsafe { *interp.regs().bp.add(idx) };
    let lo = unsafe { *interp.regs().bp.add(idx + 1) };
    interp.push_slot(hi);
    interp.push_slot(lo);
    Flow::Continue
}

#[inline]
fn store_local_long(interp: &mut Interpreter, idx: usize) -> Flow {
    let lo = interp.pop_slot();
    let hi = interp.pop_slot();
    unsafe {
        *interp.regs().bp.add(idx) = hi;
        *interp.regs().bp.add(idx + 1) = lo;
    }
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

    // ── 常量推送 ────────────────────────────────────────────────────
    t[0x01] = aconst_null; // 本阶段仅作为占位压入 0；真正引用语义在阶段 4
    t[0x02] = iconst_m1;
    t[0x03] = iconst_0;
    t[0x04] = iconst_1;
    t[0x05] = iconst_2;
    t[0x06] = iconst_3;
    t[0x07] = iconst_4;
    t[0x08] = iconst_5;

    // ── 常量 push ────────────────────────────────────────────────────
    t[0x10] = bipush;
    t[0x11] = sipush;

    // ── iload 系列 ───────────────────────────────────────────────────
    t[0x15] = iload;
    t[0x1a] = iload_0;
    t[0x1b] = iload_1;
    t[0x1c] = iload_2;
    t[0x1d] = iload_3;

    // ── lload 系列 ───────────────────────────────────────────────────
    t[0x16] = lload;
    t[0x1e] = lload_0;
    t[0x1f] = lload_1;
    t[0x20] = lload_2;
    t[0x21] = lload_3;

    // ── istore 系列 ──────────────────────────────────────────────────
    t[0x36] = istore;
    t[0x3b] = istore_0;
    t[0x3c] = istore_1;
    t[0x3d] = istore_2;
    t[0x3e] = istore_3;

    // ── lstore 系列 ──────────────────────────────────────────────────
    t[0x37] = lstore;
    t[0x3f] = lstore_0;
    t[0x40] = lstore_1;
    t[0x41] = lstore_2;
    t[0x42] = lstore_3;

    // ── 常量 push（long / ldc / ldc2_w）──────────────────────────────
    t[0x09] = lconst_0;
    t[0x0a] = lconst_1;
    t[0x12] = ldc;
    t[0x14] = ldc2_w;

    // ── int 算术 / 位运算 ────────────────────────────────────────────
    t[0x60] = iadd;
    t[0x64] = isub;
    t[0x68] = imul;
    t[0x6c] = idiv;
    t[0x70] = irem;
    t[0x74] = ineg;
    t[0x78] = ishl;
    t[0x7a] = ishr;
    t[0x7c] = iushr;
    t[0x7e] = iand;
    t[0x80] = ior;
    t[0x82] = ixor;
    t[0x84] = iinc;

    // ── long 算术 / 位运算 ───────────────────────────────────────────
    t[0x61] = ladd;
    t[0x65] = lsub;
    t[0x69] = lmul;
    t[0x6d] = ldiv;
    t[0x71] = lrem;
    t[0x75] = lneg;
    t[0x79] = lshl;
    t[0x7b] = lshr;
    t[0x7d] = lushr;
    t[0x7f] = land_;
    t[0x81] = lor_;
    t[0x83] = lxor_;

    // ── 返回 ────────────────────────────────────────────────────────
    t[0xac] = ireturn;
    t[0xad] = lreturn;
    t[0xb1] = return_void;

    t
}

// ── 常量推送 handler ──────────────────────────────────────────────────

fn aconst_null(interp: &mut Interpreter) -> Flow {
    interp.push_slot(0);
    Flow::Continue
}

fn iconst_m1(interp: &mut Interpreter) -> Flow {
    interp.push_slot(-1);
    Flow::Continue
}

fn iconst_0(interp: &mut Interpreter) -> Flow {
    interp.push_slot(0);
    Flow::Continue
}

fn iconst_1(interp: &mut Interpreter) -> Flow {
    interp.push_slot(1);
    Flow::Continue
}

fn iconst_2(interp: &mut Interpreter) -> Flow {
    interp.push_slot(2);
    Flow::Continue
}

fn iconst_3(interp: &mut Interpreter) -> Flow {
    interp.push_slot(3);
    Flow::Continue
}

fn iconst_4(interp: &mut Interpreter) -> Flow {
    interp.push_slot(4);
    Flow::Continue
}

fn iconst_5(interp: &mut Interpreter) -> Flow {
    interp.push_slot(5);
    Flow::Continue
}

fn bipush(interp: &mut Interpreter) -> Flow {
    let v = interp.read_i8() as i32;
    interp.push_slot(v);
    Flow::Continue
}

fn sipush(interp: &mut Interpreter) -> Flow {
    let v = interp.read_i16() as i32;
    interp.push_slot(v);
    Flow::Continue
}

// ── iload / istore handler ─────────────────────────────────────────────

fn iload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    load_local(interp, idx)
}

fn iload_0(interp: &mut Interpreter) -> Flow {
    load_local(interp, 0)
}
fn iload_1(interp: &mut Interpreter) -> Flow {
    load_local(interp, 1)
}
fn iload_2(interp: &mut Interpreter) -> Flow {
    load_local(interp, 2)
}
fn iload_3(interp: &mut Interpreter) -> Flow {
    load_local(interp, 3)
}

fn istore(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    store_local(interp, idx)
}

fn istore_0(interp: &mut Interpreter) -> Flow {
    store_local(interp, 0)
}
fn istore_1(interp: &mut Interpreter) -> Flow {
    store_local(interp, 1)
}
fn istore_2(interp: &mut Interpreter) -> Flow {
    store_local(interp, 2)
}
fn istore_3(interp: &mut Interpreter) -> Flow {
    store_local(interp, 3)
}

// ── int 算术 handler ──────────────────────────────────────────────────

fn iadd(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_add(b));
    Flow::Continue
}

fn iinc(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let c = interp.read_i8() as i32;
    unsafe {
        let slot = interp.regs().bp.add(idx);
        *slot = (*slot).wrapping_add(c);
    }
    Flow::Continue
}

// ── 返回 handler ─────────────────────────────────────────────────────

fn ireturn(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    Flow::Return(Some(ReturnValue::Int(v)))
}

fn return_void(interp: &mut Interpreter) -> Flow {
    Flow::Return(None)
}

// ── 阶段 2：long / ldc / 算术 handler ──────────────────────────────────

fn lconst_0(interp: &mut Interpreter) -> Flow {
    interp.push_long(0);
    Flow::Continue
}

fn lconst_1(interp: &mut Interpreter) -> Flow {
    interp.push_long(1);
    Flow::Continue
}

/// `ldc`：1 字节索引。  本阶段仅处理 int；float / String 留到阶段 3。
fn ldc(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        CPEntry::Integer(v) => interp.push_slot(*v),
        _ => unimplemented!("ldc non-int entry at {}", idx),
    }
    Flow::Continue
}

/// `ldc2_w`：2 字节索引，加载 long / double。
fn ldc2_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        CPEntry::Long(v) => interp.push_long(*v),
        _ => unimplemented!("ldc2_w non-long entry at {}", idx),
    }
    Flow::Continue
}

// ── lload / lstore handler ─────────────────────────────────────────────

fn lload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    load_local_long(interp, idx)
}

fn lload_0(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 0)
}
fn lload_1(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 1)
}
fn lload_2(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 2)
}
fn lload_3(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 3)
}

fn lstore(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    store_local_long(interp, idx)
}

fn lstore_0(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 0)
}
fn lstore_1(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 1)
}
fn lstore_2(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 2)
}
fn lstore_3(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 3)
}

// ── int 算术 / 位运算 handler ─────────────────────────────────────────

fn isub(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_sub(b));
    Flow::Continue
}

fn imul(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_mul(b));
    Flow::Continue
}

fn idiv(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_div(b));
    Flow::Continue
}

fn irem(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_rem(b));
    Flow::Continue
}

fn ineg(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_neg());
    Flow::Continue
}

fn ishl(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_shl((b & 0x1f) as u32));
    Flow::Continue
}

fn ishr(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a.wrapping_shr((b & 0x1f) as u32));
    Flow::Continue
}

fn iushr(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(((a as u32).wrapping_shr((b & 0x1f) as u32)) as i32);
    Flow::Continue
}

fn iand(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a & b);
    Flow::Continue
}

fn ior(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a | b);
    Flow::Continue
}

fn ixor(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.push_slot(a ^ b);
    Flow::Continue
}

// ── long 算术 / 位运算 handler ─────────────────────────────────────────
//
// long 移位仅取 int 的低 6 位（0x3f），其余二元运算采用标准 Rust 语义。

fn ladd(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a.wrapping_add(b));
    Flow::Continue
}

fn lsub(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a.wrapping_sub(b));
    Flow::Continue
}

fn lmul(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a.wrapping_mul(b));
    Flow::Continue
}

fn ldiv(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a.wrapping_div(b));
    Flow::Continue
}

fn lrem(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a.wrapping_rem(b));
    Flow::Continue
}

fn lneg(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_long();
    interp.push_long(a.wrapping_neg());
    Flow::Continue
}

fn lshl(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot() as u32 & 0x3f;
    let a = interp.pop_long();
    interp.push_long(a.wrapping_shl(b));
    Flow::Continue
}

fn lshr(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot() as u32 & 0x3f;
    let a = interp.pop_long();
    interp.push_long(a.wrapping_shr(b));
    Flow::Continue
}

fn lushr(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot() as u32 & 0x3f;
    let a = interp.pop_long();
    interp.push_long(((a as u64).wrapping_shr(b)) as i64);
    Flow::Continue
}

fn land_(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a & b);
    Flow::Continue
}

fn lor_(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a | b);
    Flow::Continue
}

fn lxor_(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    interp.push_long(a ^ b);
    Flow::Continue
}

// ── long 返回 handler ─────────────────────────────────────────────────

fn lreturn(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_long();
    Flow::Return(Some(ReturnValue::Long(v)))
}
