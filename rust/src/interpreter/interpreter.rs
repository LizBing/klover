use std::marker::PhantomData;
use std::ptr::null;

use crate::interpreter::instructions;
use crate::oops::acc_flags::AccFlags;
use crate::oops::cp_entry::CPEntry;
use crate::oops::desc::FieldDesc;
use crate::oops::{attr::Code, method::Method, normal_klass::NormalKlass};

/// 一个 JVM 栈槽。  long / double 占据两个相邻槽（高 32 位在低地址）。
pub type StackSlot = i32;
pub type DStackSlot = i64;

/// 执行一条字节码后控制流的去向。
pub enum Flow {
    /// 继续顺序执行下一条字节码。
    Continue,
    /// 当前方法返回。  携带方法返回值（`None` 表示 `void`）。
    ///
    /// long / double 以“高 32 位在低地址、低 32 位在高地址”的双槽形式存放。
    Return(Option<ReturnValue>),
    /// `athrow` 或异常未在当前方法捕获。  携带异常对象的 narrow ptr。
    Throw(u32),
}

/// 方法返回值。
#[derive(Clone, Copy, Debug)]
pub enum ReturnValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /// 引用返回值，以 narrow ptr（u32）形式存放，与栈槽语义一致。
    Ref(u32),
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
    pub fn code(&self) -> &Code {
        unsafe { (*self.method).code.as_ref().expect("no code") }
    }

    /// 读取当前类常量池索引处的条目。
    #[inline]
    pub(super) fn cp_get(&self, idx: usize) -> Option<&CPEntry> {
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

/// 方法调用的结果。
pub enum InvokeOutcome {
    /// 正常返回。  `None` 表示 void。
    Returned(Option<ReturnValue>),
    /// 异常抛出且未被被调用方法自身捕获。  携带异常对象的 narrow ptr。
    /// 调用方（通常是 `invoke_resolved`）应在自己的帧里继续查找 handler。
    Thrown(u32),
}

pub type InvokeResult = Result<InvokeOutcome, InvokeError>;

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

    /// 调用一个已解析的方法（static 或 instance 皆可）。
    ///
    /// `args` 按槽给出（long / double 占两槽，高 32 位在低地址），顺序与方法描述符一致。
    /// 实例方法需在 `args[0]` 传入 `this`（narrow ptr）。
    ///
    /// 本方法是帧建立 / 寄存器切换 / run_loop 的唯一实现点。
    /// `invoke_static` 与指令 handler (`invokespecial` / `invokevirtual`) 都调它。
    fn invoke_resolved(
        &mut self,
        klass: &NormalKlass,
        method: &Method,
        args: &[StackSlot],
    ) -> InvokeResult {
        unimplemented!()
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
        self.invoke_resolved(klass, method, args)
    }

    /// 调用一个实例方法（已解析出目标方法）。
    ///
    /// `args[0]` 必须是 `this`（narrow ptr），其后为方法参数。
    /// 供 `invokespecial` / `invokevirtual` handler 使用。
    pub(super) fn invoke_instance(
        &mut self,
        klass: &NormalKlass,
        method: &Method,
        args: &[StackSlot],
    ) -> InvokeResult {
        self.invoke_resolved(klass, method, args)
    }

    /// 计算实例方法的实参 slot 数（含 this）。
    /// static 方法的实参 slot 数（不含 this）由 `arg_slot_count` 提供。
    pub(super) fn instance_arg_slot_count(method: &Method) -> usize {
        Self::arg_slot_count(method) + 1
    }

    /// static 方法的实参 slot 数（不含 this）。
    pub(super) fn static_arg_slot_count(method: &Method) -> usize {
        Self::arg_slot_count(method)
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
            let handler = instructions::instruction_table()[opcode];
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

    /// 读栈顶但不弹出。
    #[inline]
    pub(super) fn peek_slot(&self) -> StackSlot {
        unsafe { *self.regs.sp }
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

    /// 读一个 u32（big-endian）。
    #[inline]
    pub(super) fn read_u32(&mut self) -> u32 {
        let b0 = self.read_u8() as u32;
        let b1 = self.read_u8() as u32;
        let b2 = self.read_u8() as u32;
        let b3 = self.read_u8() as u32;
        (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    }

    /// 读一个 i32（big-endian）。
    #[inline]
    pub(super) fn read_i32(&mut self) -> i32 {
        self.read_u32() as i32
    }

    /// 将 pc 对齐到 4 字节边界（相对方法 code 起点）。
    ///
    /// 用于 tableswitch / lookupswitch 的 0-3 字节 padding。
    /// JVM 规范：opcode 后跟 padding，使 default 偏移量的起始位置相对于
    /// 方法 code 起点是 4 的倍数。
    ///
    /// 调用时 pc 已越过 opcode（位于 insn_start + 1）。
    #[inline]
    pub(super) fn align_pc_to_4(&mut self) {
        let code_start = self.regs.code().code.as_ptr() as usize;
        // insn_start = pc - 1；default 起始位置 = insn_start + 1 + pad = pc + pad。
        // 要求 (pc + pad - code_start) % 4 == 0。
        let pc_addr = self.regs.pc as usize;
        let rel = (pc_addr - code_start) % 4;
        if rel != 0 {
            let pad = 4 - rel;
            self.regs.pc = unsafe { self.regs.pc.add(pad) };
        }
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
