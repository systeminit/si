//! This is a temporary module to co-locate all tests for the new engine layer. Once everything is
//! working, this module will go away and the tests will be moved.
//!
//! For all tests in this module, provide "SI_TEST_BUILTIN_SCHEMAS=none" as an environment variable.

mod builtins;
mod component;
mod rebaser;
mod sdf_mock;

// mod change_set;
// mod content_store;
