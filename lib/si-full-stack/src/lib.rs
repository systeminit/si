#![warn(
    bad_style,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

mod client;
mod config;
mod error;
mod server;

use std::{sync::Arc, thread};
use server::Server;
use telemetry::prelude::*;

pub use config::{Config, ConfigBuilder, ConfigBuilderError};
pub use error::SiFullStackError;
pub use client::Client;

use error::SiFullStackResult;
use tokio::runtime::Runtime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 10;

pub fn run(config: Config) -> SiFullStackResult<Client> {
    let tokio_runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("si-full-stack")
            .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
            .build()?,
    );
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let inner_runtime = tokio_runtime.clone();
    let inner_config = config.clone();
    let join_handle = thread::spawn(move || {
        inner_runtime.block_on(Server::run(inner_config, shutdown_rx))
    });
    let client = Client::new(config, join_handle, shutdown_tx);
    Ok(client)
}
