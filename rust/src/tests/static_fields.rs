use std::sync::Arc;

use crate::tests::support::{
    assert_value, fixture_dir, init_vm, isolated, load_normal_klass, run_method,
};
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    class_parser::{ClassRef, FieldDescriptor, FieldMemRef},
    code::instructions::Instruction,
    engines::{
        exec_error::ExecErrorKind,
        interpreter::{
            instructions::references::{get_static, put_static},
            interpreter_frame::InterpreterFrame,
            slot::Slot,
        },
        invocation::Invocation,
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::{
        constant_entries::FieldRef,
        jvalue::{
            JBoolean, JDouble, JFloat, JInt, JLong,
            JValue::{self, *},
        },
        oops_errors::ResolveError,
        symbol_table::SymbolTable,
    },
};

fn field_ref<'a>(owner: &str, name: &str, desc: &str) -> FieldRef<'a> {
    FieldRef::build(&FieldMemRef {
        owner: Arc::new(ClassRef { name: owner.into() }),
        name: name.into(),
        descriptor: FieldDescriptor::parse(desc).unwrap(),
    })
}

#[test]
fn static_fields_execute_through_lowering_and_dispatch() {
    isolated(
        concat!(
            module_path!(),
            "::static_fields_execute_through_lowering_and_dispatch"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticFields")).unwrap();
            let cases: &[(&str, &str, JValue, JValue)] = &[
                ("bool", "(Z)Z", Boolean(JBoolean(1)), Int(JInt(1))),
                ("bool", "(Z)Z", Boolean(JBoolean(0)), Int(JInt(0))),
                ("narrow", "(I)I", Int(JInt(65535)), Int(JInt(65533))),
                ("integer", "(I)I", Int(JInt(i32::MIN)), Int(JInt(i32::MIN))),
                (
                    "wide",
                    "(J)J",
                    Long(JLong(i64::MIN + 37)),
                    Long(JLong(i64::MIN + 37)),
                ),
                ("floating", "(F)F", Float(JFloat(-0.0)), Float(JFloat(-0.0))),
                (
                    "floating",
                    "(F)F",
                    Float(JFloat(f32::NAN)),
                    Float(JFloat(f32::NAN)),
                ),
                (
                    "precise",
                    "(D)D",
                    Double(JDouble(f64::INFINITY)),
                    Double(JDouble(f64::INFINITY)),
                ),
                (
                    "precise",
                    "(D)D",
                    Double(JDouble(-0.0)),
                    Double(JDouble(-0.0)),
                ),
                (
                    "reference",
                    "(Ljava/lang/Object;)Ljava/lang/Object;",
                    Ref(NObjPtr::null()),
                    Ref(NObjPtr::null()),
                ),
                (
                    "array",
                    "([I)[I",
                    Ref(NObjPtr::null()),
                    Ref(NObjPtr::null()),
                ),
                ("inherited", "(I)I", Int(JInt(917)), Int(JInt(917))),
            ];
            for &(name, desc, input, expected) in cases {
                for fuel in [1, 10000] {
                    assert_value(run_method(owner, name, desc, &[input], fuel), expected);
                }
            }
            let code = owner
                .find_declared_method("integer", "(I)I")
                .unwrap()
                .code
                .as_ref()
                .unwrap();
            let refs: Vec<_> = code
                .insts
                .iter()
                .filter_map(|inst| match inst {
                    Instruction::GetStatic(r) | Instruction::PutStatic(r) => Some(*r),
                    _ => None,
                })
                .collect();
            assert_eq!(refs.len(), 2);
            assert!(std::ptr::eq(refs[0], refs[1]));
        },
    );
}

#[test]
fn static_field_resolution_and_failures_preserve_storage_and_stack() {
    isolated(
        concat!(
            module_path!(),
            "::static_field_resolution_and_failures_preserve_storage_and_stack"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticFields")).unwrap();
            let caller = load_normal_klass(&cld, &SymbolTable::intern("java/lang/Object")).unwrap();
            let inherited = field_ref("StaticFields$Child", "shared", "I");
            let resolved = inherited.resolve(owner).unwrap();
            assert_eq!(resolved.declaring_klass.name().utf8(), "StaticFields$Base");
            let iface = field_ref("StaticFields$Implements", "TOKEN", "I")
                .resolve(owner)
                .unwrap();
            assert_eq!(iface.declaring_klass.name().utf8(), "StaticFields$Root");
            let precedence = field_ref("StaticFields$Implements", "shared", "I")
                .resolve(owner)
                .unwrap();
            assert_eq!(
                precedence.declaring_klass.name().utf8(),
                "StaticFields$Root"
            );
            assert!(matches!(
                field_ref("StaticFields$Hidden", "value", "I").resolve(caller),
                Err(ResolveError::IllegalClassAccess { .. })
            ));
            let private = field_ref("StaticFields", "hidden", "I");
            private.resolve(owner).unwrap();
            assert!(matches!(
                private.resolve(caller),
                Err(ResolveError::IllegalFieldAccess { .. })
            ));
            assert!(matches!(
                field_ref("StaticFields", "i", "J").resolve(owner),
                Err(ResolveError::NoSuchField { .. })
            ));

            assert_value(
                run_method(owner, "integer", "(I)I", &[Int(JInt(0))], 1),
                Int(JInt(0)),
            );
            let mut frame = InterpreterFrame::try_new(
                Invocation::lookup_declared(owner, "integer", "(I)I", &[Int(JInt(0))]).unwrap(),
            )
            .unwrap();
            frame.push(Slot::Int(JInt(42)));
            assert!(matches!(
                put_static(&mut frame, &field_ref("StaticFields", "instance", "I"))
                    .err()
                    .unwrap()
                    .kind,
                ExecErrorKind::ExpectedStaticField { .. }
            ));
            assert!(matches!(frame.operand_stack(), [Slot::Int(JInt(42))]));
            assert!(matches!(
                put_static(
                    &mut frame,
                    &field_ref("StaticFields$Constants", "CONSTANT", "I")
                )
                .err()
                .unwrap()
                .kind,
                ExecErrorKind::IllegalFinalFieldWrite { .. }
            ));
            for (class, name) in [
                ("StaticFields$Initialized", "value"),
                ("StaticFields$Constants", "CONSTANT"),
            ] {
                assert!(matches!(
                    get_static(&mut frame, &field_ref(class, name, "I")).unwrap_or_else(|e| panic!("{e:?}")),
                    crate::engines::interpreter::step_outcome::StepControl::Initialize(klass)
                        if klass.name().utf8() == class
                ));
            }
            assert!(matches!(frame.operand_stack(), [Slot::Int(JInt(42))]));
            frame.replace_stack_top(1, &[Slot::Float(JFloat(4.0))]);
            assert!(matches!(
                put_static(&mut frame, &field_ref("StaticFields", "i", "I"))
                    .err()
                    .unwrap()
                    .kind,
                ExecErrorKind::MismatchSlotType
            ));
            let int = owner.find_declared_field("i", "I").unwrap();
            assert!(matches!(owner.fields().load_static(int), Int(JInt(0))));
            assert!(matches!(frame.operand_stack(), [Slot::Float(_)]));
            frame.replace_stack_top(1, &[]);
            assert!(matches!(
                put_static(&mut frame, &field_ref("StaticFields", "j", "J"))
                    .err()
                    .unwrap()
                    .kind,
                ExecErrorKind::StackUnderflow
            ));
            for (value, expected) in [(2, 0), (3, 1)] {
                frame.push(Slot::Int(JInt(value)));
                put_static(&mut frame, &field_ref("StaticFields", "z", "Z"))
                    .unwrap_or_else(|e| panic!("{e:?}"));
                get_static(&mut frame, &field_ref("StaticFields", "z", "Z"))
                    .unwrap_or_else(|e| panic!("{e:?}"));
                assert!(matches!(frame.operand_stack(), [Slot::Int(JInt(v))] if *v == expected));
                frame.replace_stack_top(1, &[]);
            }

            // A shared symbolic reference must resolve into the second loader's storage.
            let other = ClassLoaderData::new(None);
            let other_caller =
                load_normal_klass(&other, &SymbolTable::intern("StaticFields")).unwrap();
            let other_resolved = inherited.resolve(other_caller).unwrap();
            assert!(!std::ptr::eq(
                resolved.declaring_klass,
                other_resolved.declaring_klass
            ));

            let subclass = load_normal_klass(&cld, &SymbolTable::intern("FieldAccess")).unwrap();
            for symbolic_owner in ["fieldaccess/Base", "FieldAccess", "FieldAccess$Child"] {
                field_ref(symbolic_owner, "value", "I")
                    .resolve(subclass)
                    .unwrap();
            }
            assert!(matches!(
                field_ref("FieldAccess$Sibling", "value", "I").resolve(subclass),
                Err(ResolveError::IllegalFieldAccess { .. })
            ));
            field_ref("FieldAccess$Sibling", "shared", "I")
                .resolve(subclass)
                .unwrap();
        },
    );
}

#[test]
fn volatile_static_accesses_order_across_declaring_classes() {
    use crate::engines::protocol::MethodReturn;
    use std::{sync::Barrier, thread};

    isolated(
        concat!(
            module_path!(),
            "::volatile_static_accesses_order_across_declaring_classes"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticFields")).unwrap();
            // Initialize both declaring classes before the two workers start.
            assert_value(
                run_method(owner, "writeLeftReadRight", "()I", &[], 100),
                Int(JInt(0)),
            );
            let left = field_ref("StaticFields$Left", "value", "I")
                .resolve(owner)
                .unwrap();
            let right = field_ref("StaticFields$Right", "value", "I")
                .resolve(owner)
                .unwrap();
            const ROUNDS: usize = 256;
            let barrier = Barrier::new(3);

            let both_zero = thread::scope(|scope| {
                let worker = |name| {
                    let mut reads = Vec::with_capacity(ROUNDS);
                    for _ in 0..ROUNDS {
                        barrier.wait();
                        reads.push(run_method(owner, name, "()I", &[], 100));
                        barrier.wait();
                    }
                    reads
                };
                let first = scope.spawn(move || worker("writeLeftReadRight"));
                let second = scope.spawn(move || worker("writeRightReadLeft"));
                for _ in 0..ROUNDS {
                    left.store_static(Int(JInt(0)));
                    right.store_static(Int(JInt(0)));
                    barrier.wait();
                    barrier.wait();
                }
                first
                    .join()
                    .unwrap()
                    .into_iter()
                    .zip(second.join().unwrap())
                    .any(|pair| {
                        matches!(
                            pair,
                            (
                                MethodReturn::Value(Int(JInt(0))),
                                MethodReturn::Value(Int(JInt(0)))
                            )
                        )
                    })
            });
            assert!(!both_zero, "both volatile reads observed the initial value");
        },
    );
}
