//! 端到端：通过解释器执行一个纯算术的 Java 静态方法。
//!
//! 前置：`test_data/classes/SimpleAddition.class`（已纳入仓库）与
//! `test_data/classes/java/lang/Object.class`（由测试 setup 编译产生）。
//!
//! `SimpleAddition.add(II)I` 的字节码为：
//!   iload_0  iload_1  iadd  ireturn

use std::ptr::NonNull;

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

fn find_static_method(
    klass: &NormalKlass,
    name: &str,
    desc: &str,
) -> Option<NonNull<klover::oops::method::Method>> {
    let n = SymbolTable::intern(name);
    let d = SymbolTable::intern(desc);
    klass.find_method(&n, &d).map(NonNull::from)
}

#[test]
fn invoke_static_add_two_ints() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("SimpleAddition").expect("load SimpleAddition");
    let normal = unsafe { klass.as_ref().as_normal().unwrap() };
    let method = find_static_method(normal, "add", "(II)I").expect("find add");

    let mut interp = Interpreter::new(1024);
    let ret = interp
        .invoke_static(NonNull::from(normal), method, &[3, 4])
        .expect("invoke_static");

    match ret {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 7),
        _ => panic!("expected Int(7)"),
    }
}

/// 覆盖更多算术 / 常量加载指令，避免只有 iadd 被验证。
#[test]
fn invoke_static_arith_suite() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Arith").expect("load Arith");
    let normal = unsafe { klass.as_ref().as_normal().unwrap() };
    let mut interp = Interpreter::new(1024);

    // mul2(21) -> 42（imul + istore/ iload 路径）
    let m = find_static_method(normal, "mul2", "(I)I").expect("mul2");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[21])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(42))));

    // neg(5) -> -5（ineg）
    let m = find_static_method(normal, "neg", "(I)I").expect("neg");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[5])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(-5))));

    // sub(10, 3) -> 7（isub）
    let m = find_static_method(normal, "sub", "(II)I").expect("sub");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[10, 3])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(7))));

    // rem(17, 5) -> 2（irem）
    let m = find_static_method(normal, "rem", "(II)I").expect("rem");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[17, 5])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(2))));

    // addConst() -> 15（iinc x 2）
    let m = find_static_method(normal, "addConst", "()I").expect("addConst");
    let r = interp.invoke_static(NonNull::from(normal), m, &[]).unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(15))));

    // pushByte() -> 100（bipush）
    let m = find_static_method(normal, "pushByte", "()I").expect("pushByte");
    let r = interp.invoke_static(NonNull::from(normal), m, &[]).unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(100))));

    // pushShort() -> 10000（sipush）
    let m = find_static_method(normal, "pushShort", "()I").expect("pushShort");
    let r = interp.invoke_static(NonNull::from(normal), m, &[]).unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(10000))));
}

/// 控制流：while / for / do-while / if-else / goto + break。
#[test]
fn invoke_static_control_flow() {
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("ControlFlow").expect("load ControlFlow");
    let normal = unsafe { klass.as_ref().as_normal().unwrap() };
    let mut interp = Interpreter::new(1024);

    // sum(100) = 5050（while + if_icmpgt + goto）
    let m = find_static_method(normal, "sum", "(I)I").expect("sum");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[100])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(5050))));

    // max(3, 7) = 7；max(9, 2) = 9（if_icmplt 分支）
    let m = find_static_method(normal, "max", "(II)I").expect("max");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[3, 7])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(7))));
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[9, 2])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(9))));

    // countTo(5) = 5（for + if_icmpge + goto）
    let m = find_static_method(normal, "countTo", "(I)I").expect("countTo");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[5])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(5))));

    // factorial(5) = 120（do-while + if_icmple）
    let m = find_static_method(normal, "factorial", "(I)I").expect("factorial");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[5])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(120))));

    // firstEven(7) = 8（无限循环 goto + ifne + return）
    let m = find_static_method(normal, "firstEven", "(I)I").expect("firstEven");
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[7])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(8))));
    // firstEven(2) = 2（第一个就是偶数）
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[2])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(2))));
}

/// long / float / double 算术、类型转换、位运算、比较。
#[test]
fn invoke_static_wide_arith() {
    use klover::interpreter::interpreter::ReturnValue;
    let classes = concat!(env!("CARGO_MANIFEST_DIR"), "/../test_data/classes");
    init_runtime(classes);

    let klass = BootstrapCLD::find_class("Wide").expect("load Wide");
    let normal = unsafe { klass.as_ref().as_normal().unwrap() };
    let mut interp = Interpreter::new(2048);

    // ---- long 算术 ----
    // ladd(1000000, 2345678) = 3345678
    let m = find_static_method(normal, "ladd", "(JJ)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 1000000, 0, 2345678])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 3345678),
        _ => panic!("expected Long"),
    }

    // lsub(5000000000, 1) = 4999999999（超过 i32 范围，验证 long 正确性）
    let m = find_static_method(normal, "lsub", "(JJ)J").unwrap();
    // 5000000000 = 0x12A05F200，高32=1，低32=0x2A05F200
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[1, 0x2A05F200, 0, 1])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 4999999999),
        _ => panic!("expected Long"),
    }

    // lmul(1000000, 1000000) = 1000000000000
    let m = find_static_method(normal, "lmul", "(JJ)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 1000000, 0, 1000000])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 1000000000000),
        _ => panic!("expected Long"),
    }

    // ldiv(1000000000, 7) = 142857142
    let m = find_static_method(normal, "ldiv", "(JJ)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 1000000000, 0, 7])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 142857142),
        _ => panic!("expected Long"),
    }

    // lrem(1000000000, 7) = 6
    let m = find_static_method(normal, "lrem", "(JJ)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 1000000000, 0, 7])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 6),
        _ => panic!("expected Long"),
    }

    // lneg(5) = -5
    let m = find_static_method(normal, "lneg", "(J)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 5])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, -5),
        _ => panic!("expected Long"),
    }

    // ---- float 算术 ----
    let m = find_static_method(normal, "fadd", "(FF)F").unwrap();
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[1.5f32.to_bits() as i32, 2.25f32.to_bits() as i32],
        )
        .unwrap();
    match r {
        Some(ReturnValue::Float(v)) => assert_eq!(v, 3.75),
        _ => panic!("expected Float"),
    }

    let m = find_static_method(normal, "fmul", "(FF)F").unwrap();
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[3.0f32.to_bits() as i32, 4.0f32.to_bits() as i32],
        )
        .unwrap();
    match r {
        Some(ReturnValue::Float(v)) => assert_eq!(v, 12.0),
        _ => panic!("expected Float"),
    }

    let m = find_static_method(normal, "fdiv", "(FF)F").unwrap();
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[10.0f32.to_bits() as i32, 4.0f32.to_bits() as i32],
        )
        .unwrap();
    match r {
        Some(ReturnValue::Float(v)) => assert_eq!(v, 2.5),
        _ => panic!("expected Float"),
    }

    let m = find_static_method(normal, "fneg", "(F)F").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[3.5f32.to_bits() as i32])
        .unwrap();
    match r {
        Some(ReturnValue::Float(v)) => assert_eq!(v, -3.5),
        _ => panic!("expected Float"),
    }

    // ---- double 算术 ----
    let m = find_static_method(normal, "dadd", "(DD)D").unwrap();
    let a = 1.5f64.to_bits() as i64;
    let b = 2.25f64.to_bits() as i64;
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[(a >> 32) as i32, a as i32, (b >> 32) as i32, b as i32],
        )
        .unwrap();
    match r {
        Some(ReturnValue::Double(v)) => assert_eq!(v, 3.75),
        _ => panic!("expected Double"),
    }

    let m = find_static_method(normal, "dmul", "(DD)D").unwrap();
    let a = 1e10f64.to_bits() as i64;
    let b = 2.0f64.to_bits() as i64;
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[(a >> 32) as i32, a as i32, (b >> 32) as i32, b as i32],
        )
        .unwrap();
    match r {
        Some(ReturnValue::Double(v)) => assert_eq!(v, 2e10),
        _ => panic!("expected Double"),
    }

    // ---- 比较 ----
    // lcmp 包装的 Java 方法：a<b 返回 -1，a>b 返回 1，相等返回 0
    let m = find_static_method(normal, "lcmp", "(JJ)J").unwrap();
    // a=5,b=10 -> -1
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 5, 0, 10])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, -1),
        _ => panic!("expected Long"),
    }
    // a=10,b=5 -> 1
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 10, 0, 5])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 1),
        _ => panic!("expected Long"),
    }
    // a=7,b=7 -> 0
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 7, 0, 7])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 0),
        _ => panic!("expected Long"),
    }

    // fbranch(3.0, 2.0) = 1；fbranch(2.0, 3.0) = -1；fbranch(2.0, 2.0) = 0
    let m = find_static_method(normal, "fbranch", "(FF)I").unwrap();
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[3.0f32.to_bits() as i32, 2.0f32.to_bits() as i32],
        )
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(1))));
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[2.0f32.to_bits() as i32, 3.0f32.to_bits() as i32],
        )
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(-1))));
    let r = interp
        .invoke_static(
            NonNull::from(normal),
            m,
            &[2.0f32.to_bits() as i32, 2.0f32.to_bits() as i32],
        )
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(0))));

    // ---- 类型转换 ----
    // i2l(1234567) = 1234567L
    let m = find_static_method(normal, "i2l", "(I)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[1234567])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 1234567),
        _ => panic!("expected Long"),
    }

    // l2i(0x1_00000001) = 1（只保留低32位）
    let m = find_static_method(normal, "l2i", "(J)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[1, 1])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(1))));

    // i2f(5) = 5.0
    let m = find_static_method(normal, "i2f", "(I)F").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[5])
        .unwrap();
    match r {
        Some(ReturnValue::Float(v)) => assert_eq!(v, 5.0),
        _ => panic!("expected Float"),
    }

    // i2d(7) = 7.0
    let m = find_static_method(normal, "i2d", "(I)D").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[7])
        .unwrap();
    match r {
        Some(ReturnValue::Double(v)) => assert_eq!(v, 7.0),
        _ => panic!("expected Double"),
    }

    // ---- int 位运算 ----
    let m = find_static_method(normal, "iand", "(II)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0b1100, 0b1010])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(0b1000))));

    let m = find_static_method(normal, "ior", "(II)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0b1100, 0b1010])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(0b1110))));

    let m = find_static_method(normal, "ixor", "(II)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0b1100, 0b1010])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(0b0110))));

    // ishl(1, 4) = 16
    let m = find_static_method(normal, "ishl", "(II)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[1, 4])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(16))));

    // ishr(-16, 2) = -4（算术右移）
    let m = find_static_method(normal, "ishr", "(II)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[-16, 2])
        .unwrap();
    assert!(matches!(r, Some(ReturnValue::Int(-4))));

    // iushr(-16, 2)：逻辑右移。 -16 = 0xFFFFFFF0，>> 2 = 0x3FFFFFFC
    let m = find_static_method(normal, "iushr", "(II)I").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[-16, 2])
        .unwrap();
    match r {
        Some(ReturnValue::Int(v)) => assert_eq!(v, 0x3FFFFFFCu32 as i32),
        _ => panic!("expected Int"),
    }

    // ---- long 位运算 ----
    let m = find_static_method(normal, "land", "(JJ)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 0b1100, 0, 0b1010])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 0b1000),
        _ => panic!("expected Long"),
    }

    // lshl(1L, 40) = 2^40（需验证 long 位移）
    let m = find_static_method(normal, "lshl", "(JI)J").unwrap();
    let r = interp
        .invoke_static(NonNull::from(normal), m, &[0, 1, 40])
        .unwrap();
    match r {
        Some(ReturnValue::Long(v)) => assert_eq!(v, 1i64 << 40),
        _ => panic!("expected Long"),
    }
}
