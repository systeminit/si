use axum::extract::{
    Path,
    State,
};
use innit_core::ListParametersResponse;

use super::AppError;
use crate::{
    app_state::AppState,
    routes::Json,
};

pub async fn list_parameters_route(
    Path(path): Path<String>,
    State(AppState {
        parameter_store_client,
        ..
    }): State<AppState>,
) -> Result<Json<ListParametersResponse>, AppError> {
    let parameters = parameter_store_client
        .parameters_by_path(path)
        .await?
        .iter()
        .cloned()
        .map(innit_core::Parameter::from)
        .collect();

    Ok(Json(ListParametersResponse { parameters }))
}
