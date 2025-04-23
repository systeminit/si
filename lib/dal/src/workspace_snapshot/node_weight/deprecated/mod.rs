pub use action_node_weight::DeprecatedActionNodeWeightLegacy;
pub use action_prototype_node_weight::DeprecatedActionPrototypeNodeWeightLegacy;
pub use attribute_prototype_argument_node_weight::DeprecatedAttributePrototypeArgumentNodeWeightLegacy;
pub use attribute_value_node_weight::DeprecatedAttributeValueNodeWeightLegacy;
pub use category_node_weight::DeprecatedCategoryNodeWeightLegacy;
pub use component_node_weight::DeprecatedComponentNodeWeightLegacy;
pub use content_node_weight::DeprecatedContentNodeWeightLegacy;
pub use dependent_value_root_node_weight::DeprecatedDependentValueRootNodeWeightLegacy;
pub use func_argument_node_weight::DeprecatedFuncArgumentNodeWeightLegacy;
pub use func_node_weight::DeprecatedFuncNodeWeightLegacy;
pub use ordering_node_weight::DeprecatedOrderingNodeWeightLegacy;
pub use prop_node_weight::DeprecatedPropNodeWeightLegacy;
pub use secret_node_weight::DeprecatedSecretNodeWeightLegacy;
use serde::{
    Deserialize,
    Serialize,
};

use crate::workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeprecatedNodeWeightLegacy {
    Action(DeprecatedActionNodeWeightLegacy),
    ActionPrototype(DeprecatedActionPrototypeNodeWeightLegacy),
    AttributePrototypeArgument(DeprecatedAttributePrototypeArgumentNodeWeightLegacy),
    AttributeValue(DeprecatedAttributeValueNodeWeightLegacy),
    Category(DeprecatedCategoryNodeWeightLegacy),
    Component(DeprecatedComponentNodeWeightLegacy),
    Content(DeprecatedContentNodeWeightLegacy),
    Func(DeprecatedFuncNodeWeightLegacy),
    FuncArgument(DeprecatedFuncArgumentNodeWeightLegacy),
    Ordering(DeprecatedOrderingNodeWeightLegacy),
    Prop(DeprecatedPropNodeWeightLegacy),
    Secret(DeprecatedSecretNodeWeightLegacy),
    DependentValueRoot(DeprecatedDependentValueRootNodeWeightLegacy),
}

impl DeprecatedNodeWeightLegacy {
    pub fn vector_clock_first_seen(&self) -> DeprecatedVectorClock {
        match self {
            DeprecatedNodeWeightLegacy::Action(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::ActionPrototype(weight) => {
                weight.vector_clock_first_seen.clone()
            }
            DeprecatedNodeWeightLegacy::AttributePrototypeArgument(weight) => {
                weight.vector_clock_first_seen.clone()
            }
            DeprecatedNodeWeightLegacy::AttributeValue(weight) => {
                weight.vector_clock_first_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Category(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::Component(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::Content(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::Func(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::FuncArgument(weight) => {
                weight.vector_clock_first_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Ordering(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::Prop(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::Secret(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeightLegacy::DependentValueRoot(weight) => {
                weight.vector_clock_first_seen.clone()
            }
        }
    }

    pub fn vector_clock_recently_seen(&self) -> DeprecatedVectorClock {
        match self {
            DeprecatedNodeWeightLegacy::Action(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeightLegacy::ActionPrototype(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::AttributePrototypeArgument(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::AttributeValue(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Category(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Component(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Content(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Func(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeightLegacy::FuncArgument(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Ordering(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeightLegacy::Prop(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeightLegacy::Secret(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeightLegacy::DependentValueRoot(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
        }
    }

    pub fn vector_clock_write(&self) -> DeprecatedVectorClock {
        match self {
            DeprecatedNodeWeightLegacy::Action(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::ActionPrototype(weight) => {
                weight.vector_clock_write.clone()
            }
            DeprecatedNodeWeightLegacy::AttributePrototypeArgument(weight) => {
                weight.vector_clock_write.clone()
            }
            DeprecatedNodeWeightLegacy::AttributeValue(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Category(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Component(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Content(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Func(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::FuncArgument(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Ordering(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Prop(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::Secret(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeightLegacy::DependentValueRoot(weight) => {
                weight.vector_clock_write.clone()
            }
        }
    }
}
