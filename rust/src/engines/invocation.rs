use crate::{
    code::code::Code,
    engines::exec_error::{ExecError, ExecErrorKind, ExecResult},
    oops::{jvalue::JValue, method::Method, normal_klass::NormalKlass}
};

#[derive(Debug, Clone)]
pub struct Invocation<'klass> {
    owner: &'klass NormalKlass<'klass>,
    method: &'klass Method<'klass>,
    
    args: Box<[JValue]>,
}

impl<'klass> Invocation<'klass> {
    pub fn try_new(
        owner: &'klass NormalKlass<'klass>,
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

impl<'klass> Invocation<'klass> {
    pub fn owner(&self) -> &NormalKlass<'klass> {
        &self.owner
    }

    pub fn method(&self) -> &Method<'klass> {
        &self.method
    }

    pub fn args(&self) -> &[JValue] {
        &self.args
    }

    pub fn code(&self) -> &Code<'klass> {
        self.method.code().unwrap()
    }
}
