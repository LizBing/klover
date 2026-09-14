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
