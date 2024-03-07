/// Contains tests that will become part of individual package testing (i.e. testing that a "Docker
/// Image" connects and works as intended with a "Butane Container").
// mod external;
/// Contains tests that test SI directly and use test-exclusive builtins. All tests in this module
/// should (eventually) pass with `SI_TEST_BUILTIN_SCHEMAS=test`.
// mod action_prototype;
// mod attribute;
// mod change_set;
// mod component;
// mod diagram;
// mod edge;
// mod func;
// mod func_execution;
// mod graph;
// mod history_event;
// mod key_pair;
mod before_funcs;
mod builtins;
mod component;
mod connection;
mod frame;
mod prop;
mod property_editor;
mod rebaser;
mod schema_variant_views;
mod secret;
