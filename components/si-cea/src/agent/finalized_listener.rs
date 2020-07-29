use crate::CeaResult;
use async_trait::async_trait;
use futures::StreamExt;
use si_data::Db;
use si_transport::{AgentData, AgentDataTopic, QoS, Topic, Transport, WireMessage};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tracing::warn;

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
    finalizers: HashMap<String, Arc<dyn Finalizeable>>,
}

impl FinalizedListener {
    pub fn builder<'a>(
        server_name: impl Into<Cow<'a, str>>,
        transport_server_uri: impl Into<String>,
        db: Db,
    ) -> FinalizedListenerBuilder<'a> {
        FinalizedListenerBuilder {
            server_name: server_name.into(),
            transport_server_uri: transport_server_uri.into(),
            db,
            finalizers: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> CeaResult<()> {
        let mut messages = self
            .transport
            .subscribe_to(self.subscriptions()?)
            .await?
            .messages();

        while let Some(message) = messages.next().await {
            match self.finalizers.get(message.topic_str()) {
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
                        message.topic_str()
                    );
                    continue;
                }
            }
        }

        Ok(())
    }

    fn subscriptions(&self) -> CeaResult<Vec<(Topic, QoS)>> {
        let mut subscriptions = Vec::new();
        for topic_str in self.finalizers.keys() {
            subscriptions.push((topic_str.parse()?, QoS::ExactlyOnce));
        }

        Ok(subscriptions)
    }
}

pub struct FinalizedListenerBuilder<'a> {
    server_name: Cow<'a, str>,
    transport_server_uri: String,
    db: Db,
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
                    .shared(true)
                    .object_type(finalize_key.object_type)
                    .data(AgentData::Finalize)
                    .build()
                    .to_string(),
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

async fn finalize_message(
    db: Db,
    finalizer: Arc<dyn Finalizeable>,
    message: WireMessage,
) -> CeaResult<()> {
    finalizer.finalize(&db, message).await?;
    Ok(())
}
