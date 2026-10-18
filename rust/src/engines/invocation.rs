use crate::{
    code::code::Code,
    engines::exec_error::{ExecError, ExecErrorKind, ExecResult},
    oops::{jvalue::JValue, method::Method, normal_klass::NormalKlass}
};

#[derive(Debug, Clone)]
pub struct Invocation<'metadata> {
    owner: &'metadata NormalKlass<'metadata>,
    method: &'metadata Method<'metadata>,
    
    args: Box<[JValue]>,
}

impl<'metadata> Invocation<'metadata> {
    pub fn try_new(
        owner: &'metadata NormalKlass<'metadata>,
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

        // todo: verify args

        Ok(Self {
            owner,
            method,
            args: args.into(),
        })
    }
}

impl<'metadata> Invocation<'metadata> {
    pub fn owner(&self) -> &NormalKlass<'metadata> {
        &self.owner
    }

    pub fn method(&self) -> &Method<'metadata> {
        &self.method
    }

    pub fn args(&self) -> &[JValue] {
        &self.args
    }

    pub fn code(&self) -> &Code<'metadata> {
        self.method.code().unwrap()
    }
}
