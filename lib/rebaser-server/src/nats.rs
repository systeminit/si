use si_data_nats::{
    async_nats::jetstream::{
        context::{CreateKeyValueError, KeyValueError, KeyValueErrorKind},
        kv,
    },
    jetstream,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum GetOrCreateKeyValueError {
    #[error("create key value bucket error: {0}")]
    CreateKeyValue(#[from] CreateKeyValueError),
    #[error("get key value bucket error: {0}")]
    KeyValue(#[from] KeyValueError),
}

pub(crate) async fn get_or_create_key_value(
    context: &jetstream::Context,
    kv_config: kv::Config,
) -> Result<kv::Store, GetOrCreateKeyValueError> {
    match context.get_key_value(kv_config.bucket.as_str()).await {
        Ok(store) => Ok(store),
        Err(err) if matches!(err.kind(), KeyValueErrorKind::GetBucket) => context
            .create_key_value(kv_config)
            .await
            .map_err(Into::into),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn nats_stream_name(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();

    match prefix {
        Some(prefix) => format!("{prefix}_{suffix}"),
        None => suffix.to_owned(),
    }
}
