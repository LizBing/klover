//! 字节码指令分发表。
//!
//! 约定：每个 handler 接收 `&mut Interpreter`，负责自身操作数的解码（通过
//! `interp.read_u8/read_u16/...`），返回 [`Flow`] 控制流。
//!
//! 阶段 (a) 仅实现纯算术静态方法需要的指令；其余槽位为 `nop` 占位。

#![allow(clippy::erasing_op)]

use crate::class_loader::resolve::{resolve_class_ref, resolve_method_ref};
use crate::gc_bindings::{
    gc_bindings::alloc_object,
    oop_codec::{decode_oop, encode_oop, klass_from_markword},
};
use crate::interpreter::interpreter::{Flow, Interpreter, InvokeOutcome, ReturnValue, StackSlot};
use crate::oops::{
    cp_entry::{CPEntry, CPRefEntry, ClassCPEntry},
    field::Field,
    klass::Klass,
    method::Method,
    normal_klass::NormalKlass,
};

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

    // ── 对象分配 ───────────────────────────────────────────────────
    t[0xbb] = new;
    t[0xbc] = newarray;
    t[0xbd] = anewarray;
    t[0xbe] = arraylength;

    // ── 方法调用 ───────────────────────────────────────────────────
    t[0xb6] = invokevirtual;
    t[0xb7] = invokespecial;
    t[0xb8] = invokestatic_handler;
    t[0xb9] = invokeinterface;

    // ── 字段访问 ───────────────────────────────────────────────────
    t[0xb2] = getstatic;
    t[0xb3] = putstatic;
    t[0xb4] = getfield;
    t[0xb5] = putfield;

    // ── 数组读写 ───────────────────────────────────────────────────
    t[0x2e] = iaload;
    t[0x2f] = laload;
    t[0x30] = faload;
    t[0x31] = daload;
    t[0x32] = aaload;
    t[0x33] = baload;
    t[0x34] = caload;
    t[0x35] = saload;
    t[0x4f] = iastore;
    t[0x50] = lastore;
    t[0x51] = fastore;
    t[0x52] = dastore;
    t[0x53] = aastore;
    t[0x54] = bastore;
    t[0x55] = castore;
    t[0x56] = sastore;

    // ── 类型检查 ───────────────────────────────────────────────────
    t[0xc0] = checkcast;
    t[0xc1] = instanceof;

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
        CPEntry::Integer(v) => interp.push_slot(*v),
        CPEntry::Float(v) => push_f32(interp, *v),
        _ => unimplemented!("ldc entry at {}", idx),
    }
    Flow::Continue
}

/// `ldc_w`：2 字节索引。
fn ldc_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        CPEntry::Integer(v) => interp.push_slot(*v),
        CPEntry::Float(v) => push_f32(interp, *v),
        _ => unimplemented!("ldc_w entry at {}", idx),
    }
    Flow::Continue
}

/// `ldc2_w`：2 字节索引，加载 long / double。
fn ldc2_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let cp = unsafe { (*interp.regs().klass).cp_get(idx) };
    match cp {
        CPEntry::Long(v) => interp.push_long(*v),
        CPEntry::Double(v) => push_f64(interp, *v),
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

// ── 阶段 4：对象分配 + 栈操作 handler ───────────────────────────────────

/// `new`：分配一个新对象，压入其引用（narrow ptr）。
///
/// 仅支持普通类（NormalKlass）；数组、接口在阶段 4 后续步骤实现。
fn new(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let class_entry: &ClassCPEntry = match caller.cp_get(idx) {
        CPEntry::Class(e) => e,
        _ => panic!("new: CP[{}] is not a Class", idx),
    };
    let klass_ref = resolve_class_ref(caller, class_entry).expect("new: failed to resolve class");
    let normal = klass_ref.as_normal().expect("new: not a NormalKlass");

    // 分配对象。
    let byte_size = normal.obj_layout.byte_size;
    let klass_ptr: *const Klass = &*klass_ref as *const Klass;
    let obj_ptr = alloc_object(klass_ptr, byte_size);
    let narrow = encode_oop(obj_ptr);
    interp.push_slot(narrow as i32);
    Flow::Continue
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

// ── 方法调用 handler ──────────────────────────────────────────────────
//
// invokestatic / invokespecial / invokevirtual / invokeinterface 共用以下模式：
//   1. 读 CP u16 索引
//   2. resolve 出目标方法（以及目标类）
//   3. 从栈顶收集实参（含 this）
//   4. 调 invoke_instance / invoke_static

/// 收集实例方法的实参（含 this）。
///
/// 参数从栈顶倒序 pop，结果按顺序返回（args[0] = this）。
fn collect_instance_args(interp: &mut Interpreter, method: &Method) -> Vec<StackSlot> {
    let n = Interpreter::instance_arg_slot_count(method);
    let mut args = Vec::with_capacity(n);
    for _ in 0..n {
        args.push(interp.pop_slot());
    }
    args.reverse();
    args
}

/// `invokespecial`：调用构造器 / private / super.method（不做虚派发）。
///
/// 用 `resolve_method_ref` 解析（沿继承链向上找第一个匹配），
/// 然后用解析出的“声明类”调用。
fn invokespecial(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let method_ref: &CPRefEntry<Method> = match caller.cp_get(idx) {
        CPEntry::MethodRef(e) => e,
        _ => panic!("invokespecial: CP[{}] is not a MethodRef", idx),
    };
    let method_msref =
        resolve_method_ref(caller, method_ref).expect("invokespecial: failed to resolve method");
    let method: &Method = &method_msref;

    // 目标类 = 声明方法所在的类。resolve_method_ref 不返回声明类，
    // 我们从 MethodRef 的 class_name 加载它。
    // 更简单的做法：在解析时返回 (类, 方法)，但当前接口只返回方法。
    // 这里重新走 load_class 拿声明类（已缓存，开销低）。
    let target_klass =
        crate::class_loader::resolve::load_class_by_caller(caller, method_ref.class_name().utf8())
            .expect("invokespecial: failed to load target class");
    let target_normal = target_klass
        .as_normal()
        .expect("invokespecial: target not Normal");

    let args = collect_instance_args(interp, method);
    let ret = interp
        .invoke_instance(target_normal, method, &args)
        .expect("invokespecial: invocation failed");
    handle_invoke_outcome(interp, ret)
}

/// 把方法返回值压回当前栈（void 方法不压）。
/// 处理方法调用结果。  正常返回则压栈并返回 `Continue`；
/// 异常未捕获则返回 `Throw`。
fn handle_invoke_outcome(interp: &mut Interpreter, outcome: InvokeOutcome) -> Flow {
    match outcome {
        InvokeOutcome::Returned(ret) => {
            push_return_value(interp, ret);
            Flow::Continue
        }
        InvokeOutcome::Thrown(exc_nptr) => Flow::Throw(exc_nptr),
    }
}

fn push_return_value(interp: &mut Interpreter, ret: Option<ReturnValue>) {
    match ret {
        Some(ReturnValue::Int(v)) => interp.push_slot(v),
        Some(ReturnValue::Long(v)) => interp.push_long(v),
        Some(ReturnValue::Float(v)) => push_f32(interp, v),
        Some(ReturnValue::Double(v)) => push_f64(interp, v),
        Some(ReturnValue::Ref(nptr)) => interp.push_slot(nptr as i32),
        None => {}
    }
}

// ── 字段访问 handler ──────────────────────────────────────────────────
//
// instance 字段：相对对象 markword 的偏移；通过 narrow ptr 解码访问。
// static 字段：相对 static_storage 的偏移；直接访问 metaspace 内存。
//
// 读写在 payload 字节区上按 FieldDesc 的 byte_size 进行（1/2/4/8）。
// 引用字段以 narrow ptr 形式存取（栈槽 / 字段槽都是 u32）。

/// 从 `base + offs` 读 `size` 字节，扩展为 i64 返回。
///
/// 引用字段（size == 4 且是 reference 类型）应在外层特殊处理，不走这里。
unsafe fn read_payload(base: *const u8, offs: usize, size: usize) -> i64 {
    let p = base.add(offs);
    match size {
        1 => *(p as *const i8) as i64,
        2 => (*(p as *const u16) as i32) as i64,
        4 => (*(p as *const u32) as i32) as i64,
        8 => *(p as *const i64),
        _ => unreachable!("invalid field size {}", size),
    }
}

/// 向 `base + offs` 写 `size` 字节，从 i64 截断。
unsafe fn write_payload(base: *mut u8, offs: usize, size: usize, value: i64) {
    let p = base.add(offs);
    match size {
        1 => *(p as *mut i8) = value as i8,
        2 => *(p as *mut u16) = value as u16,
        4 => *(p as *mut u32) = value as u32,
        8 => *(p as *mut i64) = value,
        _ => unreachable!("invalid field size {}", size),
    }
}

/// 字段是否是引用类型（对象 / 数组）。
fn is_reference_field(f: &Field) -> bool {
    f.desc.dimensions > 0 || matches!(f.desc.elem, crate::oops::desc::FieldElemType::Class { .. })
}

/// 获取声明类的 static_storage 起始指针。
///
/// 由于 resolve_field_ref 只返回 Field，不返回声明类，这里沿继承链
/// 查找哪个类持有该 field 的 static_storage。
fn static_storage_base(caller: &NormalKlass, field: &Field) -> *mut u8 {
    let mut cur: Option<&NormalKlass> = Some(caller);
    while let Some(k) = cur {
        let f = k.fields();
        let storage_ptr = f.static_storage.as_ref().as_ptr() as *mut u8;
        for sf in f.static_fields.as_ref() {
            if std::ptr::eq(sf as *const Field, field as *const Field) {
                return storage_ptr;
            }
        }
        cur = k.get_super();
    }
    unreachable!("static field not found in inheritance chain")
}

/// `getstatic`：读 static 字段。
fn getstatic(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let field_ref = match caller.cp_get(idx) {
        CPEntry::FieldRef(e) => e,
        _ => panic!("getstatic: CP[{}] is not a FieldRef", idx),
    };
    let field_msref = crate::class_loader::resolve::resolve_field_ref(caller, field_ref)
        .expect("getstatic: resolve failed");
    let field: &Field = &field_msref;
    assert!(
        field
            .acc_flags
            .contains(crate::oops::acc_flags::AccFlags::ACC_STATIC)
    );

    let base = static_storage_base(caller, field);
    let offs = field.offs();
    if is_reference_field(field) {
        let nptr = unsafe { read_payload(base, offs, 4) } as u32;
        interp.push_slot(nptr as i32);
    } else {
        let size = field.desc.byte_size();
        let v = unsafe { read_payload(base, offs, size) };
        if size == 8 {
            interp.push_long(v);
        } else {
            interp.push_slot(v as i32);
        }
    }
    Flow::Continue
}

/// `putstatic`：写 static 字段。
fn putstatic(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let field_ref = match caller.cp_get(idx) {
        CPEntry::FieldRef(e) => e,
        _ => panic!("putstatic: CP[{}] is not a FieldRef", idx),
    };
    let field_msref = crate::class_loader::resolve::resolve_field_ref(caller, field_ref)
        .expect("putstatic: resolve failed");
    let field: &Field = &field_msref;
    assert!(
        field
            .acc_flags
            .contains(crate::oops::acc_flags::AccFlags::ACC_STATIC)
    );

    let base = static_storage_base(caller, field);
    let offs = field.offs();
    let size = field.desc.byte_size();
    if is_reference_field(field) {
        let nptr = interp.pop_slot() as u32;
        unsafe { write_payload(base, offs, 4, nptr as i64) };
    } else if size == 8 {
        let v = interp.pop_long();
        unsafe { write_payload(base, offs, 8, v) };
    } else {
        let v = interp.pop_slot() as i64;
        unsafe { write_payload(base, offs, size, v) };
    }
    Flow::Continue
}

/// `getfield`：读 instance 字段。
fn getfield(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let field_ref = match caller.cp_get(idx) {
        CPEntry::FieldRef(e) => e,
        _ => panic!("getfield: CP[{}] is not a FieldRef", idx),
    };
    let field_msref = crate::class_loader::resolve::resolve_field_ref(caller, field_ref)
        .expect("getfield: resolve failed");
    let field: &Field = &field_msref;
    assert!(
        !field
            .acc_flags
            .contains(crate::oops::acc_flags::AccFlags::ACC_STATIC)
    );

    let this_nptr = interp.pop_slot() as u32;
    assert!(this_nptr != 0, "getfield on null");
    let obj_ptr = decode_oop(this_nptr);
    let base = obj_ptr as *const u8;
    let offs = field.offs();
    if is_reference_field(field) {
        let nptr = unsafe { read_payload(base, offs, 4) } as u32;
        interp.push_slot(nptr as i32);
    } else {
        let size = field.desc.byte_size();
        let v = unsafe { read_payload(base, offs, size) };
        if size == 8 {
            interp.push_long(v);
        } else {
            interp.push_slot(v as i32);
        }
    }
    Flow::Continue
}

/// `putfield`：写 instance 字段。
fn putfield(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let field_ref = match caller.cp_get(idx) {
        CPEntry::FieldRef(e) => e,
        _ => panic!("putfield: CP[{}] is not a FieldRef", idx),
    };
    let field_msref = crate::class_loader::resolve::resolve_field_ref(caller, field_ref)
        .expect("putfield: resolve failed");
    let field: &Field = &field_msref;
    assert!(
        !field
            .acc_flags
            .contains(crate::oops::acc_flags::AccFlags::ACC_STATIC)
    );

    let size = field.desc.byte_size();
    let offs = field.offs();
    if is_reference_field(field) {
        let v = interp.pop_slot() as i64;
        let this_nptr = interp.pop_slot() as u32;
        assert!(this_nptr != 0, "putfield on null");
        let obj_ptr = decode_oop(this_nptr);
        let base = obj_ptr as *mut u8;
        unsafe { write_payload(base, offs, 4, v) };
    } else if size == 8 {
        let v = interp.pop_long();
        let this_nptr = interp.pop_slot() as u32;
        assert!(this_nptr != 0, "putfield on null");
        let obj_ptr = decode_oop(this_nptr);
        let base = obj_ptr as *mut u8;
        unsafe { write_payload(base, offs, 8, v) };
    } else {
        let v = interp.pop_slot() as i64;
        let this_nptr = interp.pop_slot() as u32;
        assert!(this_nptr != 0, "putfield on null");
        let obj_ptr = decode_oop(this_nptr);
        let base = obj_ptr as *mut u8;
        unsafe { write_payload(base, offs, size, v) };
    }
    Flow::Continue
}

// ── invokestatic / invokevirtual handler ────────────────────────────────

/// `invokestatic`：调用静态方法（从字节码 CP 索引触发）。
fn invokestatic_handler(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let method_ref = match caller.cp_get(idx) {
        CPEntry::MethodRef(e) => e,
        _ => panic!("invokestatic: CP[{}] is not a MethodRef", idx),
    };
    let method_msref =
        resolve_method_ref(caller, method_ref).expect("invokestatic: resolve failed");
    let method: &Method = &method_msref;

    // static 方法的实参不含 this。
    let n = Interpreter::static_arg_slot_count(method);
    let mut args = Vec::with_capacity(n);
    for _ in 0..n {
        args.push(interp.pop_slot());
    }
    args.reverse();

    // 目标类 = 声明方法所在的类。
    let target_klass =
        crate::class_loader::resolve::load_class_by_caller(caller, method_ref.class_name().utf8())
            .expect("invokestatic: load target class failed");
    let target_normal = target_klass
        .as_normal()
        .expect("invokestatic: target not Normal");

    let ret = interp
        .invoke_static(target_normal, method, &args)
        .expect("invokestatic: invocation failed");
    handle_invoke_outcome(interp, ret)
}

/// `invokevirtual`：虚方法派发。
///
/// 根据 this 的运行时类型查找方法（线性查找，沿继承链向上）。
fn invokevirtual(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let method_ref = match caller.cp_get(idx) {
        CPEntry::MethodRef(e) => e,
        _ => panic!("invokevirtual: CP[{}] is not a MethodRef", idx),
    };

    // 解析出 name+desc 和实参 slot 数。
    let resolved_msref =
        resolve_method_ref(caller, method_ref).expect("invokevirtual: resolve failed");
    let resolved: &Method = &resolved_msref;
    let mname = resolved.name.clone();
    let mdesc = resolved.desc.raw.clone();
    let n = Interpreter::instance_arg_slot_count(resolved);

    invoke_dispatched(interp, &mname, &mdesc, n)
}

/// `invokeinterface`：接口方法派发。
///
/// 与 invokevirtual 类似，但 CP 条目是 InterfaceMethodRef，
/// 且字节码多一个 count 字节（历史遗留，读取后丢弃）。
///
/// MVP 限制：不支持 default method（需要遍历 implements 列表）。
fn invokeinterface(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let _count = interp.read_u8(); // 历史遗留的 arg slot 数，MVP 不用
    let caller = unsafe { &*interp.regs().klass };
    let method_ref = match caller.cp_get(idx) {
        CPEntry::InterfaceMethodRef(e) => e,
        CPEntry::MethodRef(e) => e, // 宽容：有些编译器对接口方法也用 MethodRef
        _ => panic!(
            "invokeinterface: CP[{}] is not a MethodRef/InterfaceMethodRef",
            idx
        ),
    };

    let resolved_msref =
        resolve_method_ref(caller, method_ref).expect("invokeinterface: resolve failed");
    let resolved: &Method = &resolved_msref;
    let mname = resolved.name.clone();
    let mdesc = resolved.desc.raw.clone();
    let n = Interpreter::instance_arg_slot_count(resolved);

    invoke_dispatched(interp, &mname, &mdesc, n)
}

/// invokevirtual / invokeinterface 共用的虚派发核心。
///
/// 栈布局（顶在上）：..., argN, ..., arg1, this
/// `n` 是实参总 slot 数（含 this）。
///
/// 从 this 对应的运行时 klass 开始线性查找 name+desc。
// TODO: default method 需要遍历 implements 列表。
fn invoke_dispatched(
    interp: &mut Interpreter,
    mname: &crate::oops::symbol_table::SymbolHandle,
    mdesc: &crate::oops::symbol_table::SymbolHandle,
    n: usize,
) -> Flow {
    // 收集实参（含 this）。栈顶是最后一个参数，下面是 this。
    let mut args = Vec::with_capacity(n);
    for _ in 0..n {
        args.push(interp.pop_slot());
    }
    args.reverse();
    let this_nptr = args[0] as u32;
    assert!(this_nptr != 0, "invoke on null");

    // 从对象 markword 解出运行时 klass。
    let obj_ptr = decode_oop(this_nptr);
    let markword = unsafe { (*obj_ptr).markword };
    let klass_ref = unsafe { klass_from_markword(markword) };
    let runtime_klass: &NormalKlass = match &*klass_ref {
        Klass::Normal(n) => n,
        _ => panic!("invoke: object klass is not Normal"),
    };

    // 在运行时 klass 上线性查找方法。
    let mut cur: Option<&NormalKlass> = Some(runtime_klass);
    let target: &Method = loop {
        if let Some(k) = cur {
            if let Some(m) = k.find_method(mname, mdesc) {
                break m;
            }
            cur = k.get_super();
        } else {
            panic!("invoke: method not found in runtime type");
        }
    };

    let ret = interp
        .invoke_instance(runtime_klass, target, &args)
        .expect("invoke: invocation failed");
    handle_invoke_outcome(interp, ret)
}

// ── 数组指令 handler ──────────────────────────────────────────────────
//
// 数组对象布局：markword(8) + length(4) + padding(4) + elements...
// 元素起始固定 ARRAY_DATA_OFFSET = 16。

use crate::class_loader::{bootstrap_cld::BootstrapCLD, resolve::load_class_by_caller};
use crate::oops::array_klass::{ARRAY_DATA_OFFSET, ARRAY_LENGTH_OFFSET};

/// 分配数组对象。  返回 narrow ptr。
///
/// `klass_ref` 必须是 ArrayKlass。`count` 必须 >= 0。
fn alloc_array(klass_ref: &crate::class_loader::ms_api::MSRef<Klass>, count: i32) -> u32 {
    let array_klass = klass_ref
        .as_array()
        .expect("alloc_array: not an ArrayKlass");
    let elem_size = array_klass.element_size();
    let byte_size = crate::oops::array_klass::ARRAY_HEADER_BYTES + ((count as usize) * elem_size);
    // 对齐到 8 字节。
    let byte_size = (byte_size + 7) & !7;
    let klass_ptr: *const Klass = klass_ref as *const _ as *const Klass;
    // 上面取引用的指针不安全；改用 deref 拿到内部 NonNull。
    // 这里用另一个办法：从 MSRef 的 deref 取地址。
    let obj_ptr = {
        // 先拿到内部 raw 指针。
        let k: &Klass = klass_ref;
        alloc_object(k as *const Klass, byte_size)
    };
    // 写 length。
    unsafe {
        let length_ptr = (obj_ptr as *mut u8).add(ARRAY_LENGTH_OFFSET) as *mut i32;
        *length_ptr = count;
    }
    encode_oop(obj_ptr)
}

/// 读数组长度字段。
unsafe fn read_array_length(obj_ptr: crate::oops::oop_handle::ObjPtr) -> i32 {
    let p = (obj_ptr as *const u8).add(ARRAY_LENGTH_OFFSET) as *const i32;
    unsafe { *p }
}

/// `newarray`：创建基本类型数组。
fn newarray(interp: &mut Interpreter) -> Flow {
    let atype = interp.read_u8();
    let name = match atype {
        4 => "[Z",
        5 => "[C",
        6 => "[F",
        7 => "[D",
        8 => "[B",
        9 => "[S",
        10 => "[I",
        11 => "[J",
        _ => panic!("newarray: invalid atype {}", atype),
    };
    let count = interp.pop_slot();
    assert!(count >= 0, "newarray: negative array size {}", count);

    let caller = unsafe { &*interp.regs().klass };
    let klass_ref = match caller.cld {
        Some(cld) => {
            unsafe { (*cld.as_ptr()).load_class(name) }.expect("newarray: load array klass")
        }
        None => BootstrapCLD::find_class(name).expect("newarray: load array klass"),
    };
    let nptr = alloc_array(&klass_ref, count);
    interp.push_slot(nptr as i32);
    Flow::Continue
}

/// `anewarray`：创建引用类型数组。
fn anewarray(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let count = interp.pop_slot();
    assert!(count >= 0, "anewarray: negative array size {}", count);

    let caller = unsafe { &*interp.regs().klass };
    // 先解析元素类。
    let elem_entry = match caller.cp_get(idx) {
        CPEntry::Class(e) => e,
        _ => panic!("anewarray: CP[{}] is not a Class", idx),
    };
    let _elem_klass = resolve_class_ref(caller, elem_entry).expect("anewarray: resolve elem class");

    // 数组内部名 = [L + 元素名 + ;
    let elem_name = elem_entry.name.utf8();
    // 元素名本身已经是内部形式（例如 java/lang/String 或 [I）。
    // 对于引用类型元素，数组名 = "[" + 元素描述符。
    // 元素描述符：类是 L...;，数组是本身。
    let array_name = if elem_name.starts_with('[') {
        format!("[{}", elem_name)
    } else {
        format!("[L{};", elem_name)
    };

    let klass_ref = match caller.cld {
        Some(cld) => unsafe { (*cld.as_ptr()).load_class(&array_name) }.expect("anewarray: load"),
        None => BootstrapCLD::find_class(&array_name).expect("anewarray: load"),
    };
    let nptr = alloc_array(&klass_ref, count);
    interp.push_slot(nptr as i32);
    Flow::Continue
}

/// `arraylength`：读数组长度。
fn arraylength(interp: &mut Interpreter) -> Flow {
    let nptr = interp.pop_slot() as u32;
    assert!(nptr != 0, "arraylength on null");
    let obj_ptr = decode_oop(nptr);
    let len = unsafe { read_array_length(obj_ptr) };
    interp.push_slot(len);
    Flow::Continue
}

// ── *aload / *astore helper ──────────────────────────────────────────
//
// 所有 8 种 aload 共用逻辑，只是元素大小和符号/零扩展不同。
// 所有 8 种 astore 共用逻辑，只是元素大小不同。
// 用具体的 handler 函数（而非完全参数化）来处理扩展语义。

/// 从栈顶弹出 index 和 array ref，返回 (obj_ptr, index, length)。
fn pop_array_index(interp: &mut Interpreter) -> (crate::oops::oop_handle::ObjPtr, i32, i32) {
    let index = interp.pop_slot();
    let nptr = interp.pop_slot() as u32;
    assert!(nptr != 0, "array access on null");
    let obj_ptr = decode_oop(nptr);
    let length = unsafe { read_array_length(obj_ptr) };
    // TODO 阶段 5：ArrayIndexOutOfBoundsException
    assert!(
        index >= 0 && index < length,
        "array index out of bounds: index={}, length={}",
        index,
        length
    );
    (obj_ptr, index, length)
}

/// 从 obj_ptr 读元素起始地址。
unsafe fn element_addr(
    obj_ptr: crate::oops::oop_handle::ObjPtr,
    index: i32,
    elem_size: usize,
) -> *mut u8 {
    unsafe { (obj_ptr as *mut u8).add(ARRAY_DATA_OFFSET + (index as usize) * elem_size) }
}

// ── iaload..saload ──────────────────────────────────────────────────
// int/float/引用元素占 1 槽，long/double 占 2 槽。
// byte/char/short 需要扩展到 int。

fn iaload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 4) } as *const i32;
    interp.push_slot(unsafe { *p });
    Flow::Continue
}

fn laload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 8) } as *const i64;
    interp.push_long(unsafe { *p });
    Flow::Continue
}

fn faload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 4) } as *const u32;
    push_f32(interp, f32::from_bits(unsafe { *p }));
    Flow::Continue
}

fn daload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 8) } as *const u64;
    push_f64(interp, f64::from_bits(unsafe { *p }));
    Flow::Continue
}

/// aaload：引用数组。元素是 narrow ptr，占 1 槽。
fn aaload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 4) } as *const u32;
    interp.push_slot(unsafe { *p } as i32);
    Flow::Continue
}

/// baload：byte 数组（有符号扩展）。
fn baload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 1) } as *const i8;
    interp.push_slot(unsafe { *p } as i32);
    Flow::Continue
}

/// caload：char 数组（零扩展）。
fn caload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 2) } as *const u16;
    interp.push_slot(unsafe { *p } as i32);
    Flow::Continue
}

/// saload：short 数组（有符号扩展）。
fn saload(interp: &mut Interpreter) -> Flow {
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 2) } as *const i16;
    interp.push_slot(unsafe { *p } as i32);
    Flow::Continue
}

// ── iastore..sastore ──────────────────────────────────────────────────
// 写入时按窄类型截断。

fn iastore(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot();
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 4) } as *mut i32;
    unsafe { *p = v };
    Flow::Continue
}

fn lastore(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_long();
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 8) } as *mut i64;
    unsafe { *p = v };
    Flow::Continue
}

fn fastore(interp: &mut Interpreter) -> Flow {
    let v = pop_f32(interp);
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 4) } as *mut u32;
    unsafe { *p = v.to_bits() };
    Flow::Continue
}

fn dastore(interp: &mut Interpreter) -> Flow {
    let v = pop_f64(interp);
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 8) } as *mut u64;
    unsafe { *p = v.to_bits() };
    Flow::Continue
}

fn aastore(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot() as u32;
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 4) } as *mut u32;
    unsafe { *p = v };
    Flow::Continue
}

fn bastore(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot() as i8;
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 1) } as *mut i8;
    unsafe { *p = v };
    Flow::Continue
}

fn castore(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot() as u16;
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 2) } as *mut u16;
    unsafe { *p = v };
    Flow::Continue
}

fn sastore(interp: &mut Interpreter) -> Flow {
    let v = interp.pop_slot() as i16;
    let (obj_ptr, index, _) = pop_array_index(interp);
    let p = unsafe { element_addr(obj_ptr, index, 2) } as *mut i16;
    unsafe { *p = v };
    Flow::Continue
}

// ── 类型检查 handler ──────────────────────────────────────────────────
//
// instanceof / checkcast 共用类型判断逻辑：
//   1. 解析 CP ClassRef → 目标 Klass
//   2. 解对象的运行时 klass
//   3. 判断运行时 klass 是否是目标类的子类（仅普通类，接口留 TODO）
//
// null 处理：instanceof 返回 false，checkcast 通过。

/// 判断对象是否是目标类型的实例（仅普通类）。
///
/// `obj_nptr == 0` 表示 null，返回 false。
// TODO: 接口类型判断（遍历 implements 列表）
fn is_instance_of(obj_nptr: u32, target_klass: &crate::class_loader::ms_api::MSRef<Klass>) -> bool {
    if obj_nptr == 0 {
        return false;
    }
    let obj_ptr = decode_oop(obj_nptr);
    let markword = unsafe { (*obj_ptr).markword };
    let obj_klass_ref = unsafe { klass_from_markword(markword) };
    let obj_normal = match &*obj_klass_ref {
        Klass::Normal(n) => n,
        _ => return false, // 数组对象的 instanceof 留 TODO
    };
    let target_normal = match target_klass.as_normal() {
        Some(n) => n,
        None => return false, // 目标是接口/数组，MVP 不支持
    };
    obj_normal.is_subclass_of(target_normal)
}

/// `instanceof`：弹对象，push int（1=true, 0=false）。
fn instanceof(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let class_entry = match caller.cp_get(idx) {
        CPEntry::Class(e) => e,
        _ => panic!("instanceof: CP[{}] is not a Class", idx),
    };
    let target_klass = resolve_class_ref(caller, class_entry).expect("instanceof: resolve failed");

    let obj_nptr = interp.pop_slot() as u32;
    let result = if is_instance_of(obj_nptr, &target_klass) {
        1
    } else {
        0
    };
    interp.push_slot(result);
    Flow::Continue
}

/// `checkcast`：不修改栈。  失败时 panic（TODO: ClassCastException）。
fn checkcast(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    let caller = unsafe { &*interp.regs().klass };
    let class_entry = match caller.cp_get(idx) {
        CPEntry::Class(e) => e,
        _ => panic!("checkcast: CP[{}] is not a Class", idx),
    };
    let target_klass = resolve_class_ref(caller, class_entry).expect("checkcast: resolve failed");

    let obj_nptr = interp.peek_slot() as u32;
    if obj_nptr == 0 {
        // null 可以 cast 到任何引用类型。
        return Flow::Continue;
    }
    // TODO 阶段 5：ClassCastException
    assert!(
        is_instance_of(obj_nptr, &target_klass),
        "checkcast: object is not an instance of target class"
    );
    Flow::Continue
}
