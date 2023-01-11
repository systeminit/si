use std::collections::HashMap;
use ulid::Ulid;
use serde::{Serialize, Deserialize};

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
