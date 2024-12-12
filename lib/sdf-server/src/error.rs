use axum::http::StatusCode;
use thiserror::Error;

use crate::service::{
    action::ActionError, change_set::ChangeSetError, component::ComponentError, module::ModuleError,
};

#[derive(Debug)]
pub struct ServerError(anyhow::Error);

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let mut err: anyhow::Error = err.into();

        let additional_context = if let Some(action_error) = err.downcast_ref::<ActionError>() {
            match action_error {
                ActionError::InvalidOnHoldTransition(_) => Some(HttpStatus::NotModified),
                _ => None,
            }
        } else if let Some(change_set_error) = err.downcast_ref::<ChangeSetError>() {
            match change_set_error {
                ChangeSetError::ActionAlreadyEnqueued(_) => Some(HttpStatus::NotModified),
                ChangeSetError::DvuRootsNotEmpty(_) => Some(HttpStatus::PreconditionRequired {
                    message: Some(
                        "There are dependent values that still need to be calculated. Please retry!"
                            .to_string(),
                    ),
                }),
                _ => None,
            }
        } else if let Some(dal_change_set_error) = err.downcast_ref::<dal::ChangeSetError>() {
            match dal_change_set_error {
                dal::ChangeSetError::ChangeSetNotFound(..) => Some(HttpStatus::NotFound),
                _ => None,
            }
        } else if let Some(_) = err.downcast_ref::<dal::ChangeSetApplyError>() {
            Some(HttpStatus::Conflict)
        } else if let Some(component_error) = err.downcast_ref::<ComponentError>() {
            match component_error {
                ComponentError::SchemaNotFound
                | ComponentError::InvalidVisibility
                | ComponentError::PropNotFound(_)
                | ComponentError::SchemaVariantNotFound
                | ComponentError::NotFound(_) => Some(HttpStatus::NotFound),
                ComponentError::SchemaVariantUpgradeSkipped => Some(HttpStatus::NotModified),
                ComponentError::KeyAlreadyExists(_) => Some(HttpStatus::UnprocessableEntity),
                _ => None,
            }
        } else if let Some(dal_component_error) = err.downcast_ref::<dal::ComponentError>() {
            match dal_component_error {
                dal::ComponentError::NotFound(_) => Some(HttpStatus::NotFound),
                _ => None,
            }
        } else if let Some(module_error) = err.downcast_ref::<ModuleError>() {
            match module_error {
                ModuleError::ModuleHashNotFound(_)
                | ModuleError::PackageNotFound(_)
                | ModuleError::SchemaNotFoundForVariant(_)
                | ModuleError::SchemaVariantNotFound(_)
                | ModuleError::WorkspaceNotFound(_) => Some(HttpStatus::NotFound),
                _ => None,
            }
        } else if let Some(management_error) =
            err.downcast_ref::<dal::management::prototype::ManagementPrototypeError>()
        {
            match management_error {
                dal::management::prototype::ManagementPrototypeError::FuncExecutionFailure(
                    message,
                ) => Some(HttpStatus::BadRequest {
                    message: Some(message.clone()),
                }),
                _ => None,
            }
        } else {
            None
        };

        if let Some(additional_context) = additional_context {
            err = err.context(additional_context);
        }

        Self(err)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the alternate display impl to get the full list of sources in the error.
        write!(f, "{:#}", self.0)
    }
}

impl axum::response::IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, message) = match self.0.downcast_ref::<HttpStatus>() {
            Some(status) => match status {
                HttpStatus::BadRequest { message } => {
                    if let Some(message) = message {
                        (StatusCode::BAD_REQUEST, message.clone())
                    } else {
                        (StatusCode::BAD_REQUEST, format!("{:#}", self.0))
                    }
                }
                HttpStatus::Conflict => (StatusCode::CONFLICT, format!("{:#}", self.0)),
                HttpStatus::NotFound => (StatusCode::NOT_FOUND, format!("{:#}", self.0)),
                HttpStatus::NotModified => (StatusCode::NOT_MODIFIED, format!("{:#}", self.0)),
                HttpStatus::PreconditionRequired { message } => {
                    if let Some(message) = message {
                        (StatusCode::PRECONDITION_REQUIRED, message.clone())
                    } else {
                        (StatusCode::PRECONDITION_REQUIRED, format!("{:#}", self.0))
                    }
                }
                HttpStatus::UnprocessableEntity => {
                    (StatusCode::UNPROCESSABLE_ENTITY, format!("{:#}", self.0))
                }
            },
            None => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:#}", self.0),
            ),
        };

        let body = axum::response::Json(serde_json::json!({
            "error": {
                "message": message,
                "code": 42,
                "statusCode": status_code.as_u16(),
            }
        }));
        telemetry::prelude::error!(si.error.message = message);
        (status_code, body).into_response()
    }
}

/// Can be added to any error through [`anyhow::Context`] to set the HTTP status code of the response.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum HttpStatus {
    BadRequest {
        message: Option<String>,
    },
    Conflict,
    NotFound,
    /// Causes the HTTP resposne to have the status code [`axum::http::StatusCode::NOT_MODIFIED`].
    NotModified,
    PreconditionRequired {
        message: Option<String>,
    },
    UnprocessableEntity,
}

impl std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
