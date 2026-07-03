use crate::{class_parser::parse_error::ParseError, oops::resolve_error::ResolveError};

#[derive(Debug)]
pub enum LoadError {
    NotFound(String),
    Parse(ParseError),
    Resolve(ResolveError),
    StillLoading(String),
    SuperNotNormal(String),
    Duplicated { cld_name: Option<String>, class_name: String },
    NoSuper { class_name: String }
}

pub type LoadResult<T> = Result<T, LoadError>;
