use crate::{
    engines::{
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        invocation::Invocation,
        java_frame::JavaFrame,
        protocol::{EngineExit, ExecBudget, ExecutionContext, MethodReturn},
    },
    oops::normal_klass::NormalKlass,
    runtime::{class_initialization::InitializationState, java_thread::JavaThreadId},
};

enum Phase {
    Claim,
    Constants,
    Dependencies(usize),
    Invoke,
    Complete,
}

/// A VM frame that preserves initialization progress while Java frames yield.
pub(crate) struct InitializationFrame<'exec, 'meta> {
    klass: &'exec NormalKlass<'meta>,
    dependencies: Vec<&'exec NormalKlass<'meta>>,
    owner: Option<JavaThreadId>,
    phase: Phase,
}

impl<'exec, 'meta> InitializationFrame<'exec, 'meta> {
    pub(crate) fn new(klass: &'exec NormalKlass<'meta>) -> Self {
        Self {
            klass,
            dependencies: klass.initialization_dependencies(),
            owner: None,
            phase: Phase::Claim,
        }
    }

    fn fail(&mut self, reason: &str) {
        if let Some(owner) = self.owner.take() {
            let mut state = self.klass.initialization.lock();
            if matches!(*state, InitializationState::Initializing(id) if id == owner) {
                *state = InitializationState::Failed(reason.into());
            }
        }
    }
}

impl Drop for InitializationFrame<'_, '_> {
    fn drop(&mut self) {
        self.fail("initialization abandoned");
    }
}

impl<'exec, 'meta: 'exec> JavaFrame<'exec, 'meta> for InitializationFrame<'exec, 'meta> {
    fn resume(
        &mut self,
        budget: &mut ExecBudget,
        context: &ExecutionContext,
    ) -> ExecResult<EngineExit<'exec, 'meta>> {
        if !budget.is_startable() {
            return Ok(EngineExit::BudgetExhausted);
        }
        loop {
            match self.phase {
                Phase::Claim => {
                    let mut state = self.klass.initialization.lock();
                    match &*state {
                        InitializationState::Initialized => {
                            return Ok(EngineExit::InitializationComplete);
                        }
                        InitializationState::Initializing(owner) if *owner == context.thread_id => {
                            return Ok(EngineExit::InitializationComplete);
                        }
                        InitializationState::Initializing(_) => {
                            return Ok(EngineExit::BlockedOnInitialization);
                        }
                        InitializationState::Failed(reason) => {
                            return Err(ExecError::new(ExecErrorKind::InitializationFailed {
                                owner: self.klass.name().utf8().into(),
                                reason: reason.clone(),
                            }));
                        }
                        InitializationState::Uninitialized => {}
                    }
                    *state = InitializationState::Initializing(context.thread_id);
                    self.owner = Some(context.thread_id);
                    self.phase = Phase::Constants;
                }
                Phase::Constants => {
                    self.klass
                        .fields()
                        .initialize_constants()
                        .map_err(|field| {
                            ExecError::new(ExecErrorKind::UnsupportedConstantInitialization {
                                owner: self.klass.name().utf8().into(),
                                field: field.name.utf8().into(),
                            })
                        })?;
                    self.phase = Phase::Dependencies(0);
                }
                Phase::Dependencies(index) => {
                    if let Some(dependency) = self.dependencies.get(index) {
                        self.phase = Phase::Dependencies(index + 1);
                        return Ok(EngineExit::Initialize(dependency));
                    }
                    self.phase = Phase::Invoke;
                }
                Phase::Invoke => {
                    self.phase = Phase::Complete;
                    if self.klass.find_declared_method("<clinit>", "()V").is_some() {
                        return Ok(EngineExit::Invoke(Invocation::class_initializer(
                            self.klass,
                        )?));
                    }
                }
                Phase::Complete => {
                    *self.klass.initialization.lock() = InitializationState::Initialized;
                    self.owner = None;
                    return Ok(EngineExit::InitializationComplete);
                }
            }
        }
    }

    fn accept_return(&mut self, value: MethodReturn) -> ExecResult<()> {
        match value {
            MethodReturn::Void => Ok(()),
            _ => Err(ExecError::new(ExecErrorKind::MismatchSlotType)),
        }
    }

    fn abort(&mut self, reason: &str) {
        self.fail(reason);
    }
}
