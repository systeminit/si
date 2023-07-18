//! Edges

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum EdgeWeightKind {
    #[default]
    Uses,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct EdgeWeight {
    pub kind: EdgeWeightKind,
}

impl EdgeWeight {
    pub fn new(kind: EdgeWeightKind) -> Self {
        Self { kind }
    }
}
