use crate::{
    class_loader::ms_api::MSRef,
    engine::{
        call::Invocation,
        exec_error::{ExecError, ExecResult},
        outcome::PendingException,
        resolved_method::ResolvedMethod,
    },
    oops::{
        cp_entry::ResolvedFieldRef,
        normal_klass::{ClassInitAction, NormalKlass},
    },
    runtime::java_thread::JavaThreadID,
};

/// Work suspended by an active-use check.  The dispatcher applies it only
/// after the target class has completed initialization.
#[derive(Debug)]
pub enum Continuation {
    EnterRoot(Invocation),
    InvokeStatic {
        target: ResolvedMethod,
        arg_slots: usize,
    },
    GetStatic(ResolvedFieldRef),
    PutStatic(ResolvedFieldRef),
    /// No explicit work is needed: popping the child initialization frame
    /// naturally exposes the parent ClassInitFrame again.
    ResumeCaller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassInitPhase {
    InitializeSuper,
    InvokeClinit,
    AwaitClinit,
    Complete,
}

/// A VM control frame.  It owns no Java locals or operand stack; it coordinates
/// initialization and remembers the active-use operation that must resume.
#[derive(Debug)]
pub struct ClassInitFrame {
    klass: MSRef<NormalKlass>,
    phase: ClassInitPhase,
    continuation: Continuation,
}

impl ClassInitFrame {
    pub fn new_claimed(klass: MSRef<NormalKlass>, continuation: Continuation) -> Self {
        Self {
            klass,
            phase: ClassInitPhase::InitializeSuper,
            continuation,
        }
    }

    pub fn klass(&self) -> &MSRef<NormalKlass> {
        &self.klass
    }

    pub fn phase(&self) -> ClassInitPhase {
        self.phase
    }

    pub fn set_phase(&mut self, phase: ClassInitPhase) {
        self.phase = phase;
    }

    pub fn into_parts(self) -> (MSRef<NormalKlass>, Continuation) {
        (self.klass, self.continuation)
    }
}

#[derive(Debug)]
pub struct ClassInitialization;

impl ClassInitialization {
    pub fn begin(
        klass: &NormalKlass,
        owner: JavaThreadID,
    ) -> ExecResult<ClassInitAction> {
        klass.begin_initialization(owner).map_err(ExecError::from)
    }

    /// ConstantValue installation currently lives here so this change remains
    /// scoped to executable class initialization.  It can move to a dedicated
    /// preparation phase without changing ClassInitFrame or Continuation.
    pub fn prepare_claimed(klass: &NormalKlass) -> ExecResult<()> {
        klass.initialize_static_constant_values()
    }

    pub fn complete(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        klass
            .complete_initialization(owner)
            .map_err(ExecError::from)
    }

    pub fn abandon(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        klass
            .abandon_initialization(owner)
            .map_err(ExecError::from)
    }

    pub fn fail(
        klass: &NormalKlass,
        owner: JavaThreadID,
        cause: PendingException,
    ) -> ExecResult<()> {
        klass
            .fail_initialization(owner, cause)
            .map_err(ExecError::from)
    }
}
