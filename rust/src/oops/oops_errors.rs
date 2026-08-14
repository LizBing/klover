use crate::class_loader::load_error::LoadError;

#[derive(Debug, Clone)]
pub enum ResolveError {
    // 运行时解析错误（CP 引用解析阶段）
    ClassNotFound,
    MethodNotFound,
    FieldNotFound,

    DuplicatedAttr,

    WrongRefType,

    InvalidCPIndex,

    IllegalMethodName(String),

    Load(LoadError),
}

pub type ResolveResult<T> = Result<T, ResolveError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassInitError {
    InvalidTransition,
}

pub type ClassInitResult<T> = Result<T, ClassInitError>;
