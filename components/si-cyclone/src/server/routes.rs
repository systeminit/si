use super::{handlers, Config};
use axum::{handler::get, routing::BoxRoute, Router};
use tracing::debug;

pub fn routes(config: &Config) -> Router<BoxRoute> {
    Router::new()
        .route(
            "/liveness",
            get(handlers::liveness).head(handlers::liveness),
        )
        .route(
            "/readiness",
            get(handlers::readiness).head(handlers::readiness),
        )
        .route("/execute", execute_routes(&config))
        .boxed()
}

fn execute_routes(config: &Config) -> Router<BoxRoute> {
    let mut router = Router::new().boxed();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router
            .route("/ping", get(handlers::execute_ws_ping))
            .boxed();
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router
            .route("/resolver", get(handlers::execute_ws_resolver))
            .boxed();
    }

    router
}
