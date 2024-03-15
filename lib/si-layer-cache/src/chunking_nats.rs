// TODO(fnichol): worth thinking about:
//
// - a "keep-alive" for chunks that we have received but are still ultimately un-acked because we
// haven't reached the end of the message chunks.
// - extra book-keeping to write chunks into the correct position in the buffer as well as knowing
// if any chunks are outstanding. If we know this, we can deal with some chunked messages being
// redelivered due to timeouts/nacks/etc.

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt, ops,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::{BufMut, Bytes, BytesMut};
use futures::{Future, Stream, StreamExt};
use si_data_nats::{
    async_nats::jetstream, subject::ToSubject, HeaderMap, InnerError, InnerMessage,
};
use ulid::Ulid;

use crate::error::{LayerDbError, LayerDbResult};

const DEFAULT_CHUNK_SIZE: usize = 128 * 1024;

const HEADER_EVENT_ID: &str = "X-EVENT-ID";
const HEADER_CHECKSUM: &str = "X-CHECKSUM";
const HEADER_SIZE: &str = "X-SIZE";
const HEADER_NUM_CHUNKS: &str = "X-NUM-CHUNKS";
const HEADER_CUR_CHUNK: &str = "X-CUR-CHUNK";

#[derive(Clone, Debug)]
pub struct ChunkingNats {
    context: jetstream::context::Context,
    chunk_size: Option<usize>,
    prefix: Option<Arc<str>>,
}

impl ChunkingNats {
    #[inline]
    pub fn new(prefix: Option<String>, context: jetstream::context::Context) -> Self {
        Self {
            prefix: prefix.map(|s| s.into()),
            context,
            chunk_size: None,
        }
    }

    pub fn with_chunk_size(
        prefix: Option<String>,
        context: jetstream::context::Context,
        chunk_size: usize,
    ) -> Self {
        Self {
            prefix: prefix.map(|s| s.into()),
            context,
            chunk_size: Some(chunk_size),
        }
    }

    // Note: heavily adapted from `async-nats`'s ObjectStore `put` method
    //
    // See: https://github.com/nats-io/nats.rs/blob/09e3cfdacad26fa837917a48df56aad5345a4833/async-nats/src/jetstream/object_store/mod.rs#L262-L409
    pub async fn publish_with_headers(
        &self,
        subject: impl ToSubject,
        mut headers: HeaderMap,
        payload: Bytes,
    ) -> LayerDbResult<()> {
        let subject = subject.to_subject();

        let event_id = Ulid::new();
        let object_size = payload.len();
        let chunk_size = self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        let checksum = blake3::hash(&payload);
        let num_chunks = object_size.div_ceil(chunk_size);

        headers.insert(HEADER_EVENT_ID, event_id.to_string().as_str());
        headers.insert(HEADER_SIZE, object_size.to_string().as_str());
        headers.insert(HEADER_CHECKSUM, checksum.to_string().as_str());
        headers.insert(HEADER_NUM_CHUNKS, num_chunks.to_string().as_str());

        let mut cur_chunk = 1;

        for chunk in payload.chunks(chunk_size) {
            let mut headers = headers.clone();
            headers.insert(HEADER_CUR_CHUNK, cur_chunk.to_string().as_str());

            self.context
                .publish_with_headers(subject.clone(), headers, Bytes::copy_from_slice(chunk))
                .await?
                .await?;

            cur_chunk += 1;
        }

        Ok(())
    }

    pub fn chunking_messages(messages: jetstream::consumer::pull::Stream) -> ChunkedMessagesStream {
        ChunkedMessagesStream {
            inner: messages,
            buffers: HashMap::new(),
        }
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

pub struct ChunkedMessagesStream {
    inner: jetstream::consumer::pull::Stream,
    buffers: HashMap<String, (BytesMut, Vec<jetstream::message::Acker>)>,
}

impl ChunkedMessagesStream {
    fn process_single_message(
        &mut self,
        msg: jetstream::Message,
    ) -> Option<Poll<Option<Result<Message, LayerDbError>>>> {
        let context = msg.context.clone();
        let (message, acker) = msg.split();

        match &message.headers {
            Some(headers) => {
                match (
                    headers.get(HEADER_EVENT_ID),
                    headers.get(HEADER_CUR_CHUNK),
                    headers.get(HEADER_NUM_CHUNKS),
                ) {
                    // If we don't see an event id header, then it's a "normal" message so return
                    // it immediately
                    (None, _, _) => Some(Poll::Ready(Some(Ok(Message {
                        inner: jetstream::Message { message, context },
                        ackers: Arc::new(vec![acker]),
                    })))),
                    // If the number of chunks is 1/1 then we can return immediately (i.e. the was
                    // no chunking!)
                    (Some(_), Some(cur_chunk), Some(num_chunks))
                        if cur_chunk.as_str() == "1" && num_chunks.as_str() == "1" =>
                    {
                        Some(Poll::Ready(Some(Ok(Message {
                            inner: jetstream::Message { message, context },
                            ackers: Arc::new(vec![acker]),
                        }))))
                    }
                    // We have a legit chunked message fragement
                    (Some(event_id), Some(cur_chunk), Some(num_chunks)) => {
                        // Append bytes from a chunked message into an incomplete buffer, keyed on
                        // the event it
                        match self.buffers.entry(event_id.to_string()) {
                            // Entry found for the event id
                            Entry::Occupied(occupied) => {
                                let value = occupied.into_mut();
                                value.0.put(message.payload);
                                value.1.push(acker);
                            }
                            // Entry not found for the event id
                            Entry::Vacant(vacant) => {
                                // Determine the size of the full un-chunked message from header metadata
                                let size: usize = {
                                    let size_value = match headers
                                        .get(HEADER_SIZE)
                                        .ok_or(LayerDbError::NatsMissingSizeHeader)
                                    {
                                        Ok(val) => val,
                                        Err(err) => {
                                            return Some(Poll::Ready(Some(Err(err))));
                                        }
                                    };

                                    match str::parse(size_value.as_str())
                                        .map_err(LayerDbError::nats_header_parse)
                                    {
                                        Ok(val) => val,
                                        Err(err) => {
                                            return Some(Poll::Ready(Some(Err(err))));
                                        }
                                    }
                                };

                                let mut buffer = BytesMut::with_capacity(size);
                                buffer.put(message.payload);

                                vacant.insert((buffer, vec![acker]));
                            }
                        };

                        if cur_chunk == num_chunks {
                            // We're at the final message chunk so we can return a reassembled
                            // messages

                            // Move the payload bytes out of the buffers map
                            let (payload, ackers) = {
                                let entry = match self
                                    .buffers
                                    .remove(event_id.as_str())
                                    .ok_or(LayerDbError::MissingInternalBuffer)
                                {
                                    Ok(buf) => buf,
                                    Err(err) => {
                                        return Some(Poll::Ready(Some(Err(err))));
                                    }
                                };

                                (entry.0.freeze(), entry.1)
                            };

                            // TODO(fnichol): it's unclear whether Message.length is the size of
                            // the entire message (i.e. envelope and payload) or simply the
                            // size of the payload. For the moment, let's assume it's the
                            // payload
                            let length = payload.len();

                            // Construct a new Jetstream message to return with our fully
                            // reassembled payload
                            let new_msg = Message {
                                inner: jetstream::Message {
                                    message: InnerMessage {
                                        subject: message.subject,
                                        reply: message.reply,
                                        payload,
                                        headers: message.headers,
                                        status: message.status,
                                        description: message.description,
                                        length,
                                    },
                                    context,
                                },
                                ackers: Arc::new(ackers),
                            };

                            Some(Poll::Ready(Some(Ok(new_msg))))
                        } else {
                            // Still more chunks to go with this message, so try for more
                            None
                        }
                    }
                    // In these other cases, we have incomplete headers and this message is
                    // considered malformed
                    (Some(_), None, None) | (Some(_), None, Some(_)) | (Some(_), Some(_), None) => {
                        Some(Poll::Ready(Some(Err(LayerDbError::NatsMalformedHeaders))))
                    }
                }
            }
            // No headers means this is a "normal" message so return it immediately
            None => Some(Poll::Ready(Some(Ok(Message {
                inner: jetstream::Message { message, context },
                ackers: Arc::new(vec![acker]),
            })))),
        }
    }
}

impl Stream for ChunkedMessagesStream {
    type Item = Result<Message, LayerDbError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let poll = Pin::new(&mut self.inner.next()).poll(cx);

            match poll {
                // Process the single message
                Poll::Ready(Some(Ok(msg))) => {
                    if let Some(poll) = self.process_single_message(msg) {
                        return poll;
                    }
                }
                // Upstream errors are propagated downstream
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err.into()))),
                // If the upstream closes, then we do too
                Poll::Ready(None) => return Poll::Ready(None),
                // Not ready, so...not ready!
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[derive(Clone)]
pub struct Message {
    inner: jetstream::Message,
    ackers: Arc<Vec<jetstream::message::Acker>>,
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Message {
    pub async fn ack(&self) -> Result<(), InnerError> {
        for acker in self.ackers.iter() {
            acker.ack().await?;
        }
        Ok(())
    }

    pub async fn ack_with(&self, kind: jetstream::AckKind) -> Result<(), InnerError> {
        for acker in self.ackers.iter() {
            acker.ack_with(kind).await?;
        }
        Ok(())
    }

    pub async fn double_ack(&self) -> Result<(), InnerError> {
        for acker in self.ackers.iter() {
            acker.double_ack().await?;
        }
        Ok(())
    }
}

impl ops::Deref for Message {
    type Target = jetstream::Message;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
