pub mod context;

pub use context::Context;

use crate::Client;

pub fn new(client: Client) -> Context {
    Context::new(client)
}

pub fn with_domain<T: AsRef<str>>(client: Client, domain: T) -> Context {
    context::Context::with_domain(client, domain)
}

pub fn with_prefix(client: Client, prefix: &str) -> Context {
    context::Context::with_prefix(client, prefix)
}
