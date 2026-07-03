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
        // 初始化 Java 堆（64MB 足够测试用）。
        unsafe {
            klover::gc_binding::gc_binding::init_heap(64 * 1024 * 1024);
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

// ── 阶段 4：对象分配与构造 ─────────────────────────────────────────────

/// `ObjTest.allocOnly()`：new + dup + invokespecial <init> + return 0。
/// 验证对象分配 + 构造器空跑不崩。
#[test]
fn obj_alloc_only() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "allocOnly", "()I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 0),
        other => panic!("expected Int(0), got {:?}", other),
    }
}

/// `ObjTest.setAndGetX()`：new + invokespecial + getfield。
/// 构造器内部 putfield x=7, y=9，然后 getfield x 返回 7。
#[test]
fn obj_set_and_get_x() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "setAndGetX", "()I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 7),
        other => panic!("expected Int(7), got {:?}", other),
    }
}

/// `ObjTest.createAndSum(3, 4)`：
/// 完整链路：new + dup + invokespecial <init> + invokevirtual sum。
/// <init> 用 putfield 写 x=3, y=4；sum 用 getfield 读 x+y 返回 7。
#[test]
fn obj_create_and_sum() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "createAndSum", "(II)I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[3, 4])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 7),
        other => panic!("expected Int(7), got {:?}", other),
    }
}

/// `ObjTest.getStaticTest()`：从字节码触发 invokestatic 路径。
#[test]
fn obj_invokestatic_from_bytecode() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "getStaticTest", "()I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 42),
        other => panic!("expected Int(42), got {:?}", other),
    }
}

// ── 数组指令测试 ───────────────────────────────────────────────────────

/// `ObjTest.newArrayTest()`：newarray + iastore + iaload + arraylength。
#[test]
fn array_new_int() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "newArrayTest", "()I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 35), // 10 + 20 + 5
        other => panic!("expected Int(35), got {:?}", other),
    }
}

/// `ObjTest.refArrayTest()`：anewarray + aastore + aaload + invokevirtual。
#[test]
fn array_ref() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "refArrayTest", "()I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 18), // (3+4) + (5+6)
        other => panic!("expected Int(18), got {:?}", other),
    }
}

/// `ObjTest.mixedArrayTest()`：byte/char/long 数组的 bastore/castore/lastore
/// 和对应 aload 的符号/零扩展。
#[test]
fn array_mixed() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ObjTest").expect("load ObjTest");
    let normal = klass.as_normal().expect("ObjTest is not Normal");
    let method = find_static_method(normal, "mixedArrayTest", "()I");

    let mut interp = Interpreter::new(4096);
    let ret = interp
        .invoke_static(normal, method, &[])
        .expect("invoke_static");
    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 1410065638),
        other => panic!("expected Int(1410065638), got {:?}", other),
    }
}

// ── 阶段 3：浮点 / 转换 / 比较 / 控制流 ─────────────────────────────────

/// Helper：调用 Wide 的指定 static 方法并断言返回 Int。
fn invoke_wide_int(name: &str, desc: &str, args: &[StackSlot], expected: i32) {
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
        Some(ReturnValue::Int(v)) => assert_eq!(v, expected),
        other => panic!("{:?}: expected Int({}), got {:?}", name, expected, other),
    }
}

/// Helper：调用 ControlFlow 的指定 static 方法并断言返回 Int。
fn invoke_control_flow(name: &str, desc: &str, args: &[StackSlot], expected: i32) {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ControlFlow").expect("load ControlFlow");
    let normal = klass.as_normal().expect("ControlFlow is not Normal");
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
fn cf_sum_10() {
    invoke_control_flow("sum", "(I)I", &[10], 55);
} // 1+2+...+10

#[test]
fn cf_sum_0() {
    invoke_control_flow("sum", "(I)I", &[0], 0);
} // while 不执行

#[test]
fn cf_max_ge() {
    invoke_control_flow("max", "(II)I", &[5, 3], 5);
}

#[test]
fn cf_max_lt() {
    invoke_control_flow("max", "(II)I", &[3, 5], 5);
}

#[test]
fn cf_count_to() {
    invoke_control_flow("countTo", "(I)I", &[7], 7);
}

#[test]
fn cf_factorial_5() {
    invoke_control_flow("factorial", "(I)I", &[5], 120);
}

#[test]
fn cf_first_even_from_odd() {
    invoke_control_flow("firstEven", "(I)I", &[3], 4);
}

#[test]
fn cf_first_even_from_even() {
    invoke_control_flow("firstEven", "(I)I", &[8], 8);
}

/// Helper：调用 Wide 的指定 static float 方法并断言返回 Float。
fn invoke_wide_float(name: &str, desc: &str, args: &[StackSlot], expected: f32) {
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
        Some(ReturnValue::Float(v)) => assert!(
            (v - expected).abs() < 1e-5,
            "{:?}: expected {}, got {}",
            name,
            expected,
            v
        ),
        other => panic!("{:?}: expected Float({}), got {:?}", name, expected, other),
    }
}

/// Helper：调用 Wide 的指定 static double 方法并断言返回 Double。
fn invoke_wide_double(name: &str, desc: &str, args: &[StackSlot], expected: f64) {
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
        Some(ReturnValue::Double(v)) => assert!(
            (v - expected).abs() < 1e-10,
            "{:?}: expected {}, got {}",
            name,
            expected,
            v
        ),
        other => panic!("{:?}: expected Double({}), got {:?}", name, expected, other),
    }
}

fn float_slot(v: f32) -> StackSlot {
    v.to_bits() as i32
}

fn double_slots(v: f64) -> [StackSlot; 2] {
    let bits = v.to_bits() as i64;
    long_slots(bits)
}

#[test]
fn wide_fadd() {
    invoke_wide_float("fadd", "(FF)F", &[float_slot(1.5), float_slot(2.5)], 4.0);
}

#[test]
fn wide_fsub() {
    invoke_wide_float("fsub", "(FF)F", &[float_slot(5.0), float_slot(1.5)], 3.5);
}

#[test]
fn wide_fmul() {
    invoke_wide_float("fmul", "(FF)F", &[float_slot(3.0), float_slot(4.0)], 12.0);
}

#[test]
fn wide_fdiv() {
    invoke_wide_float("fdiv", "(FF)F", &[float_slot(7.0), float_slot(2.0)], 3.5);
}

#[test]
fn wide_fneg() {
    invoke_wide_float("fneg", "(F)F", &[float_slot(3.5)], -3.5);
}

#[test]
fn wide_dadd() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(1.5));
    args.extend_from_slice(&double_slots(2.5));
    invoke_wide_double("dadd", "(DD)D", &args, 4.0);
}

#[test]
fn wide_dsub() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(5.0));
    args.extend_from_slice(&double_slots(1.25));
    invoke_wide_double("dsub", "(DD)D", &args, 3.75);
}

#[test]
fn wide_dmul() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(3.0));
    args.extend_from_slice(&double_slots(4.0));
    invoke_wide_double("dmul", "(DD)D", &args, 12.0);
}

#[test]
fn wide_ddiv() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(7.0));
    args.extend_from_slice(&double_slots(2.0));
    invoke_wide_double("ddiv", "(DD)D", &args, 3.5);
}

#[test]
fn wide_dneg() {
    invoke_wide_double("dneg", "(D)D", &double_slots(3.5), -3.5);
}

// 比较：lcmp / fcmp / dcmp 驱动分支
#[test]
fn wide_lcmp_lt() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(5));
    args.extend_from_slice(&long_slots(10));
    // lcmp(a,b) == -1，ifge 10 不跳，走 ldc -1
    invoke_wide_long("lcmp", "(JJ)J", &args, -1);
}

#[test]
fn wide_lcmp_eq() {
    let mut args = Vec::new();
    args.extend_from_slice(&long_slots(7));
    args.extend_from_slice(&long_slots(7));
    invoke_wide_long("lcmp", "(JJ)J", &args, 0);
}

#[test]
fn wide_fbranch_gt() {
    // fbranch(5.0, 3.0) → fcmpl 后 ifle 不跳 → iconst_1
    invoke_wide_int("fbranch", "(FF)I", &[float_slot(5.0), float_slot(3.0)], 1);
}

#[test]
fn wide_fbranch_lt() {
    // fbranch(3.0, 5.0) → fcmpl ifle 跳；fcmpg ifge 不跳 → iconst_m1
    invoke_wide_int("fbranch", "(FF)I", &[float_slot(3.0), float_slot(5.0)], -1);
}

#[test]
fn wide_dbranch_gt() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(5.0));
    args.extend_from_slice(&double_slots(3.0));
    invoke_wide_int("dbranch", "(DD)I", &args, 1);
}

// 类型转换：取其中几条作代表
#[test]
fn wide_i2l() {
    invoke_wide_long("i2l", "(I)J", &[123456], 123456);
}

#[test]
fn wide_l2i() {
    invoke_wide_int(
        "l2i",
        "(J)I",
        &long_slots(1234567890123i64),
        1234567890123i64 as i32,
    );
}

#[test]
fn wide_i2f() {
    invoke_wide_float("i2f", "(I)F", &[42], 42.0);
}

#[test]
fn wide_i2d() {
    invoke_wide_double("i2d", "(I)D", &[42], 42.0);
}

#[test]
fn wide_f2i() {
    invoke_wide_int("f2i", "(F)I", &[float_slot(3.7)], 3);
}

#[test]
fn wide_d2i() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(-3.7));
    invoke_wide_int("d2i", "(D)I", &args, -3);
}

#[test]
fn wide_d2f() {
    let mut args = Vec::new();
    args.extend_from_slice(&double_slots(3.5));
    invoke_wide_float("d2f", "(D)F", &args, 3.5);
}
