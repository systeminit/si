//! Results returned from operations.

use std::fmt;
use std::str;

use crate::error::CouchbaseError;
use futures::channel::{mpsc, oneshot};
use futures::stream::StreamExt;
use futures::Stream;
use futures;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use serde_json::{from_slice, Value};

pub struct GetResult {
    cas: u64,
    encoded: Vec<u8>,
    flags: u32,
}

impl GetResult {
    pub fn new(cas: u64, encoded: Vec<u8>, flags: u32) -> Self {
        Self {
            cas,
            encoded,
            flags,
        }
    }

    pub fn cas(&self) -> u64 {
        self.cas
    }

    pub fn content_as<'a, T>(&'a self) -> Result<T, CouchbaseError>
    where
        T: serde::Deserialize<'a>,
    {
        match from_slice(&self.encoded.as_slice()) {
            Ok(v) => Ok(v),
            Err(e) => Err(CouchbaseError::DecodingError(e)),
        }
    }
}

impl fmt::Debug for GetResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GetResult {{ cas: 0x{:x}, flags: 0x{:x}, encoded: {} }}",
            self.cas,
            self.flags,
            str::from_utf8(&self.encoded).unwrap()
        )
    }
}

pub struct MutationResult {
    cas: u64,
}

impl MutationResult {
    pub fn new(cas: u64) -> Self {
        Self { cas }
    }

    pub fn cas(&self) -> u64 {
        self.cas
    }
}

impl fmt::Debug for MutationResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MutationResult {{ cas: 0x{:x} }}", self.cas)
    }
}

pub struct ExistsResult {
    cas: u64,
}

impl ExistsResult {
    pub fn new(cas: u64) -> Self {
        Self { cas }
    }

    pub fn cas(&self) -> u64 {
        self.cas
    }
}

impl fmt::Debug for ExistsResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExistsResult {{ cas: 0x{:x} }}", self.cas)
    }
}

#[derive(Debug)]
pub struct QueryResult {
    rows: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    meta: Option<oneshot::Receiver<Vec<u8>>>,
}

#[derive(Debug, Deserialize)]
pub struct QueryMeta {
    #[serde(rename = "requestID")]
    pub request_id: String,
    pub status: String,
    pub metrics: QueryMetrics,
    pub errors: Option<Value>,
    #[serde(rename = "clientContextID")]
    pub client_context_id: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryMetrics {
    #[serde(rename = "elapsedTime")]
    pub elapsed_time: String,
    #[serde(rename = "executionTime")]
    pub execution_time: String,
    #[serde(rename = "resultCount")]
    pub result_count: usize,
    #[serde(rename = "resultSize")]
    pub result_size: usize,
}

impl QueryResult {
    pub fn new(rows: mpsc::UnboundedReceiver<Vec<u8>>, meta: oneshot::Receiver<Vec<u8>>) -> Self {
        Self {
            rows: Some(rows),
            meta: Some(meta),
        }
    }

    pub fn rows_as<T>(
        &mut self,
    ) -> Result<impl Stream<Item = Result<T, CouchbaseError>>, CouchbaseError>
    where
        T: DeserializeOwned,
    {
        if let Some(stream) = self.rows.take() {
            return Ok(stream.map(|v| {
                let f = from_slice::<T>(v.as_slice());
                match f {
                    Ok(r) => Ok(r),
                    Err(e) => Err(CouchbaseError::DecodingError(e)),
                }
            }));
        } else {
            return Err(CouchbaseError::RowsConsumed);
        }
    }

    pub async fn meta(&mut self) -> Result<QueryMeta, CouchbaseError> {
        match self.meta.take() {
            Some(mv) => {
                let rmv = mv.await?;
                from_slice::<QueryMeta>(rmv.as_slice()).map_err(CouchbaseError::DecodingError)
            },
            None => Err(CouchbaseError::MetaConsumed)
        }
    }
}

#[derive(Debug)]
pub struct AnalyticsResult {
    rows: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    meta: Option<oneshot::Receiver<Vec<u8>>>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsMeta {
    #[serde(rename = "requestID")]
    request_id: String,
    status: String,
    metrics: AnalyticsMetrics,
    errors: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsMetrics {
    #[serde(rename = "elapsedTime")]
    elapsed_time: String,
    #[serde(rename = "executionTime")]
    execution_time: String,
    #[serde(rename = "resultCount")]
    result_count: usize,
    #[serde(rename = "resultSize")]
    result_size: usize,
}

impl AnalyticsResult {
    pub fn new(rows: mpsc::UnboundedReceiver<Vec<u8>>, meta: oneshot::Receiver<Vec<u8>>) -> Self {
        Self {
            rows: Some(rows),
            meta: Some(meta),
        }
    }

    pub fn rows_as<T>(
        &mut self,
    ) -> Result<impl Stream<Item = Result<T, CouchbaseError>>, CouchbaseError>
    where
        T: DeserializeOwned,
    {
        if let Some(stream) = self.rows.take() {
            return Ok(stream.map(|v| {
                let f = from_slice::<T>(v.as_slice());
                match f {
                    Ok(r) => Ok(r),
                    Err(e) => Err(CouchbaseError::DecodingError(e)),
                }
            }));
        } else {
            return Err(CouchbaseError::RowsConsumed);
        }
    }

    pub async fn meta(&mut self) -> Result<AnalyticsMeta, CouchbaseError> {
        match self.meta.take() {
            Some(mv) => {
                let rmv = mv.await?;
                from_slice::<AnalyticsMeta>(rmv.as_slice()).map_err(CouchbaseError::DecodingError)

            },
            None => Err(CouchbaseError::MetaConsumed)
        }
    }
}

#[derive(Debug)]
pub struct LookupInResult {
    cas: u64,
    fields: Vec<LookupInField>,
}

impl LookupInResult {
    pub(crate) fn new(cas: u64, fields: Vec<LookupInField>) -> Self {
        LookupInResult { cas, fields }
    }
}

#[derive(Debug)]
pub struct LookupInField {
    status: CouchbaseError,
    value: Vec<u8>,
}

impl LookupInField {
    pub fn new(status: CouchbaseError, value: Vec<u8>) -> Self {
        LookupInField { status, value }
    }
}

#[derive(Debug)]
pub struct MutateInResult {
    cas: u64,
    fields: Vec<MutateInField>,
}

impl MutateInResult {
    pub(crate) fn new(cas: u64, fields: Vec<MutateInField>) -> Self {
        MutateInResult { cas, fields }
    }
}

#[derive(Debug)]
pub struct MutateInField {
    status: CouchbaseError,
    value: Vec<u8>,
}

impl MutateInField {
    pub fn new(status: CouchbaseError, value: Vec<u8>) -> Self {
        MutateInField { status, value }
    }
}
