use crate::CeaResult;
use async_trait::async_trait;
use futures::StreamExt;
use si_data::Db;
use si_transport::{AgentData, AgentDataTopic, Header, QoS, Topic, Transport, WireMessage};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tracing::warn;

/// Quality of service level for topic subscriptions.
///
/// A `FinalizedListener` will finalize objects, and this operation is assumed to be either
/// [idempotent] or at least eventually consistent. That is, in the event that there are multiple
/// finalize messages for an object, this operation should succeed when called more than once.
///
/// Therefore, all `FinalizedListener` subscriptions will be `QoS::AtLeastOnce` to guarantee
/// delivery of at least *one* message.
///
/// [idempotent]: https://en.wikipedia.org/wiki/Idempotence
const SUBSCRIBE_QOS: QoS = QoS::AtLeastOnce;

#[async_trait]
pub trait Finalize {
    async fn finalize(&self, db: &Db, message: WireMessage) -> CeaResult<()>;
    fn finalize_key(&self) -> FinalizeKey;
}

pub trait Finalizeable: Finalize + Sync + Send {}

pub trait FinalizeBuilder {
    type Finalizeable: Finalizeable;

    fn new() -> Self;

    fn finalize_key(&mut self, finalize_key: FinalizeKey) -> &mut Self;
    fn build(self) -> CeaResult<Self::Finalizeable>;

    fn object_type(&self) -> &'static str;
}

pub struct FinalizedListener {
    transport: Transport,
    db: Db,
    finalizers: HashMap<Topic, Arc<dyn Finalizeable>>,
}

impl FinalizedListener {
    pub fn builder<'a>(
        server_name: impl Into<Cow<'a, str>>,
        transport_server_uri: impl Into<String>,
        shared_topic_id: impl Into<String>,
        db: Db,
    ) -> FinalizedListenerBuilder<'a> {
        FinalizedListenerBuilder {
            server_name: server_name.into(),
            transport_server_uri: transport_server_uri.into(),
            shared_topic_id: shared_topic_id.into(),
            db,
            finalizers: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> CeaResult<()> {
        let mut messages = self
            .transport
            .subscribe_to(self.subscriptions())
            .await?
            .messages();
        let mut finalizer_topics: Vec<_> = self.finalizers.keys().collect();
        finalizer_topics.sort_unstable();

        while let Some(message) = messages.next().await {
            let topic = match match_topic(message.header(), &finalizer_topics) {
                Some(key) => key,
                None => {
                    warn!(
                        "no finalizer to handle message from subscription, topic={}",
                        message.header()
                    );
                    continue;
                }
            };

            match self.finalizers.get(topic) {
                Some(finalizer) => {
                    tokio::spawn(finalize_message(
                        self.db.clone(),
                        finalizer.clone(),
                        message,
                    ));
                }
                None => {
                    warn!(
                        "no finalizer to handle message from subscription, topic={}",
                        message.header()
                    );
                    continue;
                }
            }
        }

        Ok(())
    }

    fn subscriptions(&self) -> Vec<(Topic, QoS)> {
        let mut subscriptions = Vec::new();
        for topic in self.finalizers.keys() {
            subscriptions.push((topic.clone(), SUBSCRIBE_QOS));
        }

        subscriptions
    }
}

pub struct FinalizedListenerBuilder<'a> {
    server_name: Cow<'a, str>,
    transport_server_uri: String,
    db: Db,
    shared_topic_id: String,
    finalizers: HashMap<FinalizeKey, Arc<dyn Finalizeable>>,
}

impl<'a> FinalizedListenerBuilder<'a> {
    pub fn finalizer(&mut self, finalizer: impl Finalizeable + 'static) -> &mut Self {
        self.finalizers
            .insert(finalizer.finalize_key(), Arc::new(finalizer));
        self
    }

    pub async fn build(self) -> CeaResult<FinalizedListener> {
        let transport = Transport::create(
            self.transport_server_uri,
            format!("finalized_listener:{}", self.server_name),
        )
        .await?;

        let mut finalizers = HashMap::new();
        for (finalize_key, finalizer) in self.finalizers.into_iter() {
            finalizers.insert(
                AgentDataTopic::builder()
                    .shared_id(&self.shared_topic_id)
                    .object_type(finalize_key.object_type)
                    .data(AgentData::Finalize)
                    .build()
                    .into(),
                finalizer,
            );
        }

        Ok(FinalizedListener {
            transport,
            db: self.db,
            finalizers,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FinalizeKey {
    object_type: String,
}

impl FinalizeKey {
    pub fn new(object_type: impl Into<String>) -> Self {
        Self {
            object_type: object_type.into(),
        }
    }
}

fn match_topic<'a>(header: &Header, topics: &[&'a Topic]) -> Option<&'a Topic> {
    topics
        .iter()
        .find(|topic| header.satisfies(topic))
        .map(|topic| *topic)
}

async fn finalize_message(
    db: Db,
    finalizer: Arc<dyn Finalizeable>,
    message: WireMessage,
) -> CeaResult<()> {
    finalizer.finalize(&db, message).await?;
    Ok(())
}
