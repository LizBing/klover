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

// ── 栈 / 局部变量 操作原语 ────────────────────────────────────────────

/// 单槽（int / float / ref）入栈。
#[inline]
fn push_slot(regs: &mut Registers, v: StackSlot) {
    unsafe {
        regs.sp = regs.sp.sub(1);
        *regs.sp = v;
    }
}

/// 单槽出栈。
#[inline]
fn pop_slot(regs: &mut Registers) -> StackSlot {
    unsafe {
        let v = *regs.sp;
        regs.sp = regs.sp.add(1);
        v
    }
}

/// 双槽（long / double）入栈：高 32 位在低地址。
#[inline]
fn push_wide(regs: &mut Registers, v: DStackSlot) {
    let bits = v as u64;
    let lo = bits as u32 as StackSlot;
    let hi = (bits >> 32) as u32 as StackSlot;
    unsafe {
        regs.sp = regs.sp.sub(2);
        *regs.sp = hi;
        *regs.sp.add(1) = lo;
    }
}

/// 双槽出栈。
#[inline]
fn pop_wide(regs: &mut Registers) -> DStackSlot {
    unsafe {
        let hi = *regs.sp as u32;
        let lo = *regs.sp.add(1) as u32;
        regs.sp = regs.sp.add(2);
        (((hi as u64) << 32) | (lo as u64)) as i64
    }
}

/// 读取局部变量 `idx` 处的单槽。
#[inline]
fn get_local(regs: &mut Registers, idx: usize) -> StackSlot {
    unsafe { *regs.bp.add(idx) }
}

/// 写入局部变量 `idx`。
#[inline]
fn set_local(regs: &mut Registers, idx: usize, v: StackSlot) {
    unsafe {
        *regs.bp.add(idx) = v;
    }
}

/// 读取局部变量 `idx`、`idx+1` 处的双槽（高 32 位在 idx）。
#[inline]
fn get_local_wide(regs: &mut Registers, idx: usize) -> DStackSlot {
    let hi = unsafe { *regs.bp.add(idx) } as u32;
    let lo = unsafe { *regs.bp.add(idx + 1) } as u32;
    (((hi as u64) << 32) | lo as u64) as i64
}

/// 写入局部变量 `idx`、`idx+1` 处的双槽。
#[inline]
fn set_local_wide(regs: &mut Registers, idx: usize, v: DStackSlot) {
    let bits = v as u64;
    let hi = (bits >> 32) as u32 as StackSlot;
    let lo = bits as u32 as StackSlot;
    unsafe {
        *regs.bp.add(idx) = hi;
        *regs.bp.add(idx + 1) = lo;
    }
}

// ── 通用 handler 模板 ────────────────────────────────────────────────

fn nop(_: &mut Interpreter) -> Flow {
    Flow::Continue
}

/// `iconst_<i>`：把常量 i 入栈。
fn iconst<const N: i32>(interp: &mut Interpreter) -> Flow {
    push_slot(interp.regs(), N);
    Flow::Continue
}

/// `bipush <byte>`：1 字节有符号立即数入栈。
fn bipush(interp: &mut Interpreter) -> Flow {
    let v = interp.read_i8() as i32;
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `sipush <short>`：2 字节有符号立即数入栈。
fn sipush(interp: &mut Interpreter) -> Flow {
    let v = interp.read_i16() as i32;
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `iload <index>`：从局部变量 index 取 int 入栈。
fn iload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let v = get_local(interp.regs(), idx);
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `iload_<n>`：从局部变量 n 取 int 入栈。
fn iload_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = get_local(interp.regs(), N);
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `istore <index>`：栈顶 int 存入局部变量 index。
fn istore(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let v = pop_slot(interp.regs());
    set_local(interp.regs(), idx, v);
    Flow::Continue
}

/// `istore_<n>`：栈顶 int 存入局部变量 n。
fn istore_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    set_local(interp.regs(), N, v);
    Flow::Continue
}

/// 二元 int 运算通用模板：取两个 int（注意 JVM 是"先弹右操作数，再弹左"）。
fn ibinop<F: Fn(i32, i32) -> i32>(interp: &mut Interpreter, f: F) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    push_slot(interp.regs(), f(a, b));
    Flow::Continue
}

fn iadd(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| a.wrapping_add(b))
}
fn isub(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| a.wrapping_sub(b))
}
fn imul(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| a.wrapping_mul(b))
}
fn idiv(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| {
        if b == 0 {
            // TODO 阶段 5：ArithmeticException；当前以 panic 暴露问题。
            panic!("idiv: division by zero");
        }
        // JVM 整除向零取整；Rust 的 / 已是向零取整。
        a / b
    })
}
fn irem(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| {
        if b == 0 {
            panic!("irem: division by zero");
        }
        a % b
    })
}
fn ineg(interp: &mut Interpreter) -> Flow {
    let a = pop_slot(interp.regs());
    push_slot(interp.regs(), a.wrapping_neg());
    Flow::Continue
}

/// `iinc <index> <const>`：locals[index] += (i8)const，不改变栈。
fn iinc(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let konst = interp.read_i8() as i32;
    let cur = get_local(interp.regs(), idx);
    set_local(interp.regs(), idx, cur.wrapping_add(konst));
    Flow::Continue
}

/// `ireturn`：返回栈顶 int。
fn ireturn(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    Flow::Return(Some(ReturnValue::Int(v)))
}

/// `return`：void 返回。
fn vreturn(_: &mut Interpreter) -> Flow {
    Flow::Return(None)
}

// ── 宽类型常量加载 ────────────────────────────────────────────

/// `lconst_0` / `lconst_1`：将 long 常量 N 入栈（2 槽）。
fn lconst<const N: i64>(interp: &mut Interpreter) -> Flow {
    push_wide(interp.regs(), N);
    Flow::Continue
}

/// `fconst_<n>`：将 float 常量 N 入栈（1 槽，但语义上是 f32）。
fn fconst_n<const BITS: u32>(interp: &mut Interpreter) -> Flow {
    push_slot(interp.regs(), BITS as i32);
    Flow::Continue
}

fn fconst_0(interp: &mut Interpreter) -> Flow {
    fconst_n::<{ 0.0f32.to_bits() }>(interp)
}
fn fconst_1(interp: &mut Interpreter) -> Flow {
    fconst_n::<{ 1.0f32.to_bits() }>(interp)
}
fn fconst_2(interp: &mut Interpreter) -> Flow {
    fconst_n::<{ 2.0f32.to_bits() }>(interp)
}

/// `dconst_<n>`：将 double 常量 N 入栈（2 槽）。
fn dconst_n<const BITS: u64>(interp: &mut Interpreter) -> Flow {
    push_wide(interp.regs(), BITS as i64);
    Flow::Continue
}

fn dconst_0(interp: &mut Interpreter) -> Flow {
    dconst_n::<{ 0.0f64.to_bits() }>(interp)
}
fn dconst_1(interp: &mut Interpreter) -> Flow {
    dconst_n::<{ 1.0f64.to_bits() }>(interp)
}

/// `ldc`：1 字节 CP 索引。仅支持 int / float。
fn ldc(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    use crate::oops::cp_entry::CPEntry;
    let bits = match interp.regs().cp_get(idx) {
        CPEntry::Integer { value } => *value,
        CPEntry::Float { value } => value.to_bits() as i32,
        _ => panic!("ldc: unsupported CP entry at index {}", idx),
    };
    push_slot(interp.regs(), bits);
    Flow::Continue
}

/// `ldc_w`：2 字节 CP 索引。仅支持 int / float。
fn ldc_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    use crate::oops::cp_entry::CPEntry;
    let bits = match interp.regs().cp_get(idx) {
        CPEntry::Integer { value } => *value,
        CPEntry::Float { value } => value.to_bits() as i32,
        _ => panic!("ldc_w: unsupported CP entry at index {}", idx),
    };
    push_slot(interp.regs(), bits);
    Flow::Continue
}

/// `ldc2_w`：2 字节 CP 索引。仅支持 long / double。
fn ldc2_w(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u16() as usize;
    use crate::oops::cp_entry::CPEntry;
    let bits = match interp.regs().cp_get(idx) {
        CPEntry::Long { value } => *value as u64,
        CPEntry::Double { value } => value.to_bits(),
        _ => panic!("ldc2_w: unsupported CP entry at index {}", idx),
    };
    push_wide(interp.regs(), bits as i64);
    Flow::Continue
}

// ── 宽类型 load / store ──────────────────────────────────────

/// `lload <index>`：从局部变量 index、index+1 取 long 入栈。
fn lload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let v = get_local_wide(interp.regs(), idx);
    push_wide(interp.regs(), v);
    Flow::Continue
}

fn lload_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = get_local_wide(interp.regs(), N);
    push_wide(interp.regs(), v);
    Flow::Continue
}

/// `fload <index>`：从局部变量 index 取 float 入栈（1 槽）。
fn fload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let v = get_local(interp.regs(), idx);
    push_slot(interp.regs(), v);
    Flow::Continue
}

fn fload_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = get_local(interp.regs(), N);
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `dload <index>`：从局部变量 index、index+1 取 double 入栈。
fn dload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let v = get_local_wide(interp.regs(), idx);
    push_wide(interp.regs(), v);
    Flow::Continue
}

fn dload_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = get_local_wide(interp.regs(), N);
    push_wide(interp.regs(), v);
    Flow::Continue
}

/// `aload <index>`：对象引用载入（1 槽，语义同 iload）。
fn aload(interp: &mut Interpreter) -> Flow {
    let idx = interp.read_u8() as usize;
    let v = get_local(interp.regs(), idx);
    push_slot(interp.regs(), v);
    Flow::Continue
}

fn aload_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = get_local(interp.regs(), N);
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `lstore <index>`：栈顶 long 存入 index、index+1。
fn lstore(interp: &mut Interpreter) -> Flow {
    let v = pop_wide(interp.regs());
    let idx = interp.read_u8() as usize;
    set_local_wide(interp.regs(), idx, v);
    Flow::Continue
}

fn lstore_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = pop_wide(interp.regs());
    set_local_wide(interp.regs(), N, v);
    Flow::Continue
}

/// `fstore <index>`：栈顶 float 存入 index（1 槽）。
fn fstore(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    let idx = interp.read_u8() as usize;
    set_local(interp.regs(), idx, v);
    Flow::Continue
}

fn fstore_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    set_local(interp.regs(), N, v);
    Flow::Continue
}

/// `dstore <index>`：栈顶 double 存入 index、index+1。
fn dstore(interp: &mut Interpreter) -> Flow {
    let v = pop_wide(interp.regs());
    let idx = interp.read_u8() as usize;
    set_local_wide(interp.regs(), idx, v);
    Flow::Continue
}

fn dstore_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = pop_wide(interp.regs());
    set_local_wide(interp.regs(), N, v);
    Flow::Continue
}

/// `astore <index>`：栈顶引用存入 index（1 槽，语义同 istore）。
fn astore(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    let idx = interp.read_u8() as usize;
    set_local(interp.regs(), idx, v);
    Flow::Continue
}

fn astore_n<const N: usize>(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    set_local(interp.regs(), N, v);
    Flow::Continue
}

// ── 宽类型返回 ────────────────────────────────────────────────

fn lreturn(interp: &mut Interpreter) -> Flow {
    let v = pop_wide(interp.regs());
    Flow::Return(Some(ReturnValue::Long(v)))
}

fn freturn(interp: &mut Interpreter) -> Flow {
    let bits = pop_slot(interp.regs());
    Flow::Return(Some(ReturnValue::Float(f32::from_bits(bits as u32))))
}

fn dreturn(interp: &mut Interpreter) -> Flow {
    let bits = pop_wide(interp.regs());
    Flow::Return(Some(ReturnValue::Double(f64::from_bits(bits as u64))))
}

// ── long 算术 ────────────────────────────────────────────────

fn lbinop<F: Fn(i64, i64) -> i64>(interp: &mut Interpreter, f: F) -> Flow {
    let b = pop_wide(interp.regs());
    let a = pop_wide(interp.regs());
    push_wide(interp.regs(), f(a, b));
    Flow::Continue
}

fn ladd(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| a.wrapping_add(b))
}
fn lsub(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| a.wrapping_sub(b))
}
fn lmul(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| a.wrapping_mul(b))
}
fn ldiv(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| {
        if b == 0 {
            panic!("ldiv: division by zero");
        }
        a / b
    })
}
fn lrem(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| {
        if b == 0 {
            panic!("lrem: division by zero");
        }
        a % b
    })
}
fn lneg(interp: &mut Interpreter) -> Flow {
    let a = pop_wide(interp.regs());
    push_wide(interp.regs(), a.wrapping_neg());
    Flow::Continue
}

// ── float 算术 ────────────────────────────────────────────────

fn fbinop<F: Fn(f32, f32) -> f32>(interp: &mut Interpreter, f: F) -> Flow {
    let b = f32::from_bits(pop_slot(interp.regs()) as u32);
    let a = f32::from_bits(pop_slot(interp.regs()) as u32);
    push_slot(interp.regs(), f(a, b).to_bits() as i32);
    Flow::Continue
}

fn fadd(interp: &mut Interpreter) -> Flow {
    fbinop(interp, |a, b| a + b)
}
fn fsub(interp: &mut Interpreter) -> Flow {
    fbinop(interp, |a, b| a - b)
}
fn fmul(interp: &mut Interpreter) -> Flow {
    fbinop(interp, |a, b| a * b)
}
fn fdiv(interp: &mut Interpreter) -> Flow {
    fbinop(interp, |a, b| a / b)
}
fn frem(interp: &mut Interpreter) -> Flow {
    // JVM float % 语义：与被除数同号，|r| < |b|。Rust 的 % 符合。
    fbinop(interp, |a, b| a % b)
}
fn fneg(interp: &mut Interpreter) -> Flow {
    let a = f32::from_bits(pop_slot(interp.regs()) as u32);
    push_slot(interp.regs(), (-a).to_bits() as i32);
    Flow::Continue
}

// ── double 算术 ────────────────────────────────────────────────

fn dbinop<F: Fn(f64, f64) -> f64>(interp: &mut Interpreter, f: F) -> Flow {
    let b = f64::from_bits(pop_wide(interp.regs()) as u64);
    let a = f64::from_bits(pop_wide(interp.regs()) as u64);
    push_wide(interp.regs(), f(a, b).to_bits() as i64);
    Flow::Continue
}

fn dadd(interp: &mut Interpreter) -> Flow {
    dbinop(interp, |a, b| a + b)
}
fn dsub(interp: &mut Interpreter) -> Flow {
    dbinop(interp, |a, b| a - b)
}
fn dmul(interp: &mut Interpreter) -> Flow {
    dbinop(interp, |a, b| a * b)
}
fn ddiv(interp: &mut Interpreter) -> Flow {
    dbinop(interp, |a, b| a / b)
}
fn drem(interp: &mut Interpreter) -> Flow {
    dbinop(interp, |a, b| a % b)
}
fn dneg(interp: &mut Interpreter) -> Flow {
    let a = f64::from_bits(pop_wide(interp.regs()) as u64);
    push_wide(interp.regs(), (-a).to_bits() as i64);
    Flow::Continue
}

// ── 比较指令 ────────────────────────────────────────────────

/// `lcmp`：比较栈顶两个 long，结果（-1/0/1）以 int 入栈。
fn lcmp(interp: &mut Interpreter) -> Flow {
    let b = pop_wide(interp.regs());
    let a = pop_wide(interp.regs());
    let r = if a < b {
        -1i32
    } else if a > b {
        1
    } else {
        0
    };
    push_slot(interp.regs(), r);
    Flow::Continue
}

/// `fcmpl` / `fcmpg`：比较两个 float。
/// 如果有 NaN：`fcmpl` 返回 -1，`fcmpg` 返回 +1。`is_g` 区分二者。
fn fcmp(interp: &mut Interpreter, is_g: bool) -> Flow {
    let b = f32::from_bits(pop_slot(interp.regs()) as u32);
    let a = f32::from_bits(pop_slot(interp.regs()) as u32);
    let r = if a.is_nan() || b.is_nan() {
        if is_g { 1 } else { -1 }
    } else if a > b {
        1
    } else if a < b {
        -1
    } else {
        0
    };
    push_slot(interp.regs(), r);
    Flow::Continue
}

fn fcmpl(interp: &mut Interpreter) -> Flow {
    fcmp(interp, false)
}
fn fcmpg(interp: &mut Interpreter) -> Flow {
    fcmp(interp, true)
}

fn dcmp(interp: &mut Interpreter, is_g: bool) -> Flow {
    let b = f64::from_bits(pop_wide(interp.regs()) as u64);
    let a = f64::from_bits(pop_wide(interp.regs()) as u64);
    let r = if a.is_nan() || b.is_nan() {
        if is_g { 1 } else { -1 }
    } else if a > b {
        1
    } else if a < b {
        -1
    } else {
        0
    };
    push_slot(interp.regs(), r);
    Flow::Continue
}

fn dcmpl(interp: &mut Interpreter) -> Flow {
    dcmp(interp, false)
}
fn dcmpg(interp: &mut Interpreter) -> Flow {
    dcmp(interp, true)
}

// ── 类型转换 ────────────────────────────────────────────────

fn i2l(interp: &mut Interpreter) -> Flow {
    let a = pop_slot(interp.regs()) as i32 as i64;
    push_wide(interp.regs(), a);
    Flow::Continue
}
fn i2f(interp: &mut Interpreter) -> Flow {
    let a = pop_slot(interp.regs()) as i32 as f32;
    push_slot(interp.regs(), a.to_bits() as i32);
    Flow::Continue
}
fn i2d(interp: &mut Interpreter) -> Flow {
    let a = pop_slot(interp.regs()) as i32 as f64;
    push_wide(interp.regs(), a.to_bits() as i64);
    Flow::Continue
}

fn l2i(interp: &mut Interpreter) -> Flow {
    let a = pop_wide(interp.regs()) as i32;
    push_slot(interp.regs(), a);
    Flow::Continue
}
fn l2f(interp: &mut Interpreter) -> Flow {
    let a = pop_wide(interp.regs()) as f32;
    push_slot(interp.regs(), a.to_bits() as i32);
    Flow::Continue
}
fn l2d(interp: &mut Interpreter) -> Flow {
    let a = pop_wide(interp.regs()) as f64;
    push_wide(interp.regs(), a.to_bits() as i64);
    Flow::Continue
}

fn f2i(interp: &mut Interpreter) -> Flow {
    let a = f32::from_bits(pop_slot(interp.regs()) as u32);
    // JVM 规范：NaN 转为 0；溢出裁剪到 MIN/MAX。
    let r = if a.is_nan() {
        0
    } else if a >= i32::MAX as f32 {
        i32::MAX
    } else if a <= i32::MIN as f32 {
        i32::MIN
    } else {
        a as i32
    };
    push_slot(interp.regs(), r);
    Flow::Continue
}
fn f2l(interp: &mut Interpreter) -> Flow {
    let a = f32::from_bits(pop_slot(interp.regs()) as u32);
    let r = if a.is_nan() {
        0
    } else if a >= i64::MAX as f32 {
        i64::MAX
    } else if a <= i64::MIN as f32 {
        i64::MIN
    } else {
        a as i64
    };
    push_wide(interp.regs(), r);
    Flow::Continue
}
fn f2d(interp: &mut Interpreter) -> Flow {
    let a = f32::from_bits(pop_slot(interp.regs()) as u32) as f64;
    push_wide(interp.regs(), a.to_bits() as i64);
    Flow::Continue
}

fn d2i(interp: &mut Interpreter) -> Flow {
    let a = f64::from_bits(pop_wide(interp.regs()) as u64);
    let r = if a.is_nan() {
        0
    } else if a >= i32::MAX as f64 {
        i32::MAX
    } else if a <= i32::MIN as f64 {
        i32::MIN
    } else {
        a as i32
    };
    push_slot(interp.regs(), r);
    Flow::Continue
}
fn d2l(interp: &mut Interpreter) -> Flow {
    let a = f64::from_bits(pop_wide(interp.regs()) as u64);
    let r = if a.is_nan() {
        0
    } else if a >= i64::MAX as f64 {
        i64::MAX
    } else if a <= i64::MIN as f64 {
        i64::MIN
    } else {
        a as i64
    };
    push_wide(interp.regs(), r);
    Flow::Continue
}
fn d2f(interp: &mut Interpreter) -> Flow {
    let a = f64::from_bits(pop_wide(interp.regs()) as u64) as f32;
    push_slot(interp.regs(), a.to_bits() as i32);
    Flow::Continue
}

// ── int 位运算 / 位移 ────────────────────────────────────────

fn iand(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| a & b)
}
fn ior(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| a | b)
}
fn ixor(interp: &mut Interpreter) -> Flow {
    ibinop(interp, |a, b| a ^ b)
}
/// `ishl`：左移，移位量为 b & 0x1f。
fn ishl(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    push_slot(interp.regs(), a.wrapping_shl((b & 0x1f) as u32));
    Flow::Continue
}
/// `ishr`：算术右移。
fn ishr(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    push_slot(interp.regs(), a.wrapping_shr((b & 0x1f) as u32));
    Flow::Continue
}
/// `iushr`：逻辑右移。
fn iushr(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs()) as u32;
    push_slot(interp.regs(), (a >> (b & 0x1f) as u32) as i32);
    Flow::Continue
}

// ── long 位运算 / 位移 ────────────────────────────────────────

fn land(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| a & b)
}
fn lor(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| a | b)
}
fn lxor(interp: &mut Interpreter) -> Flow {
    lbinop(interp, |a, b| a ^ b)
}
/// `lshl`：long 左移，移位量为 int b & 0x3f。
fn lshl(interp: &mut Interpreter) -> Flow {
    let b = (pop_slot(interp.regs()) & 0x3f) as u32;
    let a = pop_wide(interp.regs());
    push_wide(interp.regs(), a.wrapping_shl(b));
    Flow::Continue
}
/// `lshr`：long 算术右移。
fn lshr(interp: &mut Interpreter) -> Flow {
    let b = (pop_slot(interp.regs()) & 0x3f) as u32;
    let a = pop_wide(interp.regs());
    push_wide(interp.regs(), a.wrapping_shr(b));
    Flow::Continue
}
/// `lushr`：long 逻辑右移。
fn lushr(interp: &mut Interpreter) -> Flow {
    let b = (pop_slot(interp.regs()) & 0x3f) as u32;
    let a = pop_wide(interp.regs()) as u64;
    push_wide(interp.regs(), (a >> b) as i64);
    Flow::Continue
}

// ── 栈操作（控制流常用配套）───────────────────────────────────────

/// `pop`：弾出栈顶 1 个分类一槽。
fn pop(interp: &mut Interpreter) -> Flow {
    pop_slot(interp.regs());
    Flow::Continue
}

/// `pop2`：弾出栈顶 1 个分类二（2 槽）或 2 个分类一槽。
/// 简单实现按 2 槽处理。
fn pop2(interp: &mut Interpreter) -> Flow {
    let _ = pop_slot(interp.regs());
    let _ = pop_slot(interp.regs());
    Flow::Continue
}

/// `dup`：复制栈顶分类一值。
fn dup(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    push_slot(interp.regs(), v);
    push_slot(interp.regs(), v);
    Flow::Continue
}

/// `dup_x1`：复制栈顶值并插入到下面两个之下。
fn dup_x1(interp: &mut Interpreter) -> Flow {
    let v1 = pop_slot(interp.regs());
    let v2 = pop_slot(interp.regs());
    push_slot(interp.regs(), v1);
    push_slot(interp.regs(), v2);
    push_slot(interp.regs(), v1);
    Flow::Continue
}

/// `dup_x2`：复制栈顶值并插入到下面三个之下。
fn dup_x2(interp: &mut Interpreter) -> Flow {
    let v1 = pop_slot(interp.regs());
    let v2 = pop_slot(interp.regs());
    let v3 = pop_slot(interp.regs());
    push_slot(interp.regs(), v1);
    push_slot(interp.regs(), v3);
    push_slot(interp.regs(), v2);
    push_slot(interp.regs(), v1);
    Flow::Continue
}

/// `dup2`：复制栈顶分类二值（2 槽）。
fn dup2(interp: &mut Interpreter) -> Flow {
    let hi = pop_slot(interp.regs());
    let lo = pop_slot(interp.regs());
    push_slot(interp.regs(), hi);
    push_slot(interp.regs(), lo);
    push_slot(interp.regs(), hi);
    push_slot(interp.regs(), lo);
    Flow::Continue
}

// ── 控制流：int 与 0 比较 ────────────────────────────────────────

/// `ifeq`：栈顶 == 0 则跳。
fn ifeq(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v == 0)
}

/// `ifne`：栈顶 != 0 则跳。
fn ifne(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v != 0)
}

/// `iflt`：栈顶 < 0 则跳。
fn iflt(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v < 0)
}

/// `ifge`：栈顶 >= 0 则跳。
fn ifge(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v >= 0)
}

/// `ifgt`：栈顶 > 0 则跳。
fn ifgt(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v > 0)
}

/// `ifle`：栈顶 <= 0 则跳。
fn ifle(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v <= 0)
}

// ── 控制流：两个 int 比较 ───────────────────────────────────────

/// `if_icmpeq`：栈顶两 int（a 为次顶，b 为顶）满足 a == b 则跳。
fn if_icmpeq(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    interp.branch_if(a == b)
}

fn if_icmpne(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    interp.branch_if(a != b)
}

fn if_icmplt(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    interp.branch_if(a < b)
}

fn if_icmpge(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    interp.branch_if(a >= b)
}

fn if_icmpgt(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    interp.branch_if(a > b)
}

fn if_icmple(interp: &mut Interpreter) -> Flow {
    let b = pop_slot(interp.regs());
    let a = pop_slot(interp.regs());
    interp.branch_if(a <= b)
}

// ── 控制流：null 比较 ────────────────────────────────────────────
// null 在当前实现里就是槽位 0（还没有真正的引用类型，先把语义占位）。

/// `ifnull`：栈顶引用 == null 则跳。
fn ifnull(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v == 0)
}

/// `ifnonnull`：栈顶引用 != null 则跳。
fn ifnonnull(interp: &mut Interpreter) -> Flow {
    let v = pop_slot(interp.regs());
    interp.branch_if(v != 0)
}

// ── 控制流：无条件跳转 ────────────────────────────────────────────

/// `goto`：2 字节偏移的无条件跳转。
fn goto_(interp: &mut Interpreter) -> Flow {
    interp.goto_branch()
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

    // 常量加载
    t[0x02] = iconst::<-1>; // iconst_m1
    t[0x03] = iconst::<0>;
    t[0x04] = iconst::<1>;
    t[0x05] = iconst::<2>;
    t[0x06] = iconst::<3>;
    t[0x07] = iconst::<4>;
    t[0x08] = iconst::<5>;
    t[0x09] = lconst::<0>; // lconst_0
    t[0x0a] = lconst::<1>; // lconst_1
    t[0x0b] = fconst_0;
    t[0x0c] = fconst_1;
    t[0x0d] = fconst_2;
    t[0x0e] = dconst_0;
    t[0x0f] = dconst_1;

    // 立即数 push / 常量池加载
    t[0x10] = bipush;
    t[0x11] = sipush;
    t[0x12] = ldc;
    t[0x13] = ldc_w;
    t[0x14] = ldc2_w;

    // iload / iload_<n>
    t[0x15] = iload;
    t[0x1a] = iload_n::<0>;
    t[0x1b] = iload_n::<1>;
    t[0x1c] = iload_n::<2>;
    t[0x1d] = iload_n::<3>;
    // lload / lload_<n>
    t[0x16] = lload;
    t[0x1e] = lload_n::<0>;
    t[0x1f] = lload_n::<1>;
    t[0x20] = lload_n::<2>;
    t[0x21] = lload_n::<3>;
    // fload / fload_<n>
    t[0x17] = fload;
    t[0x22] = fload_n::<0>;
    t[0x23] = fload_n::<1>;
    t[0x24] = fload_n::<2>;
    t[0x25] = fload_n::<3>;
    // dload / dload_<n>
    t[0x18] = dload;
    t[0x26] = dload_n::<0>;
    t[0x27] = dload_n::<1>;
    t[0x28] = dload_n::<2>;
    t[0x29] = dload_n::<3>;
    // aload / aload_<n>
    t[0x19] = aload;
    t[0x2a] = aload_n::<0>;
    t[0x2b] = aload_n::<1>;
    t[0x2c] = aload_n::<2>;
    t[0x2d] = aload_n::<3>;

    // istore / istore_<n>
    t[0x36] = istore;
    t[0x3b] = istore_n::<0>;
    t[0x3c] = istore_n::<1>;
    t[0x3d] = istore_n::<2>;
    t[0x3e] = istore_n::<3>;
    // lstore / lstore_<n>
    t[0x37] = lstore;
    t[0x3f] = lstore_n::<0>;
    t[0x40] = lstore_n::<1>;
    t[0x41] = lstore_n::<2>;
    t[0x42] = lstore_n::<3>;
    // fstore / fstore_<n>
    t[0x38] = fstore;
    t[0x43] = fstore_n::<0>;
    t[0x44] = fstore_n::<1>;
    t[0x45] = fstore_n::<2>;
    t[0x46] = fstore_n::<3>;
    // dstore / dstore_<n>
    t[0x39] = dstore;
    t[0x47] = dstore_n::<0>;
    t[0x48] = dstore_n::<1>;
    t[0x49] = dstore_n::<2>;
    t[0x4a] = dstore_n::<3>;
    // astore / astore_<n>
    t[0x3a] = astore;
    t[0x4b] = astore_n::<0>;
    t[0x4c] = astore_n::<1>;
    t[0x4d] = astore_n::<2>;
    t[0x4e] = astore_n::<3>;

    // int 算术
    t[0x60] = iadd;
    t[0x64] = isub;
    t[0x68] = imul;
    t[0x6c] = idiv;
    t[0x70] = irem;
    t[0x74] = ineg;
    // long 算术
    t[0x61] = ladd;
    t[0x65] = lsub;
    t[0x69] = lmul;
    t[0x6d] = ldiv;
    t[0x71] = lrem;
    t[0x75] = lneg;
    // float 算术
    t[0x62] = fadd;
    t[0x66] = fsub;
    t[0x6a] = fmul;
    t[0x6e] = fdiv;
    t[0x72] = frem;
    t[0x76] = fneg;
    // double 算术
    t[0x63] = dadd;
    t[0x67] = dsub;
    t[0x6b] = dmul;
    t[0x6f] = ddiv;
    t[0x73] = drem;
    t[0x77] = dneg;
    // int 位运算 / 位移
    t[0x7e] = iand;
    t[0x80] = ior;
    t[0x82] = ixor;
    t[0x78] = ishl;
    t[0x7a] = ishr;
    t[0x7c] = iushr;
    // long 位运算 / 位移
    t[0x7f] = land;
    t[0x81] = lor;
    t[0x83] = lxor;
    t[0x79] = lshl;
    t[0x7b] = lshr;
    t[0x7d] = lushr;

    // iinc
    t[0x84] = iinc;

    // 栈操作
    t[0x57] = pop;
    t[0x58] = pop2;
    t[0x59] = dup;
    t[0x5a] = dup_x1;
    t[0x5b] = dup_x2;
    t[0x5c] = dup2;

    // int 与 0 比较
    t[0x99] = ifeq;
    t[0x9a] = ifne;
    t[0x9b] = iflt;
    t[0x9c] = ifge;
    t[0x9d] = ifgt;
    t[0x9e] = ifle;

    // 两个 int 比较
    t[0x9f] = if_icmpeq;
    t[0xa0] = if_icmpne;
    t[0xa1] = if_icmplt;
    t[0xa2] = if_icmpge;
    t[0xa3] = if_icmpgt;
    t[0xa4] = if_icmple;

    // null 比较
    t[0xc6] = ifnull;
    t[0xc7] = ifnonnull;

    // 无条件跳转
    t[0xa7] = goto_;

    // 返回
    t[0xac] = ireturn;
    t[0xad] = lreturn;
    t[0xae] = freturn;
    t[0xaf] = dreturn;
    t[0xb1] = vreturn; // void `return`

    // 类型转换
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

    // 比较
    t[0x94] = lcmp;
    t[0x95] = fcmpl;
    t[0x96] = fcmpg;
    t[0x97] = dcmpl;
    t[0x98] = dcmpg;

    t
}
