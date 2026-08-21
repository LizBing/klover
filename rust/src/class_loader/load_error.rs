use crate::{class_parser::parse_error::ParseError, linkage::linkage_error::LinkageError};

#[derive(Debug, Clone)]
pub enum LoadError {
    NotFound(String),

    Parse(ParseError),

    Link(LinkageError),

    StillLoading(String),

    Duplicated {
        cld_name: Option<String>,
        class_name: String
    },
}

pub type LoadResult<T> = Result<T, LoadError>;
