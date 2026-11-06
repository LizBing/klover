use crate::{class_parser::ParseError, oops::oops_errors::LinkageError};

#[derive(Debug)]
pub enum LoadError {
    NotFound,

    NoClassDefFound,

    Duplicated,

    NameMismatch { expected: String, actual: String },

    Parse(ParseError),

    Linkage(LinkageError),
}

pub type LoadResult<T> = Result<T, LoadError>;
