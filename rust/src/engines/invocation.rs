use std::marker::PhantomData;

use crate::{class_loader::ms_api::MSRef, code::code::Code, engines::exec_error::{ExecError, ExecErrorKind, ExecResult}, oops::{jvalue::JValue, method::Method, normal_klass::NormalKlass}};

pub struct Invocation {
    __: PhantomData<()>,
    
    pub owner: MSRef<NormalKlass>,
    pub method: MSRef<Method>,
    
    pub args: Box<[JValue]>,
}

impl Invocation {
    pub fn try_new(owner: MSRef<NormalKlass>, mname: &str, desc: &str, args: &[JValue]) -> ExecResult<Self> {
        let Some(method) = owner.find_declared_method(mname, desc) else {
            return Err(ExecError::new(
                owner.name.utf8(),
                mname,
                desc,
                ExecErrorKind::MethodNotFound
            ));
        };

        if method.code.is_none() {
            return Err(ExecError::new(
                owner.name.utf8(),
                mname,
                desc,
                ExecErrorKind::NoCode,
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
