//! This module is strictly for types used by the si-fs fuse API client. The
//! types are deliberately not re-exported in the root, so that they don't get
//! mixed up with non si-fs types.

use serde::{Deserialize, Serialize};
use si_events::{ChangeSetId, SchemaId, SchemaVariantId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeSet {
    pub name: String,
    pub id: ChangeSetId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateChangeSetRequest {
    pub name: String,
}

pub type CreateChangeSetResponse = ChangeSet;
pub type ListChangeSetsResponse = Vec<ChangeSet>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Schema {
    pub installed: bool,
    pub category: String,
    pub name: String,
    pub id: SchemaId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListVariantsResponse {
    pub locked: Option<SchemaVariantId>,
    pub unlocked: Option<SchemaVariantId>,
}
