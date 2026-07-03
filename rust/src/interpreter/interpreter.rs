use std::marker::PhantomData;
use std::ptr::null;

use crate::oops::acc_flags::AccFlags;
use crate::oops::cp_entry::CPEntry;
use crate::oops::desc::FieldDesc;
use crate::oops::oop_handle::ObjPtr;
use crate::oops::{attr::CodeAttr, method::Method, normal_klass::NormalKlass};

/// 一个 JVM 栈槽。  long / double 占据两个相邻槽（高 32 位在低地址）。
pub type StackSlot = i32;
pub type DStackSlot = i64;

/// 执行一条字节码后控制流的去向。
pub enum Flow {
    /// 继续顺序执行下一条字节码。
    Continue,
    /// 当前方法返回。  携带方法返回值（`None` 表示 `void`）。
    ///
    /// long / double 以"高 32 位在低地址、低 32 位在高地址"的双槽形式存放。
    Return(Option<ReturnValue>),
    /// `athrow`。  暂时未启用（阶段 5）。
    #[allow(dead_code)]
    Throw(ObjPtr),
}

/// 方法返回值。
#[derive(Clone, Copy, Debug)]
pub enum ReturnValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Ref(ObjPtr),
}

// ── 栈帧 ──────────────────────────────────────────────────────────────

/// 当前执行上下文。  采用 HotSpot 风格布局：
///
/// ```text
///   ... 上层帧 ...
///   ┌──────────────┐ ◄── bp（locals 起点）
///   │  locals[0]   │
///   │  locals[1]   │
///   │   ...        │
///   │ locals[N-1]  │
///   ├──────────────┤
///   │ expr stack   │
///   │   (向下生长)  │ ◄── sp（栈顶槽地址；空栈时 sp = bp + max_locals + max_stack）
///   │              │
///   └──────────────┘
/// ```
///
/// locals 与 expression stack 在内存中相邻，sp 向**低**地址方向递减。
pub(super) struct Registers {
    __: PhantomData<()>,

    /// locals 起点。
    pub bp: *mut StackSlot,
    /// operand stack 栈顶（指向**当前栈顶元素**，空栈时指向该帧 expr 区末端 +1）。
    pub sp: *mut StackSlot,
    /// 程序计数器。
    pub pc: *mut u8,

    pub klass: *const NormalKlass,
    pub method: *const Method,
}

impl Registers {
    pub fn code(&self) -> &CodeAttr {
        unsafe { (*self.method).code.as_ref().unwrap_unchecked() }
    }

    /// 读取当前类常量池索引处的条目。
    #[inline]
    pub(super) fn cp_get(&self, idx: usize) -> &CPEntry {
        unsafe { (*self.klass).cp_get(idx) }
    }
}

// ── 栈帧管理 ──────────────────────────────────────────────────────────

/// 一个被压入解释器栈的帧的边界信息，用于回收。
struct FrameRecord {
    /// locals 起点（即该帧在 `stack` 中的起点）。
    base: usize,
    /// 该帧占用 `stack` 的总槽数 = max_locals + max_stack。
    total_slots: usize,
}

pub struct Interpreter {
    /// 全局执行栈，按 slot 计数。
    stack: Box<[StackSlot]>,
    /// 已压入的帧记录，用于 invoke 返回时回收。
    frames: Vec<FrameRecord>,
    /// 当前寄存器组。
    regs: Registers,
}

/// 方法返回值类型（用于静态校验）。
#[derive(Debug)]
pub enum InvokeError {
    /// 目标方法不是 ACC_STATIC。
    NotStatic,
    /// 目标方法无 Code 属性（abstract / native）。
    NoCode,
    /// 实参 slot 数与描述符不匹配。
    ArgCountMismatch { expected: usize, actual: usize },
    /// 栈空间不足。
    StackOverflow,
}

pub type InvokeResult = Result<Option<ReturnValue>, InvokeError>;

impl Interpreter {
    /// 创建一个拥有 `stack_words` 个槽的解释器栈。
    pub fn new(stack_words: usize) -> Self {
        let stack = vec![0i32; stack_words].into_boxed_slice();

        // 占位寄存器——真正的值由第一次 invoke 填入。
        let regs = Registers {
            __: PhantomData,
            bp: std::ptr::null_mut(),
            sp: std::ptr::null_mut(),
            pc: std::ptr::null_mut(),
            klass: null(),
            method: null(),
        };

        Self {
            stack,
            frames: Vec::new(),
            regs,
        }
    }

    /// 调用一个 `ACC_STATIC` 方法。
    ///
    /// `args` 按槽给出（long / double 占两槽，高 32 位在低地址），顺序与方法描述符一致。
    pub fn invoke_static(
        &mut self,
        klass: &NormalKlass,
        method: &Method,
        args: &[StackSlot],
    ) -> InvokeResult {
        if !method.acc_flags.contains(AccFlags::ACC_STATIC) {
            return Err(InvokeError::NotStatic);
        }
        let code = method.code.as_ref().ok_or(InvokeError::NoCode)?;
        let max_locals = code.max_locals as usize;
        let max_stack = code.max_stack as usize;

        // 局部变量区：实参 + 编译器补充的额外局部变量槽。
        let arg_slot_count = Self::arg_slot_count(&method);
        if args.len() != arg_slot_count {
            return Err(InvokeError::ArgCountMismatch {
                expected: arg_slot_count,
                actual: args.len(),
            });
        }
        if args.len() > max_locals {
            return Err(InvokeError::ArgCountMismatch {
                expected: max_locals,
                actual: args.len(),
            });
        }

        let total_slots = max_locals + max_stack;
        let base = match self.frames.last() {
            None => 0,
            Some(parent) => parent.base + parent.total_slots,
        };
        if base + total_slots > self.stack.len() {
            return Err(InvokeError::StackOverflow);
        }

        // 1. 写入实参到 locals[0..arg_slot_count)，剩余槽清零。
        let locals = &mut self.stack[base..base + max_locals];
        for (i, a) in args.iter().enumerate() {
            locals[i] = *a;
        }
        for slot in locals[arg_slot_count..].iter_mut() {
            *slot = 0;
        }

        // 2. 设置 sp：空栈时指向 expr 区末尾的"下一个槽"（push 时 sp-=1 再写）。
        let stack_ptr = self.stack.as_mut_ptr();
        let expr_base = base + max_locals;
        let sp = unsafe { stack_ptr.add(expr_base + max_stack) };

        // 3. 设置 pc。
        let pc = code.code.as_ptr() as *mut u8;

        // 4. 记录帧并切换寄存器。
        self.frames.push(FrameRecord { base, total_slots });
        let bp = unsafe { stack_ptr.add(base) };
        let saved = std::mem::replace(
            &mut self.regs,
            Registers {
                __: PhantomData,
                bp,
                sp,
                pc,
                klass,
                method,
            },
        );

        // 5. 跑循环。
        let flow = self.run_loop();

        // 6. 恢复寄存器并回收帧。
        self.regs = saved;
        self.frames.pop();

        match flow {
            Flow::Continue => Ok(None), // 不应发生：方法以 return 结尾
            Flow::Return(v) => Ok(v),
            Flow::Throw(_) => Ok(None), // TODO 阶段 5：异常
        }
    }

    /// 计算静态方法的实参 slot 数（long / double 算 2）。
    fn arg_slot_count(method: &Method) -> usize {
        method
            .desc
            .params_desc
            .iter()
            .map(|d| arg_slots_of(d))
            .sum()
    }

    fn run_loop(&mut self) -> Flow {
        loop {
            let opcode = unsafe { *self.regs.pc } as usize;
            self.regs.pc = unsafe { self.regs.pc.add(1) };
            let handler = super::instructions::instruction_table()[opcode];
            match handler(self) {
                Flow::Continue => {}
                other => return other,
            }
        }
    }
}

/// 一个字段描述符占用几个栈槽。
pub(super) fn arg_slots_of(d: &FieldDesc) -> usize {
    if d.byte_size() > 4 { 2 } else { 1 }
}

// ── 给指令 handler 用的辅助方法 ───────────────────────────────────────

impl Interpreter {
    /// 当前寄存器组的引用。
    pub(super) fn regs(&mut self) -> &mut Registers {
        &mut self.regs
    }

    /// 压入一个栈槽。  sp 先减 1 再写入（栈向低地址增长）。
    #[inline]
    pub(super) fn push_slot(&mut self, v: StackSlot) {
        unsafe {
            self.regs.sp = self.regs.sp.sub(1);
            *self.regs.sp = v;
        }
    }

    /// 弹出一个栈槽。
    #[inline]
    pub(super) fn pop_slot(&mut self) -> StackSlot {
        unsafe {
            let v = *self.regs.sp;
            self.regs.sp = self.regs.sp.add(1);
            v
        }
    }

    /// 压入一个 long / double（2 槽）。  约定：高 32 位在低地址。
    #[inline]
    pub(super) fn push_long(&mut self, v: DStackSlot) {
        let hi = (v >> 32) as i32;
        let lo = v as i32;
        self.push_slot(hi); // 先入低地址 → 高 32 位
        self.push_slot(lo); // 后入高地址 → 低 32 位
    }

    /// 弹出一个 long / double（2 槽）。  约定：高 32 位在低地址。
    #[inline]
    pub(super) fn pop_long(&mut self) -> DStackSlot {
        // 先弹 lo（高地址）再弹 hi（低地址）。
        // 注意：lo 是低 32 位，转 i64 前必须先走 u32，避免符号扩展污染高位。
        let lo = self.pop_slot() as u32;
        let hi = self.pop_slot() as u32;
        (((hi as u64) << 32) | (lo as u64)) as i64
    }

    /// 从字节码读取 1 个 u8 操作数并推进 pc。
    #[inline]
    pub(super) fn read_u8(&mut self) -> u8 {
        let v = unsafe { *self.regs.pc };
        self.regs.pc = unsafe { self.regs.pc.add(1) };
        v
    }

    /// 从字节码读取 1 个 u16（big-endian）操作数并推进 pc。
    #[inline]
    pub(super) fn read_u16(&mut self) -> u16 {
        let hi = unsafe { *self.regs.pc } as u16;
        let lo = unsafe { *self.regs.pc.add(1) } as u16;
        self.regs.pc = unsafe { self.regs.pc.add(2) };
        (hi << 8) | lo
    }

    /// 读一个 i8。
    #[inline]
    pub(super) fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    /// 读一个 i16（big-endian）。
    #[inline]
    pub(super) fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }

    /// 执行条件分支。在 handler 进入时 pc 指向 opcode 之后（由 run_loop 推进 1），
    /// `cond` 为真时跳到 `insn_start + rel`（insn_start = pc - 1）。
    ///
    /// 内部会读取 2 字节分支偏移操作数。
    #[inline]
    pub(super) fn branch_if(&mut self, cond: bool) -> Flow {
        let rel = self.read_i16() as isize;
        if cond {
            // handler 进入时 pc = insn_start + 1；读 2 字节后 pc = insn_start + 3。
            // 目标 = insn_start + rel = (pc - 3) + rel。
            let target = (self.regs.pc as isize - 3 + rel) as *mut u8;
            self.regs.pc = target;
        }
        Flow::Continue
    }

    /// 无条件分支。语义同 [`Self::branch_if`] 但总是跳转。
    #[inline]
    pub(super) fn goto_branch(&mut self) -> Flow {
        self.branch_if(true)
    }
}
