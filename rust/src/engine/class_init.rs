use crate::{
    class_loader::ms_api::MSRef,
    engine::{
        call::Invocation,
        exec_error::{ExecError, ExecResult},
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
pub(crate) enum Continuation {
    EnterRoot(Invocation),
    InvokeStatic {
        target: ResolvedMethod,
        arg_slots: usize,
    },
    GetStatic(ResolvedFieldRef),
    PutStatic(ResolvedFieldRef),
    /// Resume the ClassInitFrame that requested a prerequisite initialization.
    ResumeInitializer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassInitPhase {
    InstallConstantValues,
    InitializePrerequisites,
    AwaitPrerequisite,
    InvokeClinit,
    AwaitClinit,
    Complete,
}

/// A VM control frame.  It owns no Java locals or operand stack; it coordinates
/// initialization and remembers the active-use operation that must resume.
#[derive(Debug)]
pub(crate) struct ClassInitFrame {
    klass: MSRef<NormalKlass>,
    phase: ClassInitPhase,
    prerequisites: Box<[MSRef<NormalKlass>]>,
    next_prerequisite: usize,
    continuation: Continuation,
}

impl ClassInitFrame {
    pub(crate) fn new_claimed(
        klass: MSRef<NormalKlass>,
        prerequisites: Box<[MSRef<NormalKlass>]>,
        continuation: Continuation,
    ) -> Self {
        Self {
            klass,
            phase: ClassInitPhase::InstallConstantValues,
            prerequisites,
            next_prerequisite: 0,
            continuation,
        }
    }

    pub(crate) fn klass(&self) -> &MSRef<NormalKlass> {
        &self.klass
    }

    pub(crate) fn phase(&self) -> ClassInitPhase {
        self.phase
    }

    pub(crate) fn set_phase(&mut self, phase: ClassInitPhase) {
        self.phase = phase;
    }

    pub(crate) fn next_prerequisite(&mut self) -> Option<MSRef<NormalKlass>> {
        let prerequisite = self.prerequisites.get(self.next_prerequisite)?.clone();
        self.next_prerequisite += 1;
        Some(prerequisite)
    }

    pub(crate) fn into_parts(self) -> (MSRef<NormalKlass>, Continuation) {
        (self.klass, self.continuation)
    }
}

#[derive(Debug)]
pub struct ClassInitialization;

impl ClassInitialization {
    pub fn begin(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<ClassInitAction> {
        klass.begin_initialization(owner).map_err(ExecError::from)
    }

    /// JVMS 5.5 step 6 installs ConstantValue fields after the initialization
    /// claim is acquired and before any prerequisite type is initialized.
    pub fn install_constant_values(klass: &NormalKlass) -> ExecResult<()> {
        klass.initialize_static_constant_values()
    }

    pub(crate) fn prerequisites(klass: &NormalKlass) -> Box<[MSRef<NormalKlass>]> {
        if klass.is_interface() {
            return Box::new([]);
        }

        let mut prerequisites = Vec::new();
        if let Some(super_klass) = klass.super_klass_ref() {
            prerequisites.push(super_klass);
        }

        let mut visited = Vec::new();
        for interface in klass.direct_interfaces() {
            Self::collect_default_method_interfaces(
                interface.clone(),
                &mut visited,
                &mut prerequisites,
            );
        }

        prerequisites.into_boxed_slice()
    }

    fn collect_default_method_interfaces(
        interface: MSRef<NormalKlass>,
        visited: &mut Vec<MSRef<NormalKlass>>,
        prerequisites: &mut Vec<MSRef<NormalKlass>>,
    ) {
        if visited.iter().any(|seen| seen.equals(&interface)) {
            return;
        }
        visited.push(interface.clone());

        for parent in interface.direct_interfaces() {
            Self::collect_default_method_interfaces(parent.clone(), visited, prerequisites);
        }

        if interface.declares_default_method() {
            prerequisites.push(interface);
        }
    }

    pub fn complete(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        klass
            .complete_initialization(owner)
            .map_err(ExecError::from)
    }

    pub fn abort(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        klass.abort_initialization(owner).map_err(ExecError::from)
    }

    pub fn fail(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        klass.fail_initialization(owner).map_err(ExecError::from)
    }
}
