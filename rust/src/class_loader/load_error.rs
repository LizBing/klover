use crate::class_parser::parse_error::ParseError;

#[derive(Debug, Clone)]
pub enum LoadError {
    NotFound(String),
    Parse(ParseError),
    StillLoading(String),
    SuperNotNormal(String),
    Duplicated { cld_name: Option<String>, class_name: String },
    NoSuper { class_name: String },
    Circularity,
}

pub type LoadResult<T> = Result<T, LoadError>;
