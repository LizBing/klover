use std::marker::PhantomData;

use crate::{class_loader::ms_api::MSRef, oops::{cp_entry::ResolvedMethodRef, jvalue::JValue, method::Method, normal_klass::NormalKlass}};

pub struct Invocation {
    __: PhantomData<()>,
    
    pub target: ResolvedMethodRef,
    pub args: Box<[JValue]>,
}
