/// Contains tests that will become part of individual package testing (i.e. testing that a "Docker
/// Image" connects and works as intended with a "Kubernetes Deployment").
mod external;
/// Contains tests that test SI directly and use test-exclusive builtins. All tests in this module
/// should (eventually) pass with `SI_TEST_BUILTIN_SCHEMAS=test`.
mod internal;
