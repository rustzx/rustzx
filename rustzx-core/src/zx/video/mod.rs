//! Module contains all screen-rendering platform-independent
//! types and functions
#[cfg(feature = "precise-border")]
pub(crate) mod border;
pub(crate) mod screen;

pub mod colors;
