use async_nats::jetstream::message;

/// Information about a received message
#[derive(Debug, Clone)]
pub struct Info {
    /// Optional domain, present in servers post-ADR-15
    pub domain: Option<String>,
    /// Optional account hash, present in servers post-ADR-15
    pub acc_hash: Option<String>,
    /// The stream name
    pub stream: String,
    /// The consumer name
    pub consumer: String,
    /// The stream sequence number associated with this message
    pub stream_sequence: u64,
    /// The consumer sequence number associated with this message
    pub consumer_sequence: u64,
    /// The number of delivery attempts for this message
    pub delivered: i64,
    /// The number of messages known by the server to be pending to this consumer
    pub pending: u64,
    /// The time that this message was received by the server from its publisher
    pub published: time::OffsetDateTime,
    /// Optional token, present in servers post-ADR-15
    pub token: Option<String>,
}

impl From<message::Info<'_>> for Info {
    fn from(value: message::Info<'_>) -> Self {
        Self {
            domain: value.domain.map(str::to_string),
            acc_hash: value.acc_hash.map(str::to_string),
            stream: value.stream.to_string(),
            consumer: value.consumer.to_string(),
            stream_sequence: value.stream_sequence,
            consumer_sequence: value.consumer_sequence,
            delivered: value.delivered,
            pending: value.pending,
            published: value.published,
            token: value.token.map(str::to_string),
        }
    }
}
