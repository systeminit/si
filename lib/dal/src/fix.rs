//! This module contains the concept of "fixes".

/// Contains the history of _all_ fixes, provided by [`FixExecution`](crate::FixExecution).
pub mod execution;
/// Contains the ability to resolve _current_ fixes, provided by [`FixResolver`](crate::FixResolver).
pub mod resolver;
