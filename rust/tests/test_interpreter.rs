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
    interpreter::interpreter::{Interpreter, ReturnValue},
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
