//! 阶段 1 端到端测试：通过解释器执行纯算术 / 常量加载的 Java 静态方法。
//!
//! 前置：`test_data/classes/*.class`（已纳入仓库）与
//! `test_data/classes/java/lang/Object.class`（由测试 setup 编译产生）。
//!
//! 覆盖的指令分组：
//!   - 常量推送：iconst_m1..5 / bipush / sipush / aconst_null
//!   - 局部变量：iload / iload_0..3 / istore / istore_0..3 / iinc
//!   - 算术：iadd
//!   - 返回：ireturn / return

use klover::{
    class_loader::bootstrap_cld::BootstrapCLD,
    interpreter::interpreter::{Interpreter, ReturnValue, StackSlot},
    oops::{normal_klass::NormalKlass, symbol_table::SymbolTable},
    runtime::arguments::Arguments,
};

unsafe extern "C" {
    fn ms_init() -> bool;
    fn init_oop_storages();
}

fn init_runtime(bs_class_path: &str) {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        unsafe {
            init_oop_storages();
            assert!(ms_init(), "ms_init failed");
        }
        Arguments::init(Arguments {
            bs_class_path: bs_class_path.to_string(),
        });
    });
}

fn find_static_method<'a>(
    klass: &'a NormalKlass,
    name: &str,
    desc: &str,
) -> &'a klover::oops::method::Method {
    let n = SymbolTable::intern(name);
    let d = SymbolTable::intern(desc);
    klass.find_method(&n, &d).expect("method not found")
}

/// 将一个 i64 拆成两个栈槽（高 32 位在前）。
fn long_slots(v: i64) -> [StackSlot; 2] {
    [(v >> 32) as i32, v as i32]
}

// ── 阶段 1：最小闭环 ────────────────────────────────────────────────────

/// `SimpleAddition.add(II)I`：
///   iload_0  iload_1  iadd  ireturn
#[test]
fn add_two_ints() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("SimpleAddition").expect("load SimpleAddition");
    let normal = klass.as_normal().expect("SimpleAddition is not Normal");
    let method = find_static_method(normal, "add", "(II)I");

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, &[3, 4])
        .expect("invoke_static");

    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 7),
        other => panic!("expected Int(7), got {:?}", other),
    }
}

/// `Arith.pushByte()I`：bipush 100; ireturn
#[test]
fn push_byte() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Arith").expect("load Arith");
    let normal = klass.as_normal().expect("Arith is not Normal");
    let method = find_static_method(normal, "pushByte", "()I");

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");

    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 100),
        other => panic!("expected Int(100), got {:?}", other),
    }
}

/// `Arith.pushShort()I`：sipush 10000; ireturn
#[test]
fn push_short() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Arith").expect("load Arith");
    let normal = klass.as_normal().expect("Arith is not Normal");
    let method = find_static_method(normal, "pushShort", "()I");

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");

    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 10000),
        other => panic!("expected Int(10000), got {:?}", other),
    }
}

/// `Arith.addConst()I`：iconst_0; istore_0; iinc 0,5; iinc 0,10; iload_0; ireturn
#[test]
fn add_const() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Arith").expect("load Arith");
    let normal = klass.as_normal().expect("Arith is not Normal");
    let method = find_static_method(normal, "addConst", "()I");

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");

    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 15),
        other => panic!("expected Int(15), got {:?}", other),
    }
}

// ── 阶段 2：int / long 算术与位运算 ────────────────────────────────────

/// Helper：调用 Arith 的指定 static 方法并断言返回 Int。
fn invoke_arith_int(name: &str, desc: &str, args: &[StackSlot], expected: i32) {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Arith").expect("load Arith");
    let normal = klass.as_normal().expect("Arith is not Normal");
    let method = find_static_method(normal, name, desc);

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, args)
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, expected),
        other => panic!("{:?}: expected Int({}), got {:?}", name, expected, other),
    }
}

#[test]
fn arith_mul2() {
    invoke_arith_int("mul2", "(I)I", &[21], 42);
}

#[test]
fn arith_neg() {
    invoke_arith_int("neg", "(I)I", &[5], -5);
}

#[test]
fn arith_sub() {
    invoke_arith_int("sub", "(II)I", &[10, 3], 7);
}

#[test]
fn arith_rem() {
    invoke_arith_int("rem", "(II)I", &[17, 5], 2);
}

/// Helper：调用 Wide 的指定 static long 方法并断言返回 Long。
fn invoke_wide_long(name: &str, desc: &str, args: &[StackSlot], expected: i64) {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Wide").expect("load Wide");
    let normal = klass.as_normal().expect("Wide is not Normal");
    let method = find_static_method(normal, name, desc);

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, args)
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Long(v)) => assert_eq!(v, expected),
        other => panic!("{:?}: expected Long({}), got {:?}", name, expected, other),
    }
}

#[test]
fn wide_ladd() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(1000));
    args.extend_from_slice(&long_slots(2000));
    invoke_wide_long("ladd", "(JJ)J", &args, 3000);
}

#[test]
fn wide_lsub() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(1000));
    args.extend_from_slice(&long_slots(300));
    invoke_wide_long("lsub", "(JJ)J", &args, 700);
}

#[test]
fn wide_lmul() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(100));
    args.extend_from_slice(&long_slots(200));
    invoke_wide_long("lmul", "(JJ)J", &args, 20000);
}

#[test]
fn wide_ldiv() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(1000));
    args.extend_from_slice(&long_slots(7));
    invoke_wide_long("ldiv", "(JJ)J", &args, 142);
}

#[test]
fn wide_lrem() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(1000));
    args.extend_from_slice(&long_slots(7));
    invoke_wide_long("lrem", "(JJ)J", &args, 6);
}

#[test]
fn wide_lneg() {
    invoke_wide_long("lneg", "(J)J", &long_slots(42), -42);
}

#[test]
fn wide_land() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(0xff00_ff00_ff00_ff00u64 as i64));
    args.extend_from_slice(&long_slots(0xf0f0_f0f0_f0f0_f0f0u64 as i64));
    invoke_wide_long("land", "(JJ)J", &args, 0xf000_f000_f000_f000u64 as i64);
}

#[test]
fn wide_lor() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(0x0f0f_0f0f_0f0f_0f0fu64 as i64));
    args.extend_from_slice(&long_slots(0xf0f0_f0f0_f0f0_f0f0u64 as i64));
    invoke_wide_long("lor", "(JJ)J", &args, -1);
}

#[test]
fn wide_lxor() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(0xff));
    args.extend_from_slice(&long_slots(0x0f));
    invoke_wide_long("lxor", "(JJ)J", &args, 0xf0);
}

#[test]
fn wide_lshl() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(1));
    args.push(8); // int 位移量
    invoke_wide_long("lshl", "(JI)J", &args, 256);
}

#[test]
fn wide_lshr() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(-256));
    args.push(8);
    invoke_wide_long("lshr", "(JI)J", &args, -1);
}

#[test]
fn wide_lushr() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(-256));
    args.push(8);
    invoke_wide_long(
        "lushr",
        "(JI)J",
        &args,
        (0xffff_ffff_ffff_ff00u64 >> 8) as i64,
    );
}

#[test]
fn wide_inc5() {
    // ldc int 2147483647
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Wide").expect("load Wide");
    let normal = klass.as_normal().expect("Wide is not Normal");
    let method = find_static_method(normal, "inc5", "()I");

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 2147483647),
        other => panic!("expected Int(2147483647), got {:?}", other),
    }
}
