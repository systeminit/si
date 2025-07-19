use axum::extract::{
    Path,
    State,
};
use innit_core::{
    ListParametersResponse,
    Parameter,
};
use telemetry::tracing::info;

use super::AppError;
use crate::{
    app_state::AppState,
    routes::Json,
};

pub async fn list_parameters_route(
    Path(path): Path<String>,
    State(AppState {
        parameter_cache,
        parameter_store_client,
        ..
    }): State<AppState>,
) -> Result<Json<ListParametersResponse>, AppError> {
    let path = if !path.starts_with('/') {
        format!("/{path}")
    } else {
        path
    };

    if let Some(parameters) = parameter_cache.get_parameters_by_path(&path).await {
        info!("Fetched parameter from cache at path: {path}");
        return Ok(Json(ListParametersResponse {
            parameters,
            is_cached: true,
        }));
    }

    let parameters: Vec<Parameter> = parameter_store_client
        .parameters_by_path(path.clone())
        .await?
        .iter()
        .cloned()
        .map(Parameter::from)
        .collect();

    parameter_cache
        .set_parameters_by_path(path.clone(), parameters.clone())
        .await;

    info!("Fetched parameter from ParameterStore at path: {path}");

    Ok(Json(ListParametersResponse {
        parameters,
        is_cached: false,
    }))
}
