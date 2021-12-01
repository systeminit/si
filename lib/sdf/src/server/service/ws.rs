use axum::{routing::get, Router};

pub mod billing_account_updates;

pub fn routes() -> Router {
    Router::new().route(
        "/billing_account_updates",
        get(billing_account_updates::billing_account_updates),
    )
}
