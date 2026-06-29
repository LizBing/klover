use std::{
    fs,
    ptr::{NonNull, null, null_mut},
};

use rust::{
    class_loader::{cld::ClassLoaderData, ms_box::ms_init},
    class_parser::class_file::ClassFile,
    interpreter::{
        instructions::INSTRUCTION_TABLE,
        interpreter::{Frame, Registers, StackSlot},
    },
    oops::{normal_klass::NormalKlass, oop_handle::init_oop_storages},
};

#[test]
fn test_simple_addition() {
    unsafe {
        ms_init();
        init_oop_storages();
    }

    let buffer =
        fs::read("../test_data/classes/SimpleAddition.class").expect("failed to open file");
    let cf = ClassFile::from(&buffer).unwrap();

    let cld = ClassLoaderData::new_phase1_test_cld();
    let cld_ptr =
        unsafe { NonNull::new_unchecked(&cld as *const ClassLoaderData as *mut ClassLoaderData) };

    let klass = NormalKlass::from(cf, cld_ptr).unwrap();

    let method = klass
        .find_method("add", "(II)I")
        .expect("method not found.");

    let mut stack = Vec::<i32>::with_capacity(4096);
    let frame = unsafe { &mut *(stack.as_mut_ptr().add(4096).byte_sub(size_of::<Frame>()) as *mut Frame) };
    frame.ctx = Registers {
        bp: null_mut(),
        sp: unsafe { stack.as_mut_ptr().add(4096) },
        pc: null_mut(),
    };
    frame.method = method;
    let code = unsafe { method.as_ref().code.as_ref().unwrap_unchecked() };
    let mut regs = Registers {
        bp: frame,
        sp: unsafe { (frame as *mut Frame as *mut StackSlot).sub(2) },
        pc: unsafe { &code.code.as_ref()[0] },
    };

    unsafe {
        *((frame as *mut Frame as *mut i32).sub(1)) = 1;
        *((frame as *mut Frame as *mut i32).sub(2)) = 2;
    }

    loop {
        let ins = unsafe { *regs.pc } as usize;
        let ins_fn = INSTRUCTION_TABLE[ins];

        ins_fn(&mut regs);

        if ins == 172 {
            break;
        }

        regs.pc = unsafe { regs.pc.add(1) };
    }

    println!("result: {}", unsafe { *stack.as_mut_ptr().add(4095) });
}
