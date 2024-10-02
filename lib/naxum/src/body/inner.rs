use bytes::Bytes;

pub trait Body: Into<Bytes> {}
