#[cfg(not(feature = "integration-tests"))]
mod class_loader;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod class_loader;
mod class_parser;
#[cfg(not(feature = "integration-tests"))]
mod code;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod code;
#[cfg(not(feature = "integration-tests"))]
mod engines;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod engines;
#[cfg(not(feature = "integration-tests"))]
mod gc_bindings;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod gc_bindings;
#[cfg(not(feature = "integration-tests"))]
mod oops;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod oops;
#[cfg(not(feature = "integration-tests"))]
mod runtime;
#[cfg(feature = "integration-tests")]
#[doc(hidden)]
pub mod runtime;
