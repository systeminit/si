#![recursion_limit = "256"]

mod service_tests;

pub(crate) mod dal {
    pub use si_test_macros::dal_test as test;
}
