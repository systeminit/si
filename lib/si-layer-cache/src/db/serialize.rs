use crate::error::LayerDbResult;
use crate::LayerDbError;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[inline]
pub fn to_vec<T>(value: &T) -> LayerDbResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let serialized = postcard::to_stdvec(value)?;
    // 1 is the best speed, 6 is default, 9 is best compression but may be too slow
    let compressed = miniz_oxide::deflate::compress_to_vec(&serialized, 1);

    Ok(compressed)
}

#[inline]
pub fn from_bytes<T>(bytes: &[u8]) -> LayerDbResult<T>
where
    T: DeserializeOwned,
{
    let uncompressed = miniz_oxide::inflate::decompress_to_vec(bytes)
        .map_err(|e| LayerDbError::Decompress(e.to_string()))?;
    Ok(postcard::from_bytes(&uncompressed)?)
}
