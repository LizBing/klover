use std::marker::PhantomData;

use crate::{
    runtime::ms_api::MsRef,
    code::code::Code,
    engines::exec_error::{ExecError, ExecErrorKind, ExecResult},
    oops::{jvalue::JValue, method::Method, normal_klass::NormalKlass}
};

#[derive(Debug, Clone)]
pub struct Invocation {
    __: PhantomData<()>,
    
    pub owner: MsRef<NormalKlass>,
    pub method: MsRef<Method>,
    
    pub args: Box<[JValue]>,
}

impl Invocation {
    pub fn try_new(
        owner: MsRef<NormalKlass>,
        mname: &str,
        desc: &str,
        args: &[JValue]
    ) -> ExecResult<Self>
    {
        let Some(method) = owner.find_declared_method(mname, desc) else {
            return Err(ExecError::new(ExecErrorKind::MethodNotFound {
                owner: owner.name.utf8().into(),
                name: mname.into(),
                desc: desc.into(),
            }))
        };

        if method.code.is_none() {
            return Err(ExecError::new(
                ExecErrorKind::NoCode {
                    owner: owner.name.utf8().into(),
                    name: mname.into(),
                    desc: desc.into(),
                },
            ));
        }

        Ok(Self {
            __: PhantomData,
            owner,
            method,
            args: args.into(),
        })
    }
}

impl Invocation {
    pub fn code(&self) -> &Code {
        self.method.code.as_ref().unwrap()
    }
}
