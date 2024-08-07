use std::num::TryFromIntError;

use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, EncryptedSecretKey};
use strum::EnumDiscriminants;
use thiserror::Error;

use crate::EdgeWeightKindDiscriminants;
use crate::{
    action::prototype::ActionKind,
    workspace_snapshot::{content_address::ContentAddress, vector_clock::VectorClockError},
    ChangeSetId, PropKind,
};

use crate::func::FuncKind;
use crate::workspace_snapshot::graph::LineageId;
use crate::workspace_snapshot::node_weight::secret_node_weight::SecretNodeWeight;
pub use action_node_weight::ActionNodeWeight;
pub use action_prototype_node_weight::ActionPrototypeNodeWeight;
pub use attribute_prototype_argument_node_weight::ArgumentTargets;
pub use attribute_prototype_argument_node_weight::AttributePrototypeArgumentNodeWeight;
pub use attribute_value_node_weight::AttributeValueNodeWeight;
pub use category_node_weight::CategoryNodeWeight;
pub use component_node_weight::ComponentNodeWeight;
pub use content_node_weight::ContentNodeWeight;
pub use dependent_value_root_node_weight::DependentValueRootNodeWeight;
pub use func_argument_node_weight::FuncArgumentNodeWeight;
pub use func_node_weight::FuncNodeWeight;
pub use ordering_node_weight::OrderingNodeWeight;
pub use prop_node_weight::PropNodeWeight;

use super::content_address::ContentAddressDiscriminants;
use super::graph::deprecated::v1::DeprecatedNodeWeightV1;

pub mod action_node_weight;
pub mod action_prototype_node_weight;
pub mod attribute_prototype_argument_node_weight;
pub mod attribute_value_node_weight;
pub mod category_node_weight;
pub mod component_node_weight;
pub mod content_node_weight;
pub mod dependent_value_root_node_weight;
pub mod func_argument_node_weight;
pub mod func_node_weight;
pub mod ordering_node_weight;
pub mod prop_node_weight;
pub mod secret_node_weight;

pub mod deprecated;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum NodeWeightError {
    #[error("Cannot set content hash directly on node weight kind")]
    CannotSetContentHashOnKind,
    #[error("Cannot set content order directly on node weight kind")]
    CannotSetOrderOnKind,
    #[error("Cannot update root node's content hash")]
    CannotUpdateRootNodeContentHash,
    // #[error("ChangeSet error: {0}")]
    // ChangeSet(#[from] ChangeSetError),
    #[error("Incompatible node weights")]
    IncompatibleNodeWeightVariants,
    #[error("Invalid ContentAddress variant ({0}) for NodeWeight variant ({1})")]
    InvalidContentAddressForWeightKind(String, String),
    #[error("Missing Key for Child Entry {0}")]
    MissingKeytForChildEntry(Ulid),
    #[error("try from int error: {0}")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("Unexpected content address variant: {1} expected {0}")]
    UnexpectedContentAddressVariant(ContentAddressDiscriminants, ContentAddressDiscriminants),
    #[error("Unexpected node weight variant. Got {1} but expected {0}")]
    UnexpectedNodeWeightVariant(NodeWeightDiscriminants, NodeWeightDiscriminants),
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type NodeWeightResult<T> = Result<T, NodeWeightError>;

/// **WARNING**: the order of this enum is important! Do not re-order elements.
/// New variants must go at the end, even if it's not in lexical order!
#[derive(Debug, Serialize, Deserialize, Clone, EnumDiscriminants, PartialEq, Eq)]
#[strum_discriminants(derive(strum::Display, Hash, Serialize, Deserialize))]
pub enum NodeWeight {
    Action(ActionNodeWeight),
    ActionPrototype(ActionPrototypeNodeWeight),
    AttributePrototypeArgument(AttributePrototypeArgumentNodeWeight),
    AttributeValue(AttributeValueNodeWeight),
    Category(CategoryNodeWeight),
    Component(ComponentNodeWeight),
    Content(ContentNodeWeight),
    DependentValueRoot(DependentValueRootNodeWeight),
    Func(FuncNodeWeight),
    FuncArgument(FuncArgumentNodeWeight),
    Ordering(OrderingNodeWeight),
    Prop(PropNodeWeight),
    Secret(SecretNodeWeight),
}

impl NodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Action(weight) => weight.content_hash(),
            NodeWeight::ActionPrototype(weight) => weight.content_hash(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.content_hash(),
            NodeWeight::AttributeValue(weight) => weight.content_hash(),
            NodeWeight::Category(weight) => weight.content_hash(),
            NodeWeight::Component(weight) => weight.content_hash(),
            NodeWeight::Content(weight) => weight.content_hash(),
            NodeWeight::Func(weight) => weight.content_hash(),
            NodeWeight::FuncArgument(weight) => weight.content_hash(),
            NodeWeight::Ordering(weight) => weight.content_hash(),
            NodeWeight::Prop(weight) => weight.content_hash(),
            NodeWeight::Secret(weight) => weight.content_hash(),
            NodeWeight::DependentValueRoot(weight) => weight.content_hash(),
        }
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        match self {
            NodeWeight::Action(weight) => weight.content_store_hashes(),
            NodeWeight::ActionPrototype(weight) => weight.content_store_hashes(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.content_store_hashes(),
            NodeWeight::AttributeValue(weight) => weight.content_store_hashes(),
            NodeWeight::Category(weight) => weight.content_store_hashes(),
            NodeWeight::Component(weight) => weight.content_store_hashes(),
            NodeWeight::Content(weight) => weight.content_store_hashes(),
            NodeWeight::Func(weight) => weight.content_store_hashes(),
            NodeWeight::FuncArgument(weight) => weight.content_store_hashes(),
            NodeWeight::Ordering(weight) => weight.content_store_hashes(),
            NodeWeight::Prop(weight) => weight.content_store_hashes(),
            NodeWeight::Secret(weight) => weight.content_store_hashes(),
            NodeWeight::DependentValueRoot(weight) => weight.content_store_hashes(),
        }
    }

    pub fn content_address_discriminants(&self) -> Option<ContentAddressDiscriminants> {
        match self {
            NodeWeight::Content(weight) => Some(weight.content_address().into()),
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Component(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Ordering(_)
            | NodeWeight::Prop(_)
            | NodeWeight::Secret(_)
            | NodeWeight::DependentValueRoot(_) => None,
        }
    }

    pub fn id(&self) -> Ulid {
        match self {
            NodeWeight::Action(weight) => weight.id(),
            NodeWeight::ActionPrototype(weight) => weight.id(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.id(),
            NodeWeight::AttributeValue(weight) => weight.id(),
            NodeWeight::Category(weight) => weight.id(),
            NodeWeight::Component(weight) => weight.id(),
            NodeWeight::Content(weight) => weight.id(),
            NodeWeight::Func(weight) => weight.id(),
            NodeWeight::FuncArgument(weight) => weight.id(),
            NodeWeight::Ordering(weight) => weight.id(),
            NodeWeight::Prop(weight) => weight.id(),
            NodeWeight::Secret(weight) => weight.id(),
            NodeWeight::DependentValueRoot(weight) => weight.id(),
        }
    }

    pub fn lineage_id(&self) -> Ulid {
        match self {
            NodeWeight::Action(weight) => weight.lineage_id(),
            NodeWeight::ActionPrototype(weight) => weight.lineage_id(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.lineage_id(),
            NodeWeight::AttributeValue(weight) => weight.lineage_id(),
            NodeWeight::Category(weight) => weight.lineage_id(),
            NodeWeight::Component(weight) => weight.lineage_id(),
            NodeWeight::Content(weight) => weight.lineage_id(),
            NodeWeight::Func(weight) => weight.lineage_id(),
            NodeWeight::FuncArgument(weight) => weight.lineage_id(),
            NodeWeight::Ordering(weight) => weight.lineage_id(),
            NodeWeight::Prop(weight) => weight.lineage_id(),
            NodeWeight::Secret(weight) => weight.lineage_id(),
            NodeWeight::DependentValueRoot(weight) => weight.lineage_id(),
        }
    }

    pub fn set_id_and_lineage(&mut self, id: impl Into<Ulid>, lineage_id: LineageId) {
        match self {
            NodeWeight::Action(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::ActionPrototype(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::AttributeValue(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Category(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Component(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Content(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Func(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::FuncArgument(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Ordering(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Prop(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::Secret(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
            NodeWeight::DependentValueRoot(weight) => {
                weight.id = id.into();
                weight.lineage_id = lineage_id;
            }
        }
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        match self {
            NodeWeight::Action(weight) => weight.merkle_tree_hash(),
            NodeWeight::ActionPrototype(weight) => weight.merkle_tree_hash(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.merkle_tree_hash(),
            NodeWeight::AttributeValue(weight) => weight.merkle_tree_hash(),
            NodeWeight::Category(weight) => weight.merkle_tree_hash(),
            NodeWeight::Component(weight) => weight.merkle_tree_hash(),
            NodeWeight::Content(weight) => weight.merkle_tree_hash(),
            NodeWeight::Func(weight) => weight.merkle_tree_hash(),
            NodeWeight::FuncArgument(weight) => weight.merkle_tree_hash(),
            NodeWeight::Ordering(weight) => weight.merkle_tree_hash(),
            NodeWeight::Prop(weight) => weight.merkle_tree_hash(),
            NodeWeight::Secret(weight) => weight.merkle_tree_hash(),
            NodeWeight::DependentValueRoot(weight) => weight.merkle_tree_hash(),
        }
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Component(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Content(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Func(weight) => weight.new_content_hash(content_hash),
            NodeWeight::FuncArgument(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Prop(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Secret(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::DependentValueRoot(_)
            | NodeWeight::Ordering(_) => Err(NodeWeightError::CannotSetContentHashOnKind),
        }
    }

    /// The node hash is used to compare nodes directly, and should be computed based on the data
    /// that is specific to the node weight, *and* the content_hash, so that changes are detected
    /// between nodes whether the content has changed or just the node weight has changed.
    pub fn node_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Action(weight) => weight.node_hash(),
            NodeWeight::ActionPrototype(weight) => weight.node_hash(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.node_hash(),
            NodeWeight::AttributeValue(weight) => weight.node_hash(),
            NodeWeight::Category(weight) => weight.node_hash(),
            NodeWeight::Component(weight) => weight.node_hash(),
            NodeWeight::Content(weight) => weight.node_hash(),
            NodeWeight::Func(weight) => weight.node_hash(),
            NodeWeight::FuncArgument(weight) => weight.node_hash(),
            NodeWeight::Ordering(weight) => weight.node_hash(),
            NodeWeight::Prop(weight) => weight.node_hash(),
            NodeWeight::Secret(weight) => weight.node_hash(),
            NodeWeight::DependentValueRoot(weight) => weight.node_hash(),
        }
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        match self {
            NodeWeight::Action(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::ActionPrototype(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::AttributePrototypeArgument(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::AttributeValue(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Category(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Component(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Content(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Func(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::FuncArgument(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Ordering(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Prop(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Secret(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::DependentValueRoot(weight) => weight.set_merkle_tree_hash(new_hash),
        }
    }

    pub fn set_order(&mut self, order: Vec<Ulid>) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Ordering(ordering_weight) => {
                ordering_weight.set_order(order);
                Ok(())
            }
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Component(_)
            | NodeWeight::Content(_)
            | NodeWeight::DependentValueRoot(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Prop(_)
            | NodeWeight::Secret(_) => Err(NodeWeightError::CannotSetOrderOnKind),
        }
    }

    /// Many node kinds need to have complete control of their outgoing edges
    /// relative to another changeset in order to have a correctly constructed
    /// graph. For example, only one set of children of a given attribute value
    /// should "win" in a rebase operation, otherwise there could be duplicate
    /// child values for an attribute value. This method will be called during
    /// conflict detection in order to produce a conflict if the change set
    /// being rebased has unseen edges of this type for a given "container"
    /// node. If edge kinds are not returned here, those unseen edges will be
    /// silently merged with the `onto` changeset's edges. This a "business"
    /// logic problem, rather than a purely graph-theoretical one.
    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        match self {
            NodeWeight::Action(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::ActionPrototype(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::AttributeValue(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Category(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Component(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Content(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Func(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::FuncArgument(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Ordering(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Prop(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Secret(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::DependentValueRoot(weight) => weight.exclusive_outgoing_edges(),
        }
    }

    pub fn is_exclusive_outgoing_edge(&self, edge_kind: EdgeWeightKindDiscriminants) -> bool {
        self.exclusive_outgoing_edges().contains(&edge_kind)
    }

    pub fn get_action_node_weight(&self) -> NodeWeightResult<ActionNodeWeight> {
        match self {
            NodeWeight::Action(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Action,
                other.into(),
            )),
        }
    }

    pub fn get_action_prototype_node_weight(&self) -> NodeWeightResult<ActionPrototypeNodeWeight> {
        match self {
            NodeWeight::ActionPrototype(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::ActionPrototype,
                other.into(),
            )),
        }
    }

    pub fn get_attribute_prototype_argument_node_weight(
        &self,
    ) -> NodeWeightResult<AttributePrototypeArgumentNodeWeight> {
        match self {
            NodeWeight::AttributePrototypeArgument(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::AttributePrototypeArgument,
                other.into(),
            )),
        }
    }

    pub fn get_attribute_value_node_weight(&self) -> NodeWeightResult<AttributeValueNodeWeight> {
        match self {
            NodeWeight::AttributeValue(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::AttributeValue,
                other.into(),
            )),
        }
    }

    pub fn get_dependent_value_root_node_weight(
        &self,
    ) -> NodeWeightResult<DependentValueRootNodeWeight> {
        match self {
            NodeWeight::DependentValueRoot(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::DependentValueRoot,
                other.into(),
            )),
        }
    }

    pub fn get_component_node_weight(&self) -> NodeWeightResult<ComponentNodeWeight> {
        match self {
            NodeWeight::Component(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Component,
                other.into(),
            )),
        }
    }

    pub fn get_prop_node_weight(&self) -> NodeWeightResult<PropNodeWeight> {
        match self {
            NodeWeight::Prop(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Prop,
                other.into(),
            )),
        }
    }

    pub fn get_func_node_weight(&self) -> NodeWeightResult<FuncNodeWeight> {
        match self {
            NodeWeight::Func(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Func,
                other.into(),
            )),
        }
    }

    pub fn get_func_argument_node_weight(&self) -> NodeWeightResult<FuncArgumentNodeWeight> {
        match self {
            NodeWeight::FuncArgument(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::FuncArgument,
                other.into(),
            )),
        }
    }

    pub fn get_secret_node_weight(&self) -> NodeWeightResult<SecretNodeWeight> {
        match self {
            NodeWeight::Secret(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Secret,
                other.into(),
            )),
        }
    }

    pub fn get_ordering_node_weight(&self) -> NodeWeightResult<OrderingNodeWeight> {
        match self {
            NodeWeight::Ordering(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Ordering,
                other.into(),
            )),
        }
    }

    pub fn get_content_node_weight_of_kind(
        &self,
        content_addr_discrim: ContentAddressDiscriminants,
    ) -> NodeWeightResult<ContentNodeWeight> {
        match self {
            NodeWeight::Content(inner) => {
                let inner_addr_discrim: ContentAddressDiscriminants =
                    inner.content_address().into();
                if inner_addr_discrim != content_addr_discrim {
                    return Err(NodeWeightError::UnexpectedContentAddressVariant(
                        content_addr_discrim,
                        inner_addr_discrim,
                    ));
                }

                Ok(inner.to_owned())
            }
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Content,
                other.into(),
            )),
        }
    }

    pub fn get_option_content_node_weight_of_kind(
        &self,
        content_addr_discrim: ContentAddressDiscriminants,
    ) -> Option<ContentNodeWeight> {
        match self {
            NodeWeight::Content(inner) => {
                let inner_addr_discrim: ContentAddressDiscriminants =
                    inner.content_address().into();
                if inner_addr_discrim != content_addr_discrim {
                    return None;
                }
                Some(inner.to_owned())
            }
            _other => None,
        }
    }

    pub fn new_content(id: Ulid, lineage_id: Ulid, kind: ContentAddress) -> Self {
        NodeWeight::Content(ContentNodeWeight::new(id, lineage_id, kind))
    }

    pub fn new_action(
        originating_change_set_id: ChangeSetId,
        action_id: Ulid,
        lineage_id: Ulid,
    ) -> Self {
        NodeWeight::Action(ActionNodeWeight::new(
            originating_change_set_id,
            action_id,
            lineage_id,
        ))
    }

    pub fn new_action_prototype(
        action_prototype_id: Ulid,
        lineage_id: Ulid,
        kind: ActionKind,
        name: String,
        description: Option<String>,
    ) -> Self {
        NodeWeight::ActionPrototype(ActionPrototypeNodeWeight::new(
            action_prototype_id,
            lineage_id,
            kind,
            name,
            description,
        ))
    }

    pub fn new_attribute_value(
        attribute_value_id: Ulid,
        lineage_id: Ulid,
        unprocessed_value: Option<ContentAddress>,
        value: Option<ContentAddress>,
    ) -> Self {
        NodeWeight::AttributeValue(AttributeValueNodeWeight::new(
            attribute_value_id,
            lineage_id,
            unprocessed_value,
            value,
        ))
    }

    pub fn new_dependent_value_root(id: Ulid, lineage_id: Ulid, value_id: Ulid) -> Self {
        NodeWeight::DependentValueRoot(DependentValueRootNodeWeight::new(id, lineage_id, value_id))
    }

    pub fn new_component(component_id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        NodeWeight::Component(ComponentNodeWeight::new(
            component_id,
            lineage_id,
            ContentAddress::Component(content_hash),
        ))
    }

    pub fn new_prop(
        prop_id: Ulid,
        lineage_id: Ulid,
        prop_kind: PropKind,
        name: impl AsRef<str>,
        content_hash: ContentHash,
    ) -> Self {
        NodeWeight::Prop(PropNodeWeight::new(
            prop_id,
            lineage_id,
            ContentAddress::Prop(content_hash),
            prop_kind,
            name.as_ref().to_string(),
        ))
    }

    pub fn new_func(
        func_id: Ulid,
        lineage_id: Ulid,
        name: impl AsRef<str>,
        func_kind: FuncKind,
        content_hash: ContentHash,
    ) -> Self {
        NodeWeight::Func(FuncNodeWeight::new(
            func_id,
            lineage_id,
            ContentAddress::Func(content_hash),
            name.as_ref().to_string(),
            func_kind,
        ))
    }

    pub fn new_func_argument(
        func_arg_id: Ulid,
        lineage_id: Ulid,
        name: impl AsRef<str>,
        content_hash: ContentHash,
    ) -> Self {
        NodeWeight::FuncArgument(FuncArgumentNodeWeight::new(
            func_arg_id,
            lineage_id,
            ContentAddress::FuncArg(content_hash),
            name.as_ref().to_string(),
        ))
    }

    pub fn new_attribute_prototype_argument(
        attribute_prototype_argument_id: Ulid,
        lineage_id: Ulid,
        targets: Option<ArgumentTargets>,
    ) -> Self {
        NodeWeight::AttributePrototypeArgument(AttributePrototypeArgumentNodeWeight::new(
            attribute_prototype_argument_id,
            lineage_id,
            targets,
        ))
    }

    pub fn new_secret(
        secret_id: Ulid,
        lineage_id: Ulid,
        encrypted_secret_key: EncryptedSecretKey,
        content_hash: ContentHash,
    ) -> Self {
        NodeWeight::Secret(SecretNodeWeight::new(
            secret_id,
            lineage_id,
            ContentAddress::Secret(content_hash),
            encrypted_secret_key,
        ))
    }
}

impl From<DeprecatedNodeWeightV1> for NodeWeight {
    fn from(value: DeprecatedNodeWeightV1) -> Self {
        match value {
            DeprecatedNodeWeightV1::Action(weight) => Self::Action(weight.into()),
            DeprecatedNodeWeightV1::ActionPrototype(weight) => Self::ActionPrototype(weight.into()),
            DeprecatedNodeWeightV1::AttributePrototypeArgument(weight) => {
                Self::AttributePrototypeArgument(weight.into())
            }
            DeprecatedNodeWeightV1::AttributeValue(weight) => Self::AttributeValue(weight.into()),
            DeprecatedNodeWeightV1::Category(weight) => Self::Category(weight.into()),
            DeprecatedNodeWeightV1::Component(weight) => Self::Component(weight.into()),
            DeprecatedNodeWeightV1::Content(weight) => Self::Content(weight.into()),
            DeprecatedNodeWeightV1::Func(weight) => Self::Func(weight.into()),
            DeprecatedNodeWeightV1::FuncArgument(weight) => Self::FuncArgument(weight.into()),
            DeprecatedNodeWeightV1::Ordering(weight) => Self::Ordering(weight.into()),
            DeprecatedNodeWeightV1::Prop(weight) => Self::Prop(weight.into()),
            DeprecatedNodeWeightV1::Secret(weight) => Self::Secret(weight.into()),
            DeprecatedNodeWeightV1::DependentValueRoot(weight) => {
                Self::DependentValueRoot(weight.into())
            }
        }
    }
}
