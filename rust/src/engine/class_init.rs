use crate::{
    engine::exec_error::{ExecError, ExecResult},
    oops::normal_klass::{ClassInitAction, NormalKlass},
    runtime::java_thread::JavaThreadID,
};

#[derive(Debug)]
pub struct ClassInitialization;

impl ClassInitialization {
    /// Prepare a class for an active-use instruction.
    ///
    /// `NormalKlass` owns only the state transitions.  The engine owns ordering
    /// and the decision to execute bytecode.  Until ClassInitFrame exists, classes
    /// without `<clinit>` can complete initialization and classes with one fail
    /// explicitly while returning their state to `Uninitialized`.
    pub fn ensure_initialized(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        match klass.begin_initialization(owner)? {
            ClassInitAction::AlreadyInitialized | ClassInitAction::RecursiveRequest => Ok(()),
            ClassInitAction::Initialize => Self::initialize_claimed(klass, owner),
        }
    }

    fn initialize_claimed(klass: &NormalKlass, owner: JavaThreadID) -> ExecResult<()> {
        let preparation = (|| {
            if !klass.is_interface() {
                if let Some(super_klass) = klass.super_klass_ref() {
                    Self::ensure_initialized(&super_klass, owner)?;
                }
            }

            if klass.find_declared_method("<clinit>", "()V").is_some() {
                return Err(ExecError::ClassInitializerNotSupported);
            }

            Ok(())
        })();

        match preparation {
            Ok(()) => klass
                .complete_initialization(owner)
                .map_err(ExecError::from),
            Err(error) => {
                // The class did not execute `<clinit>` and therefore must not be
                // recorded as initialized or erroneous.
                klass.abandon_initialization(owner)?;
                Err(error)
            }
        }
    }
}
