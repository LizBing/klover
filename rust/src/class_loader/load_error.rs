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
    _private: (),
    
    pub cld_name: Option<String>,
    pub klass_name: String,
    pub kind: LoadErrorKind,
}

impl LoadError {
    pub(super) fn new(cld_name: Option<String>, klass_name: String, kind: LoadErrorKind) -> Self {
        Self {
            _private: (),
            cld_name,
            klass_name,
            kind,
        }
    }
}

pub type LoadResult<T> = Result<T, LoadError>;
