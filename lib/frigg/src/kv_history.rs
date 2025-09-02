// Internal impl of a `Watch` type vendored from the `async-nats` crate.
//
// See: https://github.com/nats-io/nats.rs/blob/7d63f1dd725c86a4f01723ea3194f17e30a0561b/async-nats/src/jetstream/kv/mod.rs#L1263-L1323

use std::{
    result,
    str::FromStr as _,
    task::Poll,
};

use futures::StreamExt as _;
use si_data_nats::async_nats::{
    self,
    jetstream::{
        consumer::push::{
            Ordered,
            OrderedError,
        },
        kv::{
            Entry,
            Operation,
            ParseOperationError,
            WatcherErrorKind,
        },
    },
};
use thiserror::Error;

const KV_OPERATION: &str = "KV-Operation";

/// A structure representing the history of a key-value bucket, yielding past values.
pub struct History {
    pub subscription: Ordered,
    pub done: bool,
    pub prefix: String,
    pub bucket: String,
}

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("{0}")]
    Default(WatcherErrorKind, String),
    #[error("{0}")]
    Ordered(#[from] OrderedError),
}

impl futures::Stream for History {
    type Item = result::Result<Entry, WatcherError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }
        match self.subscription.poll_next_unpin(cx) {
            Poll::Ready(message) => match message {
                None => Poll::Ready(None),
                Some(message) => {
                    let message = message?;
                    let info = message.info().map_err(|err| {
                        WatcherError::Default(
                            WatcherErrorKind::Other,
                            format!("failed to parse message metadata: {err}"),
                        )
                    })?;
                    if info.pending == 0 {
                        self.done = true;
                    }

                    let operation = kv_operation_from_message(&message).unwrap_or(Operation::Put);

                    let key = message
                        .subject
                        .strip_prefix(&self.prefix)
                        .map(|s| s.to_string())
                        .unwrap();

                    Poll::Ready(Some(Ok(Entry {
                        bucket: self.bucket.clone(),
                        key,
                        value: message.payload.clone(),
                        revision: info.stream_sequence,
                        created: info.published,
                        delta: info.pending,
                        operation,
                        seen_current: self.done,
                    })))
                }
            },
            std::task::Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub struct Keys {
    pub inner: History,
}

impl futures::Stream for Keys {
    type Item = Result<String, WatcherError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            match self.inner.poll_next_unpin(cx) {
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(res)) => match res {
                    Ok(entry) => {
                        // Skip purged and deleted keys
                        if matches!(entry.operation, Operation::Purge | Operation::Delete) {
                            // Try to poll again if we skip this one
                            continue;
                        } else {
                            return Poll::Ready(Some(Ok(entry.key)));
                        }
                    }
                    Err(e) => return Poll::Ready(Some(Err(e))),
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn kv_operation_from_message(
    message: &async_nats::message::Message,
) -> Result<Operation, ParseOperationError> {
    let headers = match message.headers.as_ref() {
        Some(headers) => headers,
        None => return Ok(Operation::Put),
    };
    if let Some(op) = headers.get(KV_OPERATION) {
        Operation::from_str(op.as_str())
    } else {
        Ok(Operation::Put)
    }
}
