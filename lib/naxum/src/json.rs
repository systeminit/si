use async_trait::async_trait;
use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::{
    extract::{
        FromMessage,
        rejection::{JsonDataError, JsonRejection, JsonSyntaxError},
    },
    message::{Message, MessageHead},
};

#[derive(Clone, Copy, Default, Debug)]
#[must_use]
pub struct Json<T>(pub T);

#[async_trait]
impl<T, S, R> FromMessage<S, R> for Json<T>
where
    T: DeserializeOwned,
    R: MessageHead + Send + 'static,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_message(req: Message<R>, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_message(req, state)
            .await
            .expect("from_message is infallible");
        Self::from_bytes(&bytes)
    }
}

impl<T> Json<T>
where
    T: DeserializeOwned,
{
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JsonRejection> {
        let deserializer = &mut serde_json::Deserializer::from_slice(bytes);

        let value = match serde_path_to_error::deserialize(deserializer) {
            Ok(value) => value,
            Err(err) => {
                let rejection = match err.inner().classify() {
                    serde_json::error::Category::Data => JsonDataError::from_err(err).into(),
                    serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                        JsonSyntaxError::from_err(err).into()
                    }
                    serde_json::error::Category::Io => {
                        if cfg!(debug_assertions) {
                            // We don't use `serde_json::from_reader` and instead always buffer
                            // bodies first, so we shouldn't encounter any IO errors
                            unreachable!()
                        } else {
                            JsonSyntaxError::from_err(err).into()
                        }
                    }
                };
                return Err(rejection);
            }
        };

        Ok(Json(value))
    }
}
