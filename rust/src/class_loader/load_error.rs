use crate::{class_parser::parse_error::ParseError, oops::oops_errors::ResolveError};

#[derive(Debug, Clone)]
pub enum LoadError {
    NotFound(String),
    Parse(ParseError),
    Resolve(ResolveError),
    StillLoading(String),
    SuperNotNormal(String),
    Duplicated { cld_name: Option<String>, class_name: String },
    NoSuper { class_name: String },
    Circularity,
}

impl From<ParseError> for LoadError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<ResolveError> for LoadError {
    fn from(value: ResolveError) -> Self {
        Self::Resolve(value)
    }
}

pub type LoadResult<T> = Result<T, LoadError>;
