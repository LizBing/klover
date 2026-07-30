use std::sync::Once;

use klover::{
    class_loader::{bootstrap_cld::BootstrapCLD, ms_api::MSRef},
    engine::{
        call::Invocation,
        exec_dispatcher::ExecDispatcher,
        outcome::{RetValue, RunOutcome, ThreadExit},
        resolved_method::ResolvedMethod,
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

fn load_class(name: &str) -> MSRef<NormalKlass> {
    init_vm();
    BootstrapCLD::find_class(name)
        .unwrap()
        .as_normal_ref()
        .unwrap()
}

fn run(holder: &MSRef<NormalKlass>, name: &str, descriptor: &str) -> ThreadExit {
    let method = holder
        .find_declared_method(name, descriptor)
        .unwrap_or_else(|| panic!("method not found: {name}{descriptor}"));
    let target = ResolvedMethod::new(holder.clone(), method);

    let mut manager = ThreadManager::new(1024);
    let mut thread = manager.create_thread().unwrap();
    thread.start().unwrap();

    let mut dispatcher = ExecDispatcher::new();
    dispatcher
        .enter_root(
            &mut thread,
            Invocation {
                target,
                args: Vec::new(),
            },
        )
        .unwrap();

    loop {
        match dispatcher.run_quantum(&mut thread, 64).unwrap() {
            RunOutcome::QuantumExpired => continue,
            RunOutcome::Terminated(exit) => return exit,
        }
    }
}

fn expect_int(exit: ThreadExit) -> i32 {
    match exit {
        ThreadExit::Returned(RetValue::Int(value)) => value,
        other => panic!("expected int return, got {other:?}"),
    }
}

fn expect_long(exit: ThreadExit) -> i64 {
    match exit {
        ThreadExit::Returned(RetValue::Long(value)) => value,
        other => panic!("expected long return, got {other:?}"),
    }
}

fn expect_float(exit: ThreadExit) -> f32 {
    match exit {
        ThreadExit::Returned(RetValue::Float(value)) => value,
        other => panic!("expected float return, got {other:?}"),
    }
}

fn expect_double(exit: ThreadExit) -> f64 {
    match exit {
        ThreadExit::Returned(RetValue::Double(value)) => value,
        other => panic!("expected double return, got {other:?}"),
    }
}

#[test]
fn fixed_constants_and_null() {
    let holder = load_class("ConstantOps");

    match run(&holder, "nullConstant", "()Ljava/lang/Object;") {
        ThreadExit::Returned(RetValue::Ref(value)) => assert_eq!(value, 0),
        other => panic!("expected null reference return, got {other:?}"),
    }

    for (method, expected) in [
        ("intMinusOne", -1),
        ("intZero", 0),
        ("intOne", 1),
        ("intTwo", 2),
        ("intThree", 3),
        ("intFour", 4),
        ("intFive", 5),
    ] {
        assert_eq!(expect_int(run(&holder, method, "()I")), expected);
    }

    assert_eq!(expect_long(run(&holder, "longZero", "()J")), 0);
    assert_eq!(expect_long(run(&holder, "longOne", "()J")), 1);
    assert_eq!(expect_float(run(&holder, "floatZero", "()F")), 0.0);
    assert_eq!(expect_float(run(&holder, "floatOne", "()F")), 1.0);
    assert_eq!(expect_float(run(&holder, "floatTwo", "()F")), 2.0);
    assert_eq!(expect_double(run(&holder, "doubleZero", "()D")), 0.0);
    assert_eq!(expect_double(run(&holder, "doubleOne", "()D")), 1.0);
}

#[test]
fn immediate_constants_are_sign_extended() {
    let holder = load_class("ConstantOps");

    assert_eq!(expect_int(run(&holder, "byteImmediate", "()I")), -100);
    assert_eq!(expect_int(run(&holder, "shortImmediate", "()I")), -30000);
}

#[test]
fn constant_pool_numeric_values() {
    let holder = load_class("ConstantOps");

    assert_eq!(expect_int(run(&holder, "intPoolConstant", "()I")), 100000);
    assert_eq!(expect_float(run(&holder, "floatPoolConstant", "()F")), 3.25);
    assert_eq!(
        expect_long(run(&holder, "longPoolConstant", "()J")),
        1234567890123
    );
    assert_eq!(expect_double(run(&holder, "doublePoolConstant", "()D")), 3.25);
}

#[test]
fn wide_constant_pool_indexes() {
    let holder = load_class("LdcWideOps");

    assert_eq!(expect_int(run(&holder, "wideInt", "()I")), 123456789);
    assert_eq!(expect_float(run(&holder, "wideFloat", "()F")), 6.5);
}
