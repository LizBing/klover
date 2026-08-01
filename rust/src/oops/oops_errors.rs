#[derive(Debug, Clone)]
pub enum ResolveError {
    MismatchCPType,
    MismatchAttrType,
    InvalidDesc(String),
    UnknownRefKind(u8),
    NotANormal,

    // 运行时解析错误（CP 引用解析阶段）
    ClassNotFound,
    MethodNotFound,
    FieldNotFound,

    DuplicatedAttr,

    WrongRefType,

    InvalidCPIndex,

    IllegalMethodName(String),
}

pub type ResolveResult<T> = Result<T, ResolveError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassInitError {
    Erroneous,
    InvalidTransition,
}

pub type ClassInitResult<T> = Result<T, ClassInitError>;
