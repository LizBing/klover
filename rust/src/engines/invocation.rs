use crate::{
    code::code::Code,
    engines::exec_error::{ExecError, ExecErrorKind, ExecResult},
    oops::{
        jvalue::JValue,
        method::{Method, MethodAccessFlags},
        normal_klass::NormalKlass,
    },
};

#[derive(Debug, Clone)]
pub struct Invocation<'metadata> {
    _private: (),
    
    pub owner: &'metadata NormalKlass<'metadata>,
    pub method: &'metadata Method<'metadata>,

    /// Instance calls include the receiver at index zero.
    pub args: Box<[JValue]>,
}

impl<'metadata> Invocation<'metadata> {
    /// Validates argument count and value kinds against the method descriptor.
    /// Reference checks, including the receiver, do not inspect the referenced object.
    pub fn try_new(
        owner: &'metadata NormalKlass<'metadata>,
        mname: &str,
        desc: &str,
        args: &[JValue],
    ) -> ExecResult<Self> {
        let Some(method) = owner.find_declared_method(mname, desc) else {
            return Err(ExecError::new(ExecErrorKind::MethodNotFound {
                owner: owner.name().utf8().into(),
                name: mname.into(),
                desc: desc.into(),
            }));
        };

        if method.code.is_none() {
            return Err(ExecError::new(ExecErrorKind::NoCode {
                owner: owner.name().utf8().into(),
                name: mname.into(),
                desc: desc.into(),
            }));
        }

        let invocation = Self {
            _private: (),
            owner,
            method,
            args: args.into(),
        };

        let receiver_count = usize::from(!method.acc_flags.contains(MethodAccessFlags::STATIC));
        let expected = method.desc.args().len() + receiver_count;
        if args.len() != expected {
            return Err(ExecError::with_invocation(
                invocation,
                ExecErrorKind::ArgumentCountMismatch {
                    expected,
                    actual: args.len(),
                },
            ));
        }

        if receiver_count == 1 && !matches!(args[0], JValue::Ref(_)) {
            return Err(ExecError::with_invocation(
                invocation,
                ExecErrorKind::ArgumentTypeMismatch {
                    index: 0,
                    expected: format!("L{};", owner.name().utf8()),
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
                return Err(ExecError::with_invocation(
                    invocation,
                    ExecErrorKind::ArgumentTypeMismatch {
                        index: parameter + receiver_count,
                        expected: desc.raw().utf8().into(),
                        actual: *arg,
                    },
                ));
            }
        }

        Ok(invocation)
    }
}

impl<'metadata> Invocation<'metadata> {
    pub fn code(&self) -> &Code<'metadata> {
        self.method.code.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        class_loader::class_loader_data::ClassLoaderData,
        gc_bindings::oop_hierarchy::NObjPtr,
        oops::normal_klass::UnlinkedNormalKlass,
        runtime::ms_api::{self, MsAllocator},
    };

    // A minimal classfile exercises the normal descriptor and method builders.
    fn owner<'a>(
        cld: &'a ClassLoaderData<'a>,
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
        for value in [if is_static { 0x0009 } else { 0x0001 }, 3, 4, 1] {
            u2(&mut bytes, value);
        }
        u2(&mut bytes, 5); // Code attribute.
        bytes.extend_from_slice(&13u32.to_be_bytes());
        u2(&mut bytes, 0); // max_stack.
        u2(&mut bytes, 32); // max_locals, including wide parameters.
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.push(0xb1); // return.
        for _ in 0..3 {
            // Exception table, Code attributes, class attributes.
            u2(&mut bytes, 0);
        }

        let mut options = cafebabe::ParseOptions::default();
        options.parse_bytecode(true);
        let mut cf = cafebabe::parse_class_with_options(&bytes, &options).unwrap();
        if !has_code {
            cf.methods[0].attributes.clear();
        }
        NormalKlass::link(UnlinkedNormalKlass::build(cf, Some(cld)), None).unwrap()
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
        let msa = MsAllocator::new();
        let cld = ClassLoaderData::new(&msa);
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
                let result = Invocation::try_new(&klass, "test", &desc, &[*value]);
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
                    assert_eq!(brief.args.len(), 1);
                }
            }
        }
    }

    fn checks_argument_count_and_counts_wide_values_once() {
        let msa = MsAllocator::new();
        let cld = ClassLoaderData::new(&msa);
        let klass = owner(&cld, "(JD)V", true, true);
        let values = [JValue::Long(1), JValue::Double(2.0), JValue::Int(3)];
        for actual in [0, 1, 3] {
            let error =
                Invocation::try_new(&klass, "test", "(JD)V", &values[..actual]).unwrap_err();
            assert!(matches!(error.kind,
                ExecErrorKind::ArgumentCountMismatch { expected: 2, actual: count } if count == actual
            ));
        }
        let invocation = Invocation::try_new(&klass, "test", "(JD)V", &values[..2]).unwrap();
        assert!(matches!(
            &*invocation.args,
            [JValue::Long(1), JValue::Double(2.0)]
        ));

        let klass = owner(&cld, "()V", true, true);
        assert!(Invocation::try_new(&klass, "test", "()V", &[]).is_ok());
        assert!(matches!(
            Invocation::try_new(&klass, "test", "()V", &values[..1])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentCountMismatch {
                expected: 0,
                actual: 1
            }
        ));
    }

    fn instance_arguments_include_a_reference_receiver() {
        let msa = MsAllocator::new();
        let cld = ClassLoaderData::new(&msa);
        let klass = owner(&cld, "(I)V", false, true);
        assert!(matches!(
            Invocation::try_new(&klass, "test", "(I)V", &[JValue::Int(1)])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentCountMismatch {
                expected: 2,
                actual: 1
            }
        ));
        assert!(matches!(
            Invocation::try_new(&klass, "test", "(I)V", &[JValue::Int(0), JValue::Int(1)])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentTypeMismatch { index: 0, .. }
        ));
        let receiver = JValue::Ref(NObjPtr::null());
        assert!(Invocation::try_new(&klass, "test", "(I)V", &[receiver, JValue::Int(1)]).is_ok());
        assert!(matches!(
            Invocation::try_new(&klass, "test", "(I)V", &[receiver, JValue::Float(1.0)])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentTypeMismatch { index: 1, .. }
        ));
        let klass = owner(&cld, "()V", false, true);
        assert!(matches!(
            Invocation::try_new(&klass, "test", "()V", &[])
                .unwrap_err()
                .kind,
            ExecErrorKind::ArgumentCountMismatch {
                expected: 1,
                actual: 0
            }
        ));
        assert!(Invocation::try_new(&klass, "test", "()V", &[receiver]).is_ok());
    }

    fn preserves_method_lookup_and_code_errors() {
        let msa = MsAllocator::new();
        let cld = ClassLoaderData::new(&msa);
        let klass = owner(&cld, "()V", true, false);
        assert!(matches!(
            Invocation::try_new(&klass, "missing", "()V", &[])
                .unwrap_err()
                .kind,
            ExecErrorKind::MethodNotFound { .. }
        ));
        assert!(matches!(
            Invocation::try_new(&klass, "test", "()V", &[JValue::Int(1)])
                .unwrap_err()
                .kind,
            ExecErrorKind::NoCode { .. }
        ));
    }
}
