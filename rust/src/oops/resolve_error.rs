#[derive(Debug)]
pub enum ResolveError {
    MismatchCPType,
    MismatchAttrType,
    InvalidDesc { raw: String },
    UnknownRefKind { kind: u8 },
    NotANormal,

    // 运行时解析错误（CP 引用解析阶段）
    ClassNotFound,
    MethodNotFound,
    FieldNotFound,

    DuplicatedAttr
}

pub type ResolveResult<T> = Result<T, ResolveError>;
