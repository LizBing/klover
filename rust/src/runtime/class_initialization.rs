use super::java_thread::JavaThreadId;

#[derive(Debug)]
pub(crate) enum InitializationState {
    Uninitialized,
    Initializing(JavaThreadId),
    Initialized,
    Failed(String),
}
