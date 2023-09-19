use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    change_set_pointer::ChangeSetPointer,
    content::hash::ContentHash,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{NodeWeightError, NodeWeightResult},
        vector_clock::VectorClock,
    },
    PropKind,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct PropNodeWeight {
    id: Ulid,
    lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: ContentHash,
    kind: PropKind,
    name: String,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl PropNodeWeight {
    pub fn new(
        change_set: &ChangeSetPointer,
        id: Ulid,
        content_address: ContentAddress,
        kind: PropKind,
        name: String,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            content_address,
            merkle_tree_hash: ContentHash::default(),
            kind,
            name,
            vector_clock_first_seen: VectorClock::new(change_set)?,
            vector_clock_recently_seen: VectorClock::new(change_set)?,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn increment_vector_clock(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write.inc(change_set)?;
        self.vector_clock_recently_seen.inc(change_set)?;

        Ok(())
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn mark_seen_at(&mut self, change_set: &ChangeSetPointer, seen_at: DateTime<Utc>) {
        self.vector_clock_recently_seen
            .inc_to(change_set, seen_at.clone());
        if self.vector_clock_first_seen.entry_for(change_set).is_none() {
            self.vector_clock_first_seen.inc_to(change_set, seen_at);
        }
    }

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSetPointer,
        other: &Self,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set, &other.vector_clock_write)?;
        self.vector_clock_first_seen
            .merge(change_set, &other.vector_clock_first_seen)?;
        self.vector_clock_recently_seen
            .merge(change_set, &other.vector_clock_recently_seen)?;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::AttributePrototype(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "AttributePrototype".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::AttributeValue(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "AttributeValue".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::Component(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Component".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::ExternalProvider(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "ExternalProvider".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::Func(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Func".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::FuncArg(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "FuncArc".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::InternalProvider(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "InternalProvider".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::Prop(_) => ContentAddress::Prop(content_hash),
            ContentAddress::Root => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Root".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::Schema(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Schema".to_string(),
                    "Prop".to_string(),
                ))
            }
            ContentAddress::SchemaVariant(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "SchemaVariant".to_string(),
                    "Prop".to_string(),
                ))
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clock(change_set)?;

        Ok(new_node_weight)
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "kind": self.kind,
            "name": self.name,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSetPointer,
        new_val: DateTime<Utc>,
    ) {
        self.vector_clock_recently_seen.inc_to(change_set, new_val);
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        &self.vector_clock_recently_seen
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }
}

impl std::fmt::Debug for PropNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PropNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .field("vector_clock_first_seen", &self.vector_clock_first_seen)
            .field(
                "vector_clock_recently_seen",
                &self.vector_clock_recently_seen,
            )
            .field("vector_clock_write", &self.vector_clock_write)
            .finish()
    }
}
