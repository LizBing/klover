#[derive(Debug)]
pub enum ResolveError {
    MismatchCPType,
    InvalidDesc { raw: String }
}

pub type ResolveResult<T> = Result<T, ResolveError>;
