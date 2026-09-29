use crate::class_loader::load_error::LoadError;

#[derive(Debug)]
pub enum ResolveError {
    Load(LoadError),
}

#[derive(Debug)]
pub enum LinkageError {
    SuperNotFound {
        name: String,
    },
    
    NotInterface {
        name: String,
    },

    NotNormalKlass {
        name: String,
    },
}

pub type ResolveResult<T> = Result<T, ResolveError>;

pub type LinkageResult<T> = Result<T, LinkageError>;
