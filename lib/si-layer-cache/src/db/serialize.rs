use serde::{Serialize, de::DeserializeOwned};
use telemetry::prelude::*;

use crate::{LayerDbError, error::LayerDbResult};

#[inline]
#[instrument(
    name = "serialize.to_vec",
    level = "debug",
    skip_all,
    fields(
        bytes.size.compressed = Empty,
        bytes.size.uncompressed = Empty,
    )
)]
pub fn to_vec<T>(value: &T) -> LayerDbResult<(Vec<u8>, usize)>
where
    T: Serialize + ?Sized,
{
    let span = current_span_for_instrument_at!("debug");

    let serialized = postcard::to_stdvec(value)?;
    let uncompressed_size = serialized.len();
    // 1 is the best speed, 6 is default, 9 is best compression but may be too slow
    let compressed = miniz_oxide::deflate::compress_to_vec(&serialized, 1);

    span.record("bytes.size.compressed", compressed.len());
    span.record("bytes.size.uncompressed", uncompressed_size);

    Ok((compressed, uncompressed_size))
}

#[inline]
#[instrument(
    name = "serialize.from_bytes",
    level = "trace",
    skip_all,
    fields(
        bytes.size = bytes.len(),
    )
)]
pub fn from_bytes<T>(bytes: &[u8]) -> LayerDbResult<T>
where
    T: DeserializeOwned,
{
    let uncompressed = miniz_oxide::inflate::decompress_to_vec(bytes)
        .map_err(|e| LayerDbError::Decompress(e.to_string()))?;

    Ok(postcard::from_bytes(&uncompressed)?)
}

#[inline]
#[instrument(
    name = "serialize.from_bytes_async",
    level = "trace",
    skip_all,
    fields(
        bytes.size = bytes.len(),
    )
)]
pub async fn from_bytes_async<T>(bytes: &[u8]) -> LayerDbResult<T>
where
    T: DeserializeOwned,
{
    let uncompressed = miniz_oxide::inflate::decompress_to_vec(bytes)
        .map_err(|e| LayerDbError::Decompress(e.to_string()))?;

    tokio::task::yield_now().await;

    Ok(postcard::from_bytes(&uncompressed)?)
}

pub fn decompress_to_vec(compressed_bytes: &[u8]) -> LayerDbResult<Vec<u8>> {
    let uncompressed = miniz_oxide::inflate::decompress_to_vec(compressed_bytes)
        .map_err(|e| LayerDbError::Decompress(e.to_string()))?;
    Ok(uncompressed)
}
