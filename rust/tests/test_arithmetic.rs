use std::sync::Once;

use klover::{
    class_loader::{bootstrap_cld::BootstrapCLD, ms_api::MSRef},
    engine::{
        call::Invocation,
        exec_error::JavaExceptionKind,
        exec_dispatcher::ExecDispatcher,
        outcome::{PendingException, RetValue, RunOutcome, ThreadExit},
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

fn arithmetic_class() -> MSRef<NormalKlass> {
    init_vm();
    BootstrapCLD::find_class("ArithmeticOps")
        .unwrap()
        .as_normal_ref()
        .unwrap()
}

fn run(
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

fn int_args(a: i32, b: i32) -> Vec<Slot> {
    vec![Slot::int(a), Slot::int(b)]
}

fn long_args(a: i64, b: i64) -> Vec<Slot> {
    vec![
        Slot::long_high(a),
        Slot::long_low(a),
        Slot::long_high(b),
        Slot::long_low(b),
    ]
}

fn float_args(a: f32, b: f32) -> Vec<Slot> {
    vec![Slot::float(a), Slot::float(b)]
}

fn double_args(a: f64, b: f64) -> Vec<Slot> {
    vec![
        Slot::double_high(a),
        Slot::double_low(a),
        Slot::double_high(b),
        Slot::double_low(b),
    ]
}

fn long_shift_args(value: i64, distance: i32) -> Vec<Slot> {
    vec![
        Slot::long_high(value),
        Slot::long_low(value),
        Slot::int(distance),
    ]
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

fn expect_arithmetic_exception(exit: ThreadExit) {
    assert!(matches!(
        exit,
        ThreadExit::UncaughtException(PendingException::JVMGen(
            JavaExceptionKind::ArithmeticException
        ))
    ));
}

#[test]
fn integer_arithmetic() {
    let holder = arithmetic_class();

    assert_eq!(expect_int(run(&holder, "iadd", "(II)I", int_args(17, 5))), 22);
    assert_eq!(expect_int(run(&holder, "isub", "(II)I", int_args(17, 5))), 12);
    assert_eq!(expect_int(run(&holder, "imul", "(II)I", int_args(17, 5))), 85);
    assert_eq!(expect_int(run(&holder, "idiv", "(II)I", int_args(17, 5))), 3);
    assert_eq!(expect_int(run(&holder, "irem", "(II)I", int_args(17, 5))), 2);
    assert_eq!(expect_int(run(&holder, "ineg", "(I)I", vec![Slot::int(17)])), -17);
    assert_eq!(expect_int(run(&holder, "iinc", "(I)I", vec![Slot::int(17)])), 24);

    assert_eq!(
        expect_int(run(&holder, "iadd", "(II)I", int_args(i32::MAX, 1))),
        i32::MIN
    );
    assert_eq!(
        expect_int(run(&holder, "idiv", "(II)I", int_args(i32::MIN, -1))),
        i32::MIN
    );
    assert_eq!(
        expect_int(run(&holder, "irem", "(II)I", int_args(i32::MIN, -1))),
        0
    );
    assert_eq!(
        expect_int(run(&holder, "ineg", "(I)I", vec![Slot::int(i32::MIN)])),
        i32::MIN
    );
    assert_eq!(
        expect_int(run(&holder, "iinc", "(I)I", vec![Slot::int(i32::MAX - 3)])),
        i32::MIN + 3
    );
}

#[test]
fn long_arithmetic() {
    let holder = arithmetic_class();

    assert_eq!(expect_long(run(&holder, "ladd", "(JJ)J", long_args(17, 5))), 22);
    assert_eq!(expect_long(run(&holder, "lsub", "(JJ)J", long_args(17, 5))), 12);
    assert_eq!(expect_long(run(&holder, "lmul", "(JJ)J", long_args(17, 5))), 85);
    assert_eq!(expect_long(run(&holder, "ldiv", "(JJ)J", long_args(17, 5))), 3);
    assert_eq!(expect_long(run(&holder, "lrem", "(JJ)J", long_args(17, 5))), 2);
    assert_eq!(
        expect_long(run(
            &holder,
            "lneg",
            "(J)J",
            vec![Slot::long_high(17), Slot::long_low(17)],
        )),
        -17
    );

    assert_eq!(
        expect_long(run(&holder, "ladd", "(JJ)J", long_args(i64::MAX, 1))),
        i64::MIN
    );
    assert_eq!(
        expect_long(run(&holder, "ldiv", "(JJ)J", long_args(i64::MIN, -1))),
        i64::MIN
    );
    assert_eq!(
        expect_long(run(&holder, "lrem", "(JJ)J", long_args(i64::MIN, -1))),
        0
    );
    assert_eq!(
        expect_long(run(
            &holder,
            "lneg",
            "(J)J",
            vec![Slot::long_high(i64::MIN), Slot::long_low(i64::MIN)],
        )),
        i64::MIN
    );
}

#[test]
fn floating_point_arithmetic() {
    let holder = arithmetic_class();

    assert_eq!(expect_float(run(&holder, "fadd", "(FF)F", float_args(7.5, 2.0))), 9.5);
    assert_eq!(expect_float(run(&holder, "fsub", "(FF)F", float_args(7.5, 2.0))), 5.5);
    assert_eq!(expect_float(run(&holder, "fmul", "(FF)F", float_args(7.5, 2.0))), 15.0);
    assert_eq!(expect_float(run(&holder, "fdiv", "(FF)F", float_args(7.5, 2.0))), 3.75);
    assert_eq!(expect_float(run(&holder, "frem", "(FF)F", float_args(7.5, 2.0))), 1.5);
    assert_eq!(expect_float(run(&holder, "fneg", "(F)F", vec![Slot::float(7.5)])), -7.5);

    assert_eq!(expect_double(run(&holder, "dadd", "(DD)D", double_args(7.5, 2.0))), 9.5);
    assert_eq!(expect_double(run(&holder, "dsub", "(DD)D", double_args(7.5, 2.0))), 5.5);
    assert_eq!(expect_double(run(&holder, "dmul", "(DD)D", double_args(7.5, 2.0))), 15.0);
    assert_eq!(expect_double(run(&holder, "ddiv", "(DD)D", double_args(7.5, 2.0))), 3.75);
    assert_eq!(expect_double(run(&holder, "drem", "(DD)D", double_args(7.5, 2.0))), 1.5);
    assert_eq!(
        expect_double(run(
            &holder,
            "dneg",
            "(D)D",
            vec![Slot::double_high(7.5), Slot::double_low(7.5)],
        )),
        -7.5
    );

    assert!(expect_float(run(&holder, "fdiv", "(FF)F", float_args(1.0, 0.0))).is_infinite());
    assert!(expect_float(run(&holder, "frem", "(FF)F", float_args(1.0, 0.0))).is_nan());
    assert!(expect_double(run(&holder, "ddiv", "(DD)D", double_args(1.0, 0.0))).is_infinite());
    assert!(expect_double(run(&holder, "drem", "(DD)D", double_args(1.0, 0.0))).is_nan());

    let float_negative_zero = expect_float(run(
        &holder,
        "fneg",
        "(F)F",
        vec![Slot::float(0.0)],
    ));
    assert_eq!(float_negative_zero.to_bits(), (-0.0_f32).to_bits());

    let double_negative_zero = expect_double(run(
        &holder,
        "dneg",
        "(D)D",
        vec![Slot::double_high(0.0), Slot::double_low(0.0)],
    ));
    assert_eq!(double_negative_zero.to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn shifts_and_bitwise_operations() {
    let holder = arithmetic_class();

    assert_eq!(expect_int(run(&holder, "ishl", "(II)I", int_args(1, 33))), 2);
    assert_eq!(expect_int(run(&holder, "ishr", "(II)I", int_args(-8, 2))), -2);
    assert_eq!(
        expect_int(run(&holder, "iushr", "(II)I", int_args(-8, 2))),
        0x3fff_fffe
    );
    assert_eq!(expect_long(run(&holder, "lshl", "(JI)J", long_shift_args(1, 65))), 2);
    assert_eq!(expect_long(run(&holder, "lshr", "(JI)J", long_shift_args(-8, 2))), -2);
    assert_eq!(
        expect_long(run(&holder, "lushr", "(JI)J", long_shift_args(-8, 2))),
        0x3fff_ffff_ffff_fffe
    );

    assert_eq!(expect_int(run(&holder, "iand", "(II)I", int_args(0b1100, 0b1010))), 0b1000);
    assert_eq!(expect_int(run(&holder, "ior", "(II)I", int_args(0b1100, 0b1010))), 0b1110);
    assert_eq!(expect_int(run(&holder, "ixor", "(II)I", int_args(0b1100, 0b1010))), 0b0110);
    assert_eq!(expect_long(run(&holder, "land", "(JJ)J", long_args(0xcccc, 0xaaaa))), 0x8888);
    assert_eq!(expect_long(run(&holder, "lor", "(JJ)J", long_args(0xcccc, 0xaaaa))), 0xeeee);
    assert_eq!(expect_long(run(&holder, "lxor", "(JJ)J", long_args(0xcccc, 0xaaaa))), 0x6666);
}

#[test]
fn integer_division_by_zero_throws() {
    let holder = arithmetic_class();

    expect_arithmetic_exception(run(&holder, "idiv", "(II)I", int_args(1, 0)));
    expect_arithmetic_exception(run(&holder, "irem", "(II)I", int_args(1, 0)));
    expect_arithmetic_exception(run(&holder, "ldiv", "(JJ)J", long_args(1, 0)));
    expect_arithmetic_exception(run(&holder, "lrem", "(JJ)J", long_args(1, 0)));
}
