use crate::{
    runtime::ms_api::MsRef,
    code::code::Code,
    engines::exec_error::{ExecError, ExecErrorKind, ExecResult},
    oops::{jvalue::JValue, method::Method, normal_klass::NormalKlass}
};

#[derive(Debug, Clone)]
pub struct Invocation {
    owner: MsRef<NormalKlass>,
    method: MsRef<Method>,
    
    args: Box<[JValue]>,
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
                owner: owner.name().utf8().into(),
                name: mname.into(),
                desc: desc.into(),
            }))
        };

        if method.code().is_none() {
            return Err(ExecError::new(
                ExecErrorKind::NoCode {
                    owner: owner.name().utf8().into(),
                    name: mname.into(),
                    desc: desc.into(),
                },
            ));
        }

        Ok(Self {
            owner,
            method,
            args: args.into(),
        })
    }
}

impl Invocation {
    pub fn owner(&self) -> &NormalKlass {
        &self.owner
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn args(&self) -> &[JValue] {
        &self.args
    }

    pub fn code(&self) -> &Code {
        self.method.code().unwrap()
    }
}
