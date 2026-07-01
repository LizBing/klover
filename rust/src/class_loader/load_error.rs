use crate::{class_parser::parse_error::ParseError, oops::resolve_error::ResolveError};

#[derive(Debug)]
pub enum LoadError {
    NotFound(String),
    Parse(ParseError),
    Resolve(ResolveError),
    NotLoaded(String),
    StillLoading(String),
    SuperNotNormal(String),
    LoadingFailed(String),
}

pub type LoadResult<T> = Result<T, LoadError>;
