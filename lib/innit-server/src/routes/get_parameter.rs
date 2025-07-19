use axum::extract::{
    Path,
    State,
};
use innit_core::{
    GetParameterResponse,
    Parameter,
};
use telemetry::tracing::info;

use super::AppError;
use crate::{
    app_state::AppState,
    routes::Json,
};

pub async fn get_parameter_route(
    Path(name): Path<String>,
    State(AppState {
        parameter_cache,
        parameter_store_client,
        ..
    }): State<AppState>,
) -> Result<Json<GetParameterResponse>, AppError> {
    let name = if !name.starts_with('/') {
        format!("/{name}")
    } else {
        name.clone()
    };

    if let Some(parameter) = parameter_cache.get_parameter(&name).await {
        info!("Fetched parameter from cache: {name}");
        return Ok(Json(GetParameterResponse {
            parameter,
            is_cached: true,
        }));
    }

    let parameter: Parameter = parameter_store_client
        .get_parameter(name.clone())
        .await?
        .into();

    parameter_cache
        .set_parameter(Parameter {
            name: parameter.name.clone(),
            value: parameter.value.clone(),
            r#type: parameter.r#type.clone(),
        })
        .await;

    info!("Fetched parameter from ParameterStore: {name}");

    Ok(Json(GetParameterResponse {
        parameter,
        is_cached: false,
    }))
}
