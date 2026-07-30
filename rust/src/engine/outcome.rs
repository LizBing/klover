use crate::{engine::{engine_error::JavaExceptionKind, resolved_method::ResolvedMethod}, gc_bindings::oop_handle::NObjPtr};

#[derive(Debug)]
pub enum RetValue {
    Void,
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Ref(NObjPtr)
}

#[derive(Debug)]
pub enum PendingException {
    JavaObj(NObjPtr),
    JVMGen(JavaExceptionKind),
}

#[derive(Debug)]
pub enum StepOutcome {
    Continue,
    Branch(usize),
    Call {
        target: ResolvedMethod,
        arg_slots: usize
    },
    Return(RetValue),
    Throw(PendingException)
}

#[derive(Debug)]
pub enum ThreadExit {
    Returned(RetValue),
    UncaughtException(PendingException),
}

#[derive(Debug)]
pub enum RunOutcome {
    QuantumExpired,
    Terminated(ThreadExit),
}
