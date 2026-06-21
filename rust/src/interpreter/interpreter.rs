type StackSlot = u32;

#[repr(C)]
struct Frame {
    ctx: Registers,

    argc: i32,
    max_locals: i32,
}

#[repr(C)]
struct Registers {
    bp: *mut Frame,
    sp: *mut StackSlot,
    pc: i32
}

#[inline]
fn get_arg<T: Copy>(regs: &Registers, idx: i32) -> T {
    let arg_start = unsafe { regs.bp.add(1) as *const StackSlot };
    let addr = unsafe { arg_start.add(idx as _) as *const T };
    
    unsafe { *addr }
}

#[inline]
fn get_local<T: Copy>(regs: &Registers, idx: i32) -> T {
    let locals_start = unsafe { (regs.bp as *const StackSlot).sub((*regs.bp).max_locals as _) };
    let addr = unsafe { locals_start.add(idx as _) as *const T };

    unsafe { *addr }
}


