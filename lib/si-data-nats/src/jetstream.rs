pub use super::{Client, Message};

// Re-export JetStream types. Since this is a private module, we'll have to name them from
// `nats::jetstream` :(
pub use nats::jetstream::{
    AccountInfo, AccountLimits, AckKind, AckPolicy, ApiStats, ClusterInfo, ConsumerConfig,
    ConsumerInfo, DateTime, DeliverPolicy, DiscardPolicy, JetStreamMessageInfo, PurgeResponse,
    ReplayPolicy, RetentionPolicy, SequencePair, StorageType, StreamConfig, StreamInfo,
    StreamState,
};
