use axum::{
    Json,
    extract::{
        Path,
        State,
    },
};
use innit_core::{
    CreateParameterRequest,
    CreateParameterResponse,
    Parameter,
};
use telemetry::tracing::info;

use super::AppError;
use crate::app_state::AppState;

pub async fn create_parameter_route(
    Path(name): Path<String>,
    State(AppState {
        parameter_cache,
        parameter_store_client,
        ..
    }): State<AppState>,
    Json(CreateParameterRequest { value }): Json<CreateParameterRequest>,
) -> Result<Json<CreateParameterResponse>, AppError> {
    let name = if !name.starts_with('/') {
        format!("/{name}")
    } else {
        name.clone()
    };

    parameter_store_client
        .create_string_parameter(name.clone(), value.clone())
        .await?;

    parameter_cache
        .set_parameter(Parameter {
            name: name.clone(),
            value: Some(value.clone()),
            // TODO(scott): this whole route should handle types somehow, but since this is currently just used
            // in tests this is fine for now
            r#type: None,
        })
        .await;

    info!("Created parameter: {name}/{value}");

    Ok(Json(CreateParameterResponse {}))
}
