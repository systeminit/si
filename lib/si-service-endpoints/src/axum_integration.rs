use crate::{DefaultServiceEndpoints, ServiceEndpointsConfig};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn create_router(
    service: Arc<DefaultServiceEndpoints>,
    config: &ServiceEndpointsConfig,
) -> Router {
    Router::new()
        .route(&config.health_endpoint, get(health_handler))
        .route(&config.config_endpoint, get(config_handler))
        .layer(CorsLayer::permissive())
        .with_state(service)
}

async fn health_handler(
    State(_service): State<Arc<DefaultServiceEndpoints>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

async fn config_handler(
    State(service): State<Arc<DefaultServiceEndpoints>>,
) -> impl IntoResponse {
    let response = json!({
        "service": service.service_name(),
        "config": service.config()
    });
    (StatusCode::OK, Json(response))
}