use crate::class_loader::load_error::LoadError;

#[derive(Debug)]
pub enum ResolveError {
    MismatchCPType,
    MismatchAttrType,
    InvalidDesc { raw: String },
    UnknownRefKind { kind: u8 },
}

pub type ResolveResult<T> = Result<T, ResolveError>;
