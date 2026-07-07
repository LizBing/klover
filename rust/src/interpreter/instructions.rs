//! 字节码指令分发表。
//!
//! 约定：每个 handler 接收 `&mut Interpreter`，负责自身操作数的解码（通过
//! `interp.read_u8/read_u16/...`），返回 [`Flow`] 控制流。
//!
//! 阶段 (a) 仅实现纯算术静态方法需要的指令；其余槽位为 `nop` 占位。

#![allow(clippy::erasing_op)]

use crate::interpreter::interpreter::{Flow, Interpreter, ReturnValue};
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

// float 占 1 槽，以 IEEE-754 位模式存放。
#[inline]
fn push_f32(interp: &mut Interpreter, v: f32) {
    interp.push_slot(v.to_bits() as i32);
}

#[inline]
fn pop_f32(interp: &mut Interpreter) -> f32 {
    f32::from_bits(interp.pop_slot() as u32)
}

// double 占 2 槽，与 long 同布局。
#[inline]
fn push_f64(interp: &mut Interpreter, v: f64) {
    interp.push_long(v.to_bits() as i64);
}

#[inline]
fn pop_f64(interp: &mut Interpreter) -> f64 {
    f64::from_bits(interp.pop_long() as u64)
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

    // ── fload 系列 ───────────────────────────────────────────────────
    t[0x17] = fload;
    t[0x22] = fload_0;
    t[0x23] = fload_1;
    t[0x24] = fload_2;
    t[0x25] = fload_3;

    // ── dload 系列 ───────────────────────────────────────────────────
    t[0x18] = dload;
    t[0x26] = dload_0;
    t[0x27] = dload_1;
    t[0x28] = dload_2;
    t[0x29] = dload_3;

    // ── aload 系列（阶段 4 才真正用，本阶段先注册以便编译）────────
    t[0x19] = aload;
    t[0x2a] = aload_0;
    t[0x2b] = aload_1;
    t[0x2c] = aload_2;
    t[0x2d] = aload_3;

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

    // ── fstore 系列 ──────────────────────────────────────────────────
    t[0x38] = fstore;
    t[0x43] = fstore_0;
    t[0x44] = fstore_1;
    t[0x45] = fstore_2;
    t[0x46] = fstore_3;

    // ── dstore 系列 ──────────────────────────────────────────────────
    t[0x39] = dstore;
    t[0x47] = dstore_0;
    t[0x48] = dstore_1;
    t[0x49] = dstore_2;
    t[0x4a] = dstore_3;

    // ── astore 系列（阶段 4 才真正用，本阶段先注册以便编译）────────
    t[0x3a] = astore;
    t[0x4b] = astore_0;
    t[0x4c] = astore_1;
    t[0x4d] = astore_2;
    t[0x4e] = astore_3;

    // ── 常量 push（long / ldc / ldc2_w）──────────────────────────────
    t[0x09] = lconst_0;
    t[0x0a] = lconst_1;
    t[0x0b] = fconst_0;
    t[0x0c] = fconst_1;
    t[0x0d] = fconst_2;
    t[0x0e] = dconst_0;
    t[0x0f] = dconst_1;
    t[0x12] = ldc;
    t[0x13] = ldc_w;
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

    // ── float 算术 ───────────────────────────────────────────────────
    t[0x62] = fadd;
    t[0x66] = fsub;
    t[0x6a] = fmul;
    t[0x6e] = fdiv;
    t[0x72] = frem;
    t[0x76] = fneg;

    // ── double 算术 ──────────────────────────────────────────────────
    t[0x63] = dadd;
    t[0x67] = dsub;
    t[0x6b] = dmul;
    t[0x6f] = ddiv;
    t[0x73] = drem;
    t[0x77] = dneg;

    // ── 类型转换 ─────────────────────────────────────────────────────
    t[0x85] = i2l;
    t[0x86] = i2f;
    t[0x87] = i2d;
    t[0x88] = l2i;
    t[0x89] = l2f;
    t[0x8a] = l2d;
    t[0x8b] = f2i;
    t[0x8c] = f2l;
    t[0x8d] = f2d;
    t[0x8e] = d2i;
    t[0x8f] = d2l;
    t[0x90] = d2f;

    // ── 比较 ─────────────────────────────────────────────────────────
    t[0x94] = lcmp;
    t[0x95] = fcmpl;
    t[0x96] = fcmpg;
    t[0x97] = dcmpl;
    t[0x98] = dcmpg;

    // ── 条件跳转（int）────────────────────────────────────────────
    t[0x99] = ifeq;
    t[0x9a] = ifne;
    t[0x9b] = iflt;
    t[0x9c] = ifge;
    t[0x9d] = ifgt;
    t[0x9e] = ifle;

    // ── 条件跳转（int 比较）──────────────────────────────────────
    t[0x9f] = if_icmpeq;
    t[0xa0] = if_icmpne;
    t[0xa1] = if_icmplt;
    t[0xa2] = if_icmpge;
    t[0xa3] = if_icmpgt;
    t[0xa4] = if_icmple;

    // ── 条件跳转（引用比较）────────────────────────────────────
    t[0xa5] = if_acmpeq;
    t[0xa6] = if_acmpne;
    t[0xc6] = ifnull;
    t[0xc7] = ifnonnull;

    // ── 无条件跳转 / switch ─────────────────────────────────────────
    t[0xa7] = goto_;
    t[0xaa] = tableswitch;
    t[0xab] = lookupswitch;
    t[0xc8] = goto_w;

    // ── 栈操作 ─────────────────────────────────────────────────────
    t[0x57] = pop;
    t[0x58] = pop2;
    t[0x59] = dup;
    t[0x5a] = dup_x1;
    t[0x5b] = dup_x2;
    t[0x5c] = dup2;
    t[0x5d] = dup2_x1;
    t[0x5e] = dup2_x2;
    t[0x5f] = swap;

    // ── 返回 ────────────────────────────────────────────────────────
    t[0xac] = ireturn;
    t[0xad] = lreturn;
    t[0xae] = freturn;
    t[0xaf] = dreturn;
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

/// `ldc`：1 字节索引。  本阶段处理 int / float；String 留到阶段 4。
fn ldc(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        Some(CPEntry::Integer(v)) => interp.push_slot(*v),
        Some(CPEntry::Float(v)) => push_f32(interp, *v),
        _ => unimplemented!("ldc entry at {}", idx),
    }
    Flow::Continue
}

/// `ldc_w`：2 字节索引。
fn ldc_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        Some(CPEntry::Integer(v)) => interp.push_slot(*v),
        Some(CPEntry::Float(v)) => push_f32(interp, *v),
        _ => unimplemented!("ldc_w entry at {}", idx),
    }
    Flow::Continue
}

/// `ldc2_w`：2 字节索引，加载 long / double。
fn ldc2_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        Some(CPEntry::Long(v)) => interp.push_long(*v),
        Some(CPEntry::Double(v)) => push_f64(interp, *v),
        _ => unimplemented!("ldc2_w entry at {}", idx),
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

// ── 阶段 3：float / double / 转换 / 比较 / 控制流 handler ──────────────

// ── float/double 常量与 load/store ────────────────────────────────────

fn fconst_0(interp: &mut Interpreter) -> Flow {
    push_f32(interp, 0.0);
    Flow::Continue
}
fn fconst_1(interp: &mut Interpreter) -> Flow {
    push_f32(interp, 1.0);
    Flow::Continue
}
fn fconst_2(interp: &mut Interpreter) -> Flow {
    push_f32(interp, 2.0);
    Flow::Continue
}

fn dconst_0(interp: &mut Interpreter) -> Flow {
    push_f64(interp, 0.0);
    Flow::Continue
}
fn dconst_1(interp: &mut Interpreter) -> Flow {
    push_f64(interp, 1.0);
    Flow::Continue
}

fn fload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    load_local(interp, idx) // float 与 int 共享 1 槽存储
}
fn fload_0(interp: &mut Interpreter) -> Flow {
    load_local(interp, 0)
}
fn fload_1(interp: &mut Interpreter) -> Flow {
    load_local(interp, 1)
}
fn fload_2(interp: &mut Interpreter) -> Flow {
    load_local(interp, 2)
}
fn fload_3(interp: &mut Interpreter) -> Flow {
    load_local(interp, 3)
}

fn fstore(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    store_local(interp, idx)
}
fn fstore_0(interp: &mut Interpreter) -> Flow {
    store_local(interp, 0)
}
fn fstore_1(interp: &mut Interpreter) -> Flow {
    store_local(interp, 1)
}
fn fstore_2(interp: &mut Interpreter) -> Flow {
    store_local(interp, 2)
}
fn fstore_3(interp: &mut Interpreter) -> Flow {
    store_local(interp, 3)
}

fn dload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    load_local_long(interp, idx) // double 与 long 共享双槽存储
}
fn dload_0(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 0)
}
fn dload_1(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 1)
}
fn dload_2(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 2)
}
fn dload_3(interp: &mut Interpreter) -> Flow {
    load_local_long(interp, 3)
}

fn dstore(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    store_local_long(interp, idx)
}
fn dstore_0(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 0)
}
fn dstore_1(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 1)
}
fn dstore_2(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 2)
}
fn dstore_3(interp: &mut Interpreter) -> Flow {
    store_local_long(interp, 3)
}

// aload / astore：本阶段仅作为引用占位（未实现真实语义，阶段 4 补全）。
fn aload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    load_local(interp, idx)
}
fn aload_0(interp: &mut Interpreter) -> Flow {
    load_local(interp, 0)
}
fn aload_1(interp: &mut Interpreter) -> Flow {
    load_local(interp, 1)
}
fn aload_2(interp: &mut Interpreter) -> Flow {
    load_local(interp, 2)
}
fn aload_3(interp: &mut Interpreter) -> Flow {
    load_local(interp, 3)
}

fn astore(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    store_local(interp, idx)
}
fn astore_0(interp: &mut Interpreter) -> Flow {
    store_local(interp, 0)
}
fn astore_1(interp: &mut Interpreter) -> Flow {
    store_local(interp, 1)
}
fn astore_2(interp: &mut Interpreter) -> Flow {
    store_local(interp, 2)
}
fn astore_3(interp: &mut Interpreter) -> Flow {
    store_local(interp, 3)
}

// ── float 算术 ────────────────────────────────────────────────────────

fn fadd(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    push_f32(interp, a + b);
    Flow::Continue
}
fn fsub(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    push_f32(interp, a - b);
    Flow::Continue
}
fn fmul(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    push_f32(interp, a * b);
    Flow::Continue
}
fn fdiv(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    push_f32(interp, a / b);
    Flow::Continue
}
fn frem(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    push_f32(interp, a % b);
    Flow::Continue
}
fn fneg(interp: &mut Interpreter) -> Flow {
    let a = pop_f32(interp);
    push_f32(interp, -a);
    Flow::Continue
}

// ── double 算术 ───────────────────────────────────────────────────────

fn dadd(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    push_f64(interp, a + b);
    Flow::Continue
}
fn dsub(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    push_f64(interp, a - b);
    Flow::Continue
}
fn dmul(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    push_f64(interp, a * b);
    Flow::Continue
}
fn ddiv(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    push_f64(interp, a / b);
    Flow::Continue
}
fn drem(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    push_f64(interp, a % b);
    Flow::Continue
}
fn dneg(interp: &mut Interpreter) -> Flow {
    let a = pop_f64(interp);
    push_f64(interp, -a);
    Flow::Continue
}

// ── 类型转换 ─────────────────────────────────────────────────────────
//
// JVM 规范的浮点 → 整数转换遵循“向 0 截断”语义：NaN → 0，±Inf → ±T_MAX。
// Rust 的 `as` 转换默认就是这样，除 NaN→0 外均符合。

fn i2l(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_slot() as i64;
    interp.push_long(a);
    Flow::Continue
}
fn i2f(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_slot() as f32;
    push_f32(interp, a);
    Flow::Continue
}
fn i2d(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_slot() as f64;
    push_f64(interp, a);
    Flow::Continue
}
fn l2i(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_long() as i32;
    interp.push_slot(a);
    Flow::Continue
}
fn l2f(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_long() as f32;
    push_f32(interp, a);
    Flow::Continue
}
fn l2d(interp: &mut Interpreter) -> Flow {
    let a = interp.pop_long() as f64;
    push_f64(interp, a);
    Flow::Continue
}

// f2i / f2l 需要手动处理 NaN，避免 Rust `as` 可能的 UB。
fn f2i(interp: &mut Interpreter) -> Flow {
    let a = pop_f32(interp);
    let v = if a.is_nan() { 0 } else { a as i32 };
    interp.push_slot(v);
    Flow::Continue
}
fn f2l(interp: &mut Interpreter) -> Flow {
    let a = pop_f32(interp);
    let v: i64 = if a.is_nan() { 0 } else { a as i64 };
    interp.push_long(v);
    Flow::Continue
}
fn f2d(interp: &mut Interpreter) -> Flow {
    let a = pop_f32(interp) as f64;
    push_f64(interp, a);
    Flow::Continue
}

fn d2i(interp: &mut Interpreter) -> Flow {
    let a = pop_f64(interp);
    let v = if a.is_nan() { 0 } else { a as i32 };
    interp.push_slot(v);
    Flow::Continue
}
fn d2l(interp: &mut Interpreter) -> Flow {
    let a = pop_f64(interp);
    let v: i64 = if a.is_nan() { 0 } else { a as i64 };
    interp.push_long(v);
    Flow::Continue
}
fn d2f(interp: &mut Interpreter) -> Flow {
    let a = pop_f64(interp) as f32;
    push_f32(interp, a);
    Flow::Continue
}

// ── 比较 ─────────────────────────────────────────────────────────────

fn lcmp(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_long();
    let a = interp.pop_long();
    let r = if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    };
    interp.push_slot(r);
    Flow::Continue
}

/// fcmpl：遇 NaN 返回 -1。
fn fcmpl(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    let r = if a.is_nan() || b.is_nan() {
        -1
    } else if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    };
    interp.push_slot(r);
    Flow::Continue
}

/// fcmpg：遇 NaN 返回 1。
fn fcmpg(interp: &mut Interpreter) -> Flow {
    let b = pop_f32(interp);
    let a = pop_f32(interp);
    let r = if a.is_nan() || b.is_nan() {
        1
    } else if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    };
    interp.push_slot(r);
    Flow::Continue
}

fn dcmpl(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    let r = if a.is_nan() || b.is_nan() {
        -1
    } else if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    };
    interp.push_slot(r);
    Flow::Continue
}

fn dcmpg(interp: &mut Interpreter) -> Flow {
    let b = pop_f64(interp);
    let a = pop_f64(interp);
    let r = if a.is_nan() || b.is_nan() {
        1
    } else if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    };
    interp.push_slot(r);
    Flow::Continue
}

// ── 控制流 ───────────────────────────────────────────────────────────

fn ifeq(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v == 0)
}
fn ifne(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v != 0)
}
fn iflt(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v < 0)
}
fn ifge(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v >= 0)
}
fn ifgt(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v > 0)
}
fn ifle(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v <= 0)
}

fn if_icmpeq(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a == b)
}
fn if_icmpne(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a != b)
}
fn if_icmplt(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a < b)
}
fn if_icmpge(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a >= b)
}
fn if_icmpgt(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a > b)
}
fn if_icmple(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a <= b)
}

// 引用比较：本阶段引用以裸指针存储在栈槽中（占用 1 槽），直接比较位模式。
fn if_acmpeq(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a == b)
}
fn if_acmpne(interp: &mut Interpreter) -> Flow {
    let b = interp.pop_slot();
    let a = interp.pop_slot();
    interp.branch_if(a != b)
}
fn ifnull(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v == 0)
}
fn ifnonnull(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.branch_if(v != 0)
}

fn goto_(interp: &mut Interpreter) -> Flow {
    interp.goto_branch()
}

/// `goto_w`：4 字节分支偏移。
fn goto_w(interp: &mut Interpreter) -> Flow {
    let rel = interp.read_i32() as isize;
    // handler 进入时 pc = insn_start + 1；读 4 字节后 pc = insn_start + 5。
    // 目标 = insn_start + rel = (pc - 5) + rel。
    let target = (interp.regs().pc as isize - 5 + rel) as *mut u8;
    interp.regs().pc = target;
    Flow::Continue
}

/// `tableswitch`：稠密跳转表。
fn tableswitch(interp: &mut Interpreter) -> Flow {
    // 记录指令起点。handler 进入时 pc = insn_start + 1。
    let insn_start = unsafe { interp.regs().pc.sub(1) };
    interp.align_pc_to_4();

    let default_off = interp.read_i32() as isize;
    let low = interp.read_i32();
    let high = interp.read_i32();
    let index = interp.pop_slot();

    let off = if (low..=high).contains(&index) {
        // 跳过 (index - low) 个 4 字节 offset
        let skip = (index - low) as usize;
        for _ in 0..skip {
            let _ = interp.read_i32();
        }
        interp.read_i32() as isize
    } else {
        default_off
    };

    interp.regs().pc = unsafe { insn_start.add(off as usize) };
    Flow::Continue
}

/// `lookupswitch`：稀疏跳转表。
fn lookupswitch(interp: &mut Interpreter) -> Flow {
    let insn_start = unsafe { interp.regs().pc.sub(1) };
    interp.align_pc_to_4();

    let default_off = interp.read_i32() as isize;
    let npairs = interp.read_i32() as usize;
    let key = interp.pop_slot();

    let mut off = default_off;
    for _ in 0..npairs {
        let match_val = interp.read_i32();
        let pair_off = interp.read_i32() as isize;
        if match_val == key {
            off = pair_off;
            // 仍需读余下的 pairs 吗？不需要，跳转后不再使用 pc。
            break;
        }
    }

    interp.regs().pc = unsafe { insn_start.add(off as usize) };
    Flow::Continue
}

// ── float / double 返回 handler ───────────────────────────────────────

fn freturn(interp: &mut Interpreter) -> Flow {
    let v = pop_f32(interp);
    Flow::Return(Some(ReturnValue::Float(v)))
}

fn dreturn(interp: &mut Interpreter) -> Flow {
    let v = pop_f64(interp);
    Flow::Return(Some(ReturnValue::Double(v)))
}

// ── 栈操作 handler ────────────────────────────────────────────────────

/// `pop`：弹出 1 个栈槽（cat-1 类型）。
fn pop(interp: &mut Interpreter) -> Flow {
    interp.pop_slot();
    Flow::Continue
}

/// `pop2`：弹出 2 个栈槽（cat-2 类型，或两个 cat-1）。
fn pop2(interp: &mut Interpreter) -> Flow {
    interp.pop_slot();
    interp.pop_slot();
    Flow::Continue
}

/// `dup`：复制栈顶 1 个槽。
fn dup(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    interp.push_slot(v);
    interp.push_slot(v);
    Flow::Continue
}

/// `dup_x1`：复制栈顶 1 个槽，插入到栈顶第二个之下。
/// 栈：..., v2, v1 → ..., v1, v2, v1
fn dup_x1(interp: &mut Interpreter) -> Flow {
    let v1 = interp.pop_slot();
    let v2 = interp.pop_slot();
    interp.push_slot(v1);
    interp.push_slot(v2);
    interp.push_slot(v1);
    Flow::Continue
}

/// `dup_x2`：复制栈顶 1 个槽，插入到栈顶第三个之下（cat-1 形式）。
/// 栈：..., v3, v2, v1 → ..., v1, v3, v2, v1
fn dup_x2(interp: &mut Interpreter) -> Flow {
    let v1 = interp.pop_slot();
    let v2 = interp.pop_slot();
    let v3 = interp.pop_slot();
    interp.push_slot(v1);
    interp.push_slot(v3);
    interp.push_slot(v2);
    interp.push_slot(v1);
    Flow::Continue
}

/// `dup2`：复制栈顶 2 个槽（cat-2 形式，或两个 cat-1）。
/// 栈：..., v2, v1 → ..., v2, v1, v2, v1
fn dup2(interp: &mut Interpreter) -> Flow {
    let v1 = interp.pop_slot();
    let v2 = interp.pop_slot();
    interp.push_slot(v2);
    interp.push_slot(v1);
    interp.push_slot(v2);
    interp.push_slot(v1);
    Flow::Continue
}

/// `dup2_x1`：复制栈顶 2 个槽，插入到栈顶第二个之下。
/// 栈：..., v2, v1 → ..., v1, v2, v1
fn dup2_x1(interp: &mut Interpreter) -> Flow {
    let v1 = interp.pop_slot();
    let v2 = interp.pop_slot();
    interp.push_slot(v1);
    interp.push_slot(v2);
    interp.push_slot(v1);
    Flow::Continue
}

/// `dup2_x2`：复制栈顶 2 个槽，插入到栈顶第四个之下。
/// 栈：..., v4, v3, v2, v1 → ..., v2, v1, v4, v3, v2, v1
fn dup2_x2(interp: &mut Interpreter) -> Flow {
    let v1 = interp.pop_slot();
    let v2 = interp.pop_slot();
    let v3 = interp.pop_slot();
    let v4 = interp.pop_slot();
    interp.push_slot(v2);
    interp.push_slot(v1);
    interp.push_slot(v4);
    interp.push_slot(v3);
    interp.push_slot(v2);
    interp.push_slot(v1);
    Flow::Continue
}

/// `swap`：交换栈顶两个槽。
/// 栈：..., v2, v1 → ..., v1, v2
fn swap(interp: &mut Interpreter) -> Flow {
    let v1 = interp.pop_slot();
    let v2 = interp.pop_slot();
    interp.push_slot(v1);
    interp.push_slot(v2);
    Flow::Continue
}
