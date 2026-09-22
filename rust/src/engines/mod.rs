pub mod exec_dispatcher;
pub mod exec_error;
#[cfg(not(feature = "integration-tests"))]
mod interpreter;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod interpreter;
pub mod invocation;
pub mod java_frame;
