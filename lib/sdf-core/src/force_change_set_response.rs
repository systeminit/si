use axum::{
    http::{HeaderName, HeaderValue},
    response::{IntoResponse, Response},
};
use dal::ChangeSetId;
use hyper::{HeaderMap, StatusCode, header};
use serde::Serialize;
use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, Copy, Default)]
pub struct ForceChangeSetResponse<T>
where
    T: Serialize,
{
    force_change_set_id: Option<ChangeSetId>,
    response: Option<T>,
}

impl<T> ForceChangeSetResponse<T>
where
    T: Serialize,
{
    pub fn new(force_change_set_id: Option<ChangeSetId>, response: T) -> Self {
        Self {
            force_change_set_id,
            response: Some(response),
        }
    }

    pub fn empty(force_change_set_id: Option<ChangeSetId>) -> Self {
        Self {
            force_change_set_id,
            response: None,
        }
    }
}

const APPLICATION_JSON: &str = "application/json";
const TEXT_PLAIN_UTF_8: &str = "text/plain; charset=utf-8";
const FORCE_CHANGE_SET_HEADER: &str = "force_change_set_id";

impl<T> IntoResponse for ForceChangeSetResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();

        if let Some(force_change_set_id) = self.force_change_set_id {
            headers.insert(
                HeaderName::from_static(FORCE_CHANGE_SET_HEADER),
                match HeaderValue::from_str(&force_change_set_id.to_string()) {
                    Ok(header_value) => header_value,
                    Err(err) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            [(header::CONTENT_TYPE, TEXT_PLAIN_UTF_8)],
                            err.to_string(),
                        )
                            .into_response();
                    }
                },
            );
        }

        match self.response {
            Some(response) => {
                let mut buf = BytesMut::with_capacity(128).writer();
                match serde_json::to_writer(&mut buf, &response) {
                    Ok(()) => {
                        headers.insert(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static(APPLICATION_JSON),
                        );

                        (headers, buf.into_inner().freeze()).into_response()
                    }
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [(header::CONTENT_TYPE, TEXT_PLAIN_UTF_8)],
                        err.to_string(),
                    )
                        .into_response(),
                }
            }
            None => (headers, ()).into_response(),
        }
    }
}
