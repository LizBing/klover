use crate::{
    engines::exec_error::{ExecError, ExecErrorKind, ExecResult},
    oops::{
        jvalue::JValue,
        method::{Method, MethodAccessFlags},
        normal_klass::NormalKlass,
    },
};

/// A selected method and the class that declares it.
/// Private fields prevent pairing a method with an unrelated class.
#[derive(Debug, Clone, Copy)]
pub struct CallTarget<'exec, 'meta: 'exec> {
    pub declaring_class: &'exec NormalKlass<'meta>,
    pub method: &'exec Method<'meta>,
}

impl<'exec, 'meta: 'exec> CallTarget<'exec, 'meta> {
    /// Looks up a declaration only; this does not perform JVM method resolution.
    pub fn lookup_declared(
        declaring_class: &'exec NormalKlass<'meta>,
        name: &str,
        desc: &str,
    ) -> ExecResult<Self> {
        let method = declaring_class
            .find_declared_method(name, desc)
            .ok_or_else(|| {
                ExecError::new(ExecErrorKind::MethodNotFound {
                    owner: declaring_class.name().utf8().into(),
                    name: name.into(),
                    desc: desc.into(),
                })
            })?;
        Ok(Self {
            declaring_class,
            method,
        })
    }
}

/// A validated call request, consumed when an execution frame is created.
/// Metadata outlives the execution borrow; arguments are owned by the request.
#[derive(Debug)]
pub struct Invocation<'exec, 'meta: 'exec> {
    target: CallTarget<'exec, 'meta>,
    /// Instance calls include the receiver at index zero. Wide values count once.
    args: Box<[JValue]>,
}

impl<'exec, 'meta: 'exec> Invocation<'exec, 'meta> {
    /// Validates argument count and value kinds against the selected method.
    /// Reference checks, including the receiver, do not inspect the referenced object.
    /// Bytecode availability and local-slot capacity are checked by the interpreter.
    pub fn try_new(target: CallTarget<'exec, 'meta>, args: Box<[JValue]>) -> ExecResult<Self> {
        let method = target.method;
        let receiver_count = usize::from(!method.acc_flags.contains(MethodAccessFlags::STATIC));
        let expected = method.desc.args().len() + receiver_count;
        if args.len() != expected {
            return Err(ExecError::with_target(
                &target,
                None,
                ExecErrorKind::ArgumentCountMismatch {
                    expected,
                    actual: args.len(),
                },
            ));
        }
        if receiver_count == 1 && !matches!(args[0], JValue::Ref(_)) {
            return Err(ExecError::with_target(
                &target,
                None,
                ExecErrorKind::ArgumentTypeMismatch {
                    index: 0,
                    expected: format!("L{};", target.declaring_class.name().utf8()),
                    actual: args[0],
                },
            ));
        }
        for (parameter, (desc, arg)) in method
            .desc
            .args()
            .iter()
            .zip(&args[receiver_count..])
            .enumerate()
        {
            if !desc.matches_value(arg) {
                return Err(ExecError::with_target(
                    &target,
                    None,
                    ExecErrorKind::ArgumentTypeMismatch {
                        index: parameter + receiver_count,
                        expected: desc.raw().utf8().into(),
                        actual: *arg,
                    },
                ));
            }
        }
        Ok(Self { target, args })
    }

    /// Convenience entry point for callers that have a method name and borrowed arguments.
    pub fn lookup_declared(
        owner: &'exec NormalKlass<'meta>,
        name: &str,
        desc: &str,
        args: &[JValue],
    ) -> ExecResult<Self> {
        Self::try_new(CallTarget::lookup_declared(owner, name, desc)?, args.into())
    }

    pub(super) fn into_parts(self) -> (CallTarget<'exec, 'meta>, Box<[JValue]>) {
        (self.target, self.args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        class_loader::class_loader_data::ClassLoaderData,
        gc_bindings::oop_hierarchy::NObjPtr,
        oops::normal_klass::UnlinkedNormalKlass,
        runtime::ms_api::{self, MsBox},
    };

    // A minimal classfile exercises the normal descriptor and method builders.
    fn owner<'a>(
        cld: &'a ClassLoaderData,
        desc: &str,
        is_static: bool,
        has_code: bool,
    ) -> NormalKlass<'a> {
        fn u2(bytes: &mut Vec<u8>, value: u16) {
            bytes.extend_from_slice(&value.to_be_bytes());
        }

        let mut bytes = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
        u2(&mut bytes, 6); // Constant pool count.
        bytes.push(1); // UTF8: class name (#1).
        u2(&mut bytes, 16);
        bytes.extend_from_slice(b"java/lang/Object");
        bytes.extend_from_slice(&[7, 0, 1]); // Class (#2).
        for value in ["test", desc, "Code"] {
            bytes.push(1);
            u2(&mut bytes, value.len() as u16);
            bytes.extend_from_slice(value.as_bytes());
        }
        for value in [0x0021, 2, 0, 0, 0, 1] {
            // Class header; one method.
            u2(&mut bytes, value);
        }
        for value in [
            (if is_static { 0x0009 } else { 0x0001 }) | (if has_code { 0 } else { 0x0100 }),
            3,
            4,
            if has_code { 1 } else { 0 },
        ] {
            u2(&mut bytes, value);
        }
        if has_code {
            u2(&mut bytes, 5); // Code attribute.
            bytes.extend_from_slice(&13u32.to_be_bytes());
            u2(&mut bytes, 0); // max_stack.
            u2(&mut bytes, 32); // max_locals, including wide parameters.
            bytes.extend_from_slice(&1u32.to_be_bytes());
            bytes.push(0xb1); // return.
            for _ in 0..2 {
                // Exception table and nested Code attributes.
                u2(&mut bytes, 0);
            }
        }
        u2(&mut bytes, 0); // class attributes.

        let cf = crate::class_parser::parse_class(&bytes).unwrap();
        let interfaces = unsafe { MsBox::new_uninit_slice(cld.ms_allocator(), 0).assume_init() };
        NormalKlass::link(UnlinkedNormalKlass::build(cf, cld), None, interfaces).unwrap()
    }

    #[test]
    fn validates_invocation_arguments() {
        // The shared metaspace can only be initialized once per process.
        let initialized = ms_api::initialize();
        assert!(
            matches!(
                initialized,
                Ok(()) | Err(ms_api::MsInitError::AlreadyInitialized)
            ),
            "{initialized:?}"
        );
        validates_each_parameter_kind_including_arrays();
        checks_argument_count_and_counts_wide_values_once();
        instance_arguments_include_a_reference_receiver();
        preserves_method_lookup_and_code_errors();
    }

    fn validates_each_parameter_kind_including_arrays() {
        let cld = ClassLoaderData::new();
        let values = [
            JValue::Boolean(1),
            JValue::Byte(-1),
            JValue::Char(65),
            JValue::Short(-2),
            JValue::Int(3),
            JValue::Long(4),
            JValue::Float(5.0),
            JValue::Double(6.0),
            JValue::Ref(NObjPtr::null()),
        ];
        for (descriptor, matching_index) in [
            ("Z", 0),
            ("B", 1),
            ("C", 2),
            ("S", 3),
            ("I", 4),
            ("J", 5),
            ("F", 6),
            ("D", 7),
            ("Ljava/lang/Object;", 8),
            ("[I", 8),
            ("[[D", 8),
            ("[Ljava/lang/Object;", 8),
        ] {
            let desc = format!("({descriptor})V");
            let klass = owner(&cld, &desc, true, true);
            for (index, value) in values.iter().enumerate() {
                let result = Invocation::lookup_declared(&klass, "test", &desc, &[*value]);
                if index == matching_index {
                    assert!(result.is_ok(), "{descriptor}: {value:?}");
                } else {
                    let error = result.unwrap_err();
                    assert!(
                        matches!(error.kind,
                            ExecErrorKind::ArgumentTypeMismatch { index: 0, ref expected, .. }
                                if expected == descriptor
                        ),
                        "{descriptor}: {value:?}: {error:?}"
                    );
                    let brief = error.brief.unwrap();
                    assert_eq!(brief.klass, "java/lang/Object");
                    assert_eq!(brief.method, "test");
                    assert_eq!(brief.desc, desc);
                }
            }
        }
    }

    fn checks_argument_count_and_counts_wide_values_once() {
        let cld = ClassLoaderData::new();
        let klass = owner(&cld, "(JD)V", true, true);
        let values = [JValue::Long(1), JValue::Double(2.0), JValue::Int(3)];
        for actual in [0, 1, 3] {
            let error = Invocation::lookup_declared(&klass, "test", "(JD)V", &values[..actual])
                .unwrap_err();
            assert!(matches!(error.kind,
                ExecErrorKind::ArgumentCountMismatch { expected: 2, actual: count } if count == actual
            ));
        }
        let invocation =
            Invocation::lookup_declared(&klass, "test", "(JD)V", &values[..2]).unwrap();
        assert!(matches!(
            &*invocation.args,
            [JValue::Long(1), JValue::Double(2.0)]
        ));

        let klass = owner(&cld, "()V", true, true);
        assert!(Invocation::lookup_declared(&klass, "test", "()V", &[]).is_ok());
        assert!(matches!(
            Invocation::lookup_declared(&klass, "test", "()V", &values[..1])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentCountMismatch {
                expected: 0,
                actual: 1
            }
        ));
    }

    fn instance_arguments_include_a_reference_receiver() {
        let cld = ClassLoaderData::new();
        let klass = owner(&cld, "(I)V", false, true);
        assert!(matches!(
            Invocation::lookup_declared(&klass, "test", "(I)V", &[JValue::Int(1)])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentCountMismatch {
                expected: 2,
                actual: 1
            }
        ));
        assert!(matches!(
            Invocation::lookup_declared(&klass, "test", "(I)V", &[JValue::Int(0), JValue::Int(1)])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentTypeMismatch { index: 0, .. }
        ));
        let receiver = JValue::Ref(NObjPtr::null());
        assert!(
            Invocation::lookup_declared(&klass, "test", "(I)V", &[receiver, JValue::Int(1)])
                .is_ok()
        );
        assert!(matches!(
            Invocation::lookup_declared(&klass, "test", "(I)V", &[receiver, JValue::Float(1.0)])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentTypeMismatch { index: 1, .. }
        ));
        let klass = owner(&cld, "()V", false, true);
        assert!(matches!(
            Invocation::lookup_declared(&klass, "test", "()V", &[])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentCountMismatch {
                expected: 1,
                actual: 0
            }
        ));
        assert!(Invocation::lookup_declared(&klass, "test", "()V", &[receiver]).is_ok());
    }

    fn preserves_method_lookup_and_code_errors() {
        let cld = ClassLoaderData::new();
        let klass = owner(&cld, "()V", true, false);
        assert!(matches!(
            Invocation::lookup_declared(&klass, "missing", "()V", &[])
                .unwrap_err()
                .kind,
            ExecErrorKind::MethodNotFound { .. }
        ));
        assert!(matches!(
            crate::engines::interpreter::interpreter_frame::InterpreterFrame::try_new(
                Invocation::lookup_declared(&klass, "test", "()V", &[]).unwrap()
            )
            .err()
            .unwrap()
            .kind,
            ExecErrorKind::NoCode { .. }
        ));
    }
    #[test]
    fn selected_target_accepts_owned_arguments_without_copying() {
        crate::tests::support::ensure_metaspace();
        let cld = ClassLoaderData::new();
        let klass = owner(&cld, "(I)V", true, true);
        let target = CallTarget::lookup_declared(&klass, "test", "(I)V").unwrap();
        assert!(std::ptr::eq(target.declaring_class, &klass));
        assert!(std::ptr::eq(
            target.method,
            klass.find_declared_method("test", "(I)V").unwrap()
        ));
        let args: Box<[JValue]> = vec![JValue::Int(7)].into_boxed_slice();
        let address = args.as_ptr();
        let call = Invocation::try_new(target, args).unwrap();
        let (_, args) = call.into_parts();
        assert_eq!(args.as_ptr(), address);
        assert!(matches!(&*args, [JValue::Int(7)]));
    }
}
