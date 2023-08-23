//! This library provides the ability to [connect](Environment) to [RabbitMQ](https://rabbitmq.com)
//! nodes, [produce](Producer) stream messages, and [consume](Consumer) stream messages.

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

mod consumer;
mod environment;
mod error;
mod producer;

pub use consumer::Consumer;
pub use environment::Environment;
pub use error::RabbitError;
pub use error::RabbitResult;
pub use producer::Producer;
