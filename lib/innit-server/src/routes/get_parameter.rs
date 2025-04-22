use axum::extract::{
    Path,
    State,
};
use innit_core::GetParameterResponse;

use super::AppError;
use crate::{
    app_state::AppState,
    routes::Json,
};

pub async fn get_parameter_route(
    Path(name): Path<String>,
    State(AppState {
        parameter_store_client,
        ..
    }): State<AppState>,
) -> Result<Json<GetParameterResponse>, AppError> {
    let parameter = parameter_store_client.get_parameter(name).await?.into();

    Ok(Json(GetParameterResponse { parameter }))
}
