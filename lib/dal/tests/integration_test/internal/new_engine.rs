//! This is a temporary module to co-locate all tests for the new engine layer. Once everything is
//! working, this module will go away and the tests will be moved or removed.
//!
//! For all tests in this module, provide "SI_TEST_BUILTIN_SCHEMAS=none" or "SI_TEST_BUILTIN_SCHEMAS=test" as an
//! environment variable.

mod before_funcs;
mod builtins;
mod component;
mod connection;
mod frame;
mod prop;
mod property_editor;
mod rebaser;
mod sdf_mock;
