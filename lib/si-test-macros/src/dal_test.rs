//! Expansion implementation of the `dal_test` attribute macro.
//!
//! This implementation is a combination of a configurable threaded Tokio runtime (formerly
//! provided via the `tokio::test` macro from the `tokio` crate), support for optional
//! tracing/logging support (formerly provided via the `test` mecro from the `test-env-log` crate),
//! and an "extractor"-style dependency setup a little like axum's extractors.
//!
//! # Reference Implementations and Credits
//!
//! * [`tokio::test` macro](https://github.com/tokio-rs/tokio/blob/121769c762ad6b1686ecd0e8618005aab8b7e980/tokio-macros/src/entry.rs)
//! * [`test_env_log::test` macro](https://github.com/d-e-s-o/test-log/blob/544dbac50321aaf580959ad7a7997358517db198/src/lib.rs)

use proc_macro2::TokenStream;
use quote::quote;
use syn::{AttributeArgs, ItemFn, ReturnType};

const LOG_ENV_VAR: &str = "SI_TEST_LOG";
const SPAN_EVENTS_ENV_VAR: &str = "SI_TEST_LOG_SPAN_EVENTS";

const RT_DEFAULT_WORKER_THREADS: usize = 2;
const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

pub(crate) fn expand(item: ItemFn, _args: AttributeArgs) -> TokenStream {
    if item.sig.asyncness.is_none() {
        panic!("test function must be async--blocking tests not supported");
    }

    let attrs = &item.attrs;
    let body = &item.block;
    let test_name = &item.sig.ident;
    // Note that Rust doesn't allow a test function with `#[should_panic]` that has a non-unit
    // return value. Huh
    let output = match &item.sig.output {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, typeness) => quote! {-> #typeness},
    };
    let test_attr = quote! {#[::core::prelude::v1::test]};

    let worker_threads = RT_DEFAULT_WORKER_THREADS;
    let thread_stack_size = RT_DEFAULT_THREAD_STACK_SIZE;

    let tracing_init = expand_tracing_init();
    let rt = expand_runtime(worker_threads, thread_stack_size);

    quote! {
        #test_attr
        #(#attrs)*
        fn #test_name() #output {
            async fn inner() #output #body

            #[inline]
            async fn imp() #output {
                // Future test parameters/"extractors" get set up here and passed into `inner(...)`

                inner().await
            }

            #tracing_init

            eprintln!("I'm going to call the things");
            #[allow(clippy::expect_used)]
            #rt.block_on(imp())
        }
    }
}

fn expand_tracing_init() -> TokenStream {
    let span_events_env_var = SPAN_EVENTS_ENV_VAR;
    let log_env_var = LOG_ENV_VAR;

    quote! {
        let event_filter = {
            use ::tracing_subscriber::fmt::format::FmtSpan;

            match ::std::env::var(#span_events_env_var) {
                Ok(value) => {
                    value
                        .to_ascii_lowercase()
                        .split(",")
                        .map(|filter| match filter.trim() {
                            "new" => FmtSpan::NEW,
                            "enter" => FmtSpan::ENTER,
                            "exit" => FmtSpan::EXIT,
                            "close" => FmtSpan::CLOSE,
                            "active" => FmtSpan::ACTIVE,
                            "full" => FmtSpan::FULL,
                            _ => panic!(
                                "{}: {} must contain filters separated by `,`.\n\t\
                                For example: `active` or `new,close`\n\t
                                Got: {}",
                                concat!(env!("CARGO_PKG_NAME"), "::dal_test"),
                                #span_events_env_var,
                                value,
                            ),
                        })
                        .fold(FmtSpan::NONE, |acc, filter| filter | acc)
                },
                Err(::std::env::VarError::NotUnicode(_)) => {
                    panic!(
                        "{}: {} must contain a valid UTF-8 string",
                        concat!(env!("CARGO_PKG_NAME"), "::dal_test"),
                        #span_events_env_var,
                    )
                }
                Err(::std::env::VarError::NotPresent) => FmtSpan::NONE,
            }
        };

        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_env(#log_env_var))
            .with_span_events(event_filter)
            .with_test_writer()
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);
    }
}

fn expand_runtime(worker_threads: usize, thread_stack_size: usize) -> TokenStream {
    quote! {
        ::tokio::runtime::Builder::new_multi_thread()
            .worker_threads(#worker_threads)
            .thread_stack_size(#thread_stack_size)
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
    }
}
