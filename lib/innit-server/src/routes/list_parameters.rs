use crate::routes::Json;
use aws_sdk_ssm::error::SdkError;
use aws_sdk_ssm::operation::get_parameters_by_path::GetParametersByPathError;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use innit_core::ListParametersResponse;
use thiserror::Error;

use crate::{api_error::ApiError, app_state::AppState};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ListParamsError {
    #[error("AWS SSM error: {0}")]
    Aws(#[from] SdkError<GetParametersByPathError>),
    #[error("Parameter path not found: {0}")]
    PathNotFound(String),
}

impl IntoResponse for ListParamsError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        ApiError::new(status_code, error_message).into_response()
    }
}

pub async fn list_parameters_route(
    Path(path): Path<String>,
    State(AppState { ssm_client, .. }): State<AppState>,
) -> Result<Json<ListParametersResponse>, ListParamsError> {
    let result = ssm_client
        .get_parameters_by_path()
        .path(format!("/{path}"))
        .recursive(true)
        .send()
        .await?;

    let parameters = result.parameters();
    if parameters.is_empty() {
        return Err(ListParamsError::PathNotFound(path));
    }
    let parameters = parameters
        .iter()
        .cloned()
        .map(innit_core::Parameter::from)
        .collect();

    Ok(Json(ListParametersResponse { parameters }))
}
