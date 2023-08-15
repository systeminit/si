use futures::StreamExt;
use si_data_nats::{NatsClient, Subscription};
use std::time::Duration;
use telemetry::prelude::*;

use crate::{Graph, Id, Request, Response};

#[remain::sorted]
#[derive(Debug)]
pub enum State {
    Continue,
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct PubClient {
    change_set_id: Id,
    pub_channel: String,
    reply_channel: String,
    nats: NatsClient,
}

impl PubClient {
    pub async fn finished_creating_values(&self) -> Result<()> {
        let message = serde_json::to_vec(&Request::ValueCreationDone)?;
        self.nats
            .publish_with_reply(&self.pub_channel, &self.reply_channel, message)
            .await?;
        Ok(())
    }

    pub async fn register_dependency_graph(&self, dependency_graph: Graph) -> Result<()> {
        let message = serde_json::to_vec(&Request::ValueDependencyGraph {
            change_set_id: self.change_set_id,
            dependency_graph,
        })?;
        self.nats
            .publish_with_reply(&self.pub_channel, &self.reply_channel, message)
            .await?;
        Ok(())
    }

    pub async fn processed_value(&self, node_id: Id) -> Result<()> {
        let message = serde_json::to_vec(&Request::ProcessedValue {
            change_set_id: self.change_set_id,
            node_id,
        })?;
        self.nats
            .publish_with_reply(&self.pub_channel, &self.reply_channel, message)
            .await?;
        Ok(())
    }

    pub async fn failed_processing_value(&self, node_id: Id) -> Result<()> {
        let message = serde_json::to_vec(&Request::ValueProcessingFailed {
            change_set_id: self.change_set_id,
            node_id,
        })?;
        self.nats
            .publish_with_reply(&self.pub_channel, &self.reply_channel, message)
            .await?;
        Ok(())
    }

    pub async fn bye(self) -> Result<()> {
        let message = serde_json::to_vec(&Request::Bye {
            change_set_id: self.change_set_id,
        })?;
        self.nats
            .publish_with_reply(&self.pub_channel, &self.reply_channel, message)
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Client {
    change_set_id: Id,
    pub_channel: String,
    reply_channel: String,
    subscription: Subscription,
    nats: NatsClient,
}

impl Client {
    pub async fn new(
        nats: NatsClient,
        subject_prefix: &str,
        id: Id,
        change_set_id: Id,
    ) -> Result<Self> {
        let pub_channel = format!("{subject_prefix}.{id}");
        let reply_channel = format!("{pub_channel}.reply");
        Ok(Self {
            pub_channel,
            change_set_id,
            subscription: nats.subscribe(&reply_channel).await?,
            reply_channel,
            nats,
        })
    }

    pub fn clone_into_pub(&self) -> PubClient {
        PubClient {
            pub_channel: self.pub_channel.clone(),
            reply_channel: self.reply_channel.clone(),
            change_set_id: self.change_set_id,
            nats: self.nats.clone(),
        }
    }

    // None means subscription has been unsubscribed or that the connection has been closed
    pub async fn fetch_response(&mut self) -> Result<Option<Response>> {
        // TODO: timeout so we don't get stuck here forever if council goes away
        // TODO: handle message.data() empty with Status header as 503: https://github.com/nats-io/nats.go/pull/576
        let msg = loop {
            let res = tokio::time::timeout(Duration::from_secs(60), self.subscription.next()).await;

            match res {
                Ok(msg) => break msg,
                Err(_) => {
                    warn!(change_set_id = ?self.change_set_id, pub_channel = ?self.pub_channel, reply_channel = ?self.reply_channel, "Council client waiting for response for 60 seconds");
                }
            }
        };

        match msg {
            Some(msg) => {
                if msg.data().is_empty() {
                    return Err(Error::NoListenerAvailable);
                }
                Ok(Some(serde_json::from_slice::<Response>(msg.data())?))
            }
            None => Ok(None),
        }
    }

    pub async fn wait_to_create_values(&mut self) -> Result<State> {
        let message = serde_json::to_vec(&Request::CreateValues)?;
        self.nats
            .publish_with_reply(&self.pub_channel, &self.reply_channel, message)
            .await?;

        match self.fetch_response().await? {
            Some(Response::OkToCreate) => Ok(State::Continue),
            Some(Response::Shutdown) => Ok(State::Shutdown),
            resp => unreachable!("{:?}", resp),
        }
    }

    pub async fn finished_creating_values(&self) -> Result<()> {
        self.clone_into_pub().finished_creating_values().await
    }

    pub async fn register_dependency_graph(&self, dependency_graph: Graph) -> Result<()> {
        self.clone_into_pub()
            .register_dependency_graph(dependency_graph)
            .await
    }

    pub async fn processed_value(&self, node_id: Id) -> Result<()> {
        self.clone_into_pub().processed_value(node_id).await
    }

    pub async fn bye(&self) -> Result<()> {
        self.clone_into_pub().bye().await
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
    #[error("no listener available for message that was just sent")]
    NoListenerAvailable,
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}
