#![allow(dead_code)]

use std::sync::Once;

use klover::{
    class_loader::{bootstrap_cld::BootstrapCLD, ms_api::MSRef},
    engine::{
        call::Invocation,
        exec_dispatcher::ExecDispatcher,
        outcome::{RetValue, RunOutcome, ThreadExit},
        resolved_method::ResolvedMethod,
        slot::Slot,
    },
    oops::normal_klass::NormalKlass,
    runtime::{arguments::Arguments, thread_manager::ThreadManager, vm::vm_init},
};

static VM_INIT: Once = Once::new();

fn init_vm() {
    VM_INIT.call_once(|| {
        vm_init(Arguments {
            bs_class_path: format!("{}/../test_data/classes", env!("CARGO_MANIFEST_DIR")),
            xmx: 64 * 1024 * 1024,
        });
    });
}

pub fn load_class(name: &str) -> MSRef<NormalKlass> {
    init_vm();
    BootstrapCLD::find_class(name)
        .unwrap()
        .as_normal_ref()
        .unwrap()
}

pub fn run(
    holder: &MSRef<NormalKlass>,
    name: &str,
    descriptor: &str,
    args: Vec<Slot>,
) -> ThreadExit {
    let method = holder
        .find_declared_method(name, descriptor)
        .unwrap_or_else(|| panic!("method not found: {name}{descriptor}"));
    let target = ResolvedMethod::new(holder.clone(), method);

    let mut manager = ThreadManager::new(1024);
    let mut thread = manager.create_thread().unwrap();
    thread.start().unwrap();

    let mut dispatcher = ExecDispatcher::new();
    dispatcher
        .enter_root(&mut thread, Invocation { target, args })
        .unwrap();

    loop {
        match dispatcher.run_quantum(&mut thread, 64).unwrap() {
            RunOutcome::QuantumExpired => continue,
            RunOutcome::Terminated(exit) => return exit,
        }
    }
}

pub fn expect_int(exit: ThreadExit) -> i32 {
    match exit {
        ThreadExit::Returned(RetValue::Int(value)) => value,
        other => panic!("expected int return, got {other:?}"),
    }
}

pub fn expect_long(exit: ThreadExit) -> i64 {
    match exit {
        ThreadExit::Returned(RetValue::Long(value)) => value,
        other => panic!("expected long return, got {other:?}"),
    }
}

pub fn expect_float(exit: ThreadExit) -> f32 {
    match exit {
        ThreadExit::Returned(RetValue::Float(value)) => value,
        other => panic!("expected float return, got {other:?}"),
    }
}

pub fn expect_double(exit: ThreadExit) -> f64 {
    match exit {
        ThreadExit::Returned(RetValue::Double(value)) => value,
        other => panic!("expected double return, got {other:?}"),
    }
}

pub fn expect_ref(exit: ThreadExit) -> u32 {
    match exit {
        ThreadExit::Returned(RetValue::Ref(value)) => value,
        other => panic!("expected reference return, got {other:?}"),
    }
}
