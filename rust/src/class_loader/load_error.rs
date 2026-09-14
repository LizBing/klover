use std::marker::PhantomData;

use crate::oops::oops_errors::LinkageError;

#[derive(Debug)]
pub enum LoadErrorKind {
    NotFound,

    Duplicated,

    Parse(cafebabe::ParseError),

    Linkage(LinkageError),
}

#[derive(Debug)]
pub struct LoadError {
    pub(super) __: PhantomData<()>,
    
    pub cld_name: Option<String>,
    pub klass_name: String,
    pub kind: LoadErrorKind,
}

pub type LoadResult<T> = Result<T, LoadError>;
