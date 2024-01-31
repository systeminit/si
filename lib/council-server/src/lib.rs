#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn
)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumDiscriminants;
use ulid::Ulid;

pub mod client;
pub mod server;

// The subject generator module is for internal use. The module is private as a result.
mod subject_generator;

pub use client::management::ManagementClient;
pub use client::{Client, PubClient};
pub use server::Server;
pub(crate) use subject_generator::SubjectGenerator;

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct Id(Ulid);

impl Default for Id {
    fn default() -> Self {
        Self(Ulid::new())
    }
}

impl Id {
    pub const fn from_string(str: &str) -> Result<Self, ulid::DecodeError> {
        match Ulid::from_string(str) {
            Ok(ulid) => Ok(Self(ulid)),
            Err(err) => Err(err),
        }
    }
}

impl From<Ulid> for Id {
    fn from(ulid: Ulid) -> Self {
        Self(ulid)
    }
}

impl From<Id> for Ulid {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&self.0.to_string()).finish()
    }
}

pub type Graph = HashMap<Id, Vec<Id>>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, EnumDiscriminants)]
#[serde(tag = "kind")]
pub enum Request {
    Bye {
        change_set_id: Id,
    },
    ProcessedValue {
        change_set_id: Id,
        node_id: Id,
    },
    Restart,
    ValueDependencyGraph {
        change_set_id: Id,
        dependency_graph: Graph,
    },
    ValueProcessingFailed {
        change_set_id: Id,
        node_id: Id,
    },
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Response {
    BeenProcessed { node_id: Id },
    Failed { node_id: Id },
    OkToProcess { node_ids: Vec<Id> },
    Restart,
    Shutdown,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum ManagementResponse {
    Restart,
}
