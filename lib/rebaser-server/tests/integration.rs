//! All tests should be ran with the following environment variable:
//!
//! ```shell
//! SI_TEST_BUILTIN_SCHEMAS=none
//! ```

// TODO(nick): decide what to do here. Originally, the rebaser had its own test database (the const
// below) to test the overall connections of the rebaser. It used to not depend on the dal and, in
// the early development of the rebaser, I was testing client calls and server responses. It was
// mostly just returning simple messages and asserting what they looked like in tests. Now, at the
// time of writing, the rebaser performs real rebases and the simple responses have been removed.
// The "si_dal", "si_dal_test", and "si_dal_test_<id>" databases are used with the rebaser. As a
// result, the tests in this directory have been reduced to simple connection calls. The question
// becomes: what to do next? On one hand, we can remove this directory since the dal integration
// tests are performing the heavy work. On the other hand, this directory might have tests that help
// us catch basic connection and health check issues with the rebaser rather than making the user
// comb through panics in the dal test macro (e.g. the rebaser server fails booting when running a
// dal integration test). We should decide what to do once the "Mostly Everything is a Node or an
// Edge" work matures further or hits "main".
const TEST_PG_DBNAME: &str = "si_test_rebaser";

mod integration_test;
