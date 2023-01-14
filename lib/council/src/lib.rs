use futures::StreamExt;
use serde::{Deserialize, Serialize};
use si_data_nats::{NatsClient, Subscription};
use std::collections::HashMap;
use ulid::Ulid;

pub type Id = Ulid;
pub type Graph = HashMap<Id, Vec<Id>>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Request {
    CreateValues,
    ValueCreationDone,
    ValueDependencyGraph {
        change_set_id: Id,
        dependency_graph: Graph,
    },
    ProcessedValue {
        change_set_id: Id,
        node_id: Id,
    },
    Bye {
        change_set_id: Id,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Response {
    OkToCreate,
    OkToProcess { node_ids: Vec<Id> },
    BeenProcessed { node_id: Id },
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
            .publish_with_reply_or_headers(
                &self.pub_channel,
                Some(&self.reply_channel),
                None,
                message,
            )
            .await?;
        Ok(())
    }

    pub async fn register_dependency_graph(&self, dependency_graph: Graph) -> Result<()> {
        let message = serde_json::to_vec(&Request::ValueDependencyGraph {
            change_set_id: self.change_set_id,
            dependency_graph,
        })?;
        self.nats
            .publish_with_reply_or_headers(
                &self.pub_channel,
                Some(&self.reply_channel),
                None,
                message,
            )
            .await?;
        Ok(())
    }

    pub async fn processed_value(&self, node_id: Id) -> Result<()> {
        let message = serde_json::to_vec(&Request::ProcessedValue {
            change_set_id: self.change_set_id,
            node_id,
        })?;
        self.nats
            .publish_with_reply_or_headers(
                &self.pub_channel,
                Some(&self.reply_channel),
                None,
                message,
            )
            .await?;
        Ok(())
    }

    pub async fn bye(self) -> Result<()> {
        let message = serde_json::to_vec(&Request::Bye {
            change_set_id: self.change_set_id,
        })?;
        self.nats
            .publish_with_reply_or_headers(
                &self.pub_channel,
                Some(&self.reply_channel),
                None,
                message,
            )
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
    pub async fn new(nats: NatsClient, id: Id, change_set_id: Id) -> Result<Self> {
        let pub_channel = format!("council.{id}");
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
        match self.subscription.next().await {
            Some(msg) => Ok(Some(serde_json::from_slice::<Response>(msg?.data())?)),
            None => Ok(None),
        }
    }

    pub async fn wait_to_create_values(&mut self) -> Result<State> {
        let message = serde_json::to_vec(&Request::CreateValues)?;
        self.nats
            .publish_with_reply_or_headers(
                &self.pub_channel,
                Some(&self.reply_channel),
                None,
                message,
            )
            .await?;

        // TODO: timeout so we don't get stuck here forever if council goes away
        loop {
            if let Some(message) = self.subscription.next().await {
                match serde_json::from_slice(message?.data())? {
                    Response::OkToCreate => break,
                    Response::Shutdown => return Ok(State::Shutdown),
                    msg => unreachable!("{msg:?}"),
                }
            }
        }

        Ok(State::Continue)
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

    pub async fn bye(self) -> Result<()> {
        self.clone_into_pub().bye().await
    }
}

#[derive(Debug)]
pub enum State {
    Continue,
    Shutdown,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}
