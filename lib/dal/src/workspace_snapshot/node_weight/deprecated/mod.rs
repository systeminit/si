use serde::{Deserialize, Serialize};

pub use action_node_weight::DeprecatedActionNodeWeight;
pub use action_prototype_node_weight::DeprecatedActionPrototypeNodeWeight;
pub use attribute_prototype_argument_node_weight::DeprecatedAttributePrototypeArgumentNodeWeight;
pub use attribute_value_node_weight::DeprecatedAttributeValueNodeWeight;
pub use category_node_weight::DeprecatedCategoryNodeWeight;
pub use component_node_weight::DeprecatedComponentNodeWeight;
pub use content_node_weight::DeprecatedContentNodeWeight;
pub use dependent_value_root_node_weight::DeprecatedDependentValueRootNodeWeight;
pub use func_argument_node_weight::DeprecatedFuncArgumentNodeWeight;
pub use func_node_weight::DeprecatedFuncNodeWeight;
pub use ordering_node_weight::DeprecatedOrderingNodeWeight;
pub use prop_node_weight::DeprecatedPropNodeWeight;
pub use secret_node_weight::DeprecatedSecretNodeWeight;

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
pub enum DeprecatedNodeWeight {
    Action(DeprecatedActionNodeWeight),
    ActionPrototype(DeprecatedActionPrototypeNodeWeight),
    AttributePrototypeArgument(DeprecatedAttributePrototypeArgumentNodeWeight),
    AttributeValue(DeprecatedAttributeValueNodeWeight),
    Category(DeprecatedCategoryNodeWeight),
    Component(DeprecatedComponentNodeWeight),
    Content(DeprecatedContentNodeWeight),
    Func(DeprecatedFuncNodeWeight),
    FuncArgument(DeprecatedFuncArgumentNodeWeight),
    Ordering(DeprecatedOrderingNodeWeight),
    Prop(DeprecatedPropNodeWeight),
    Secret(DeprecatedSecretNodeWeight),
    DependentValueRoot(DeprecatedDependentValueRootNodeWeight),
}

impl DeprecatedNodeWeight {
    pub fn vector_clock_first_seen(&self) -> DeprecatedVectorClock {
        match self {
            DeprecatedNodeWeight::Action(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::ActionPrototype(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::AttributePrototypeArgument(weight) => {
                weight.vector_clock_first_seen.clone()
            }
            DeprecatedNodeWeight::AttributeValue(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Category(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Component(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Content(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Func(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::FuncArgument(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Ordering(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Prop(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::Secret(weight) => weight.vector_clock_first_seen.clone(),
            DeprecatedNodeWeight::DependentValueRoot(weight) => {
                weight.vector_clock_first_seen.clone()
            }
        }
    }

    pub fn vector_clock_recently_seen(&self) -> DeprecatedVectorClock {
        match self {
            DeprecatedNodeWeight::Action(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::ActionPrototype(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeight::AttributePrototypeArgument(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeight::AttributeValue(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
            DeprecatedNodeWeight::Category(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::Component(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::Content(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::Func(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::FuncArgument(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::Ordering(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::Prop(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::Secret(weight) => weight.vector_clock_recently_seen.clone(),
            DeprecatedNodeWeight::DependentValueRoot(weight) => {
                weight.vector_clock_recently_seen.clone()
            }
        }
    }

    pub fn vector_clock_write(&self) -> DeprecatedVectorClock {
        match self {
            DeprecatedNodeWeight::Action(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::ActionPrototype(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::AttributePrototypeArgument(weight) => {
                weight.vector_clock_write.clone()
            }
            DeprecatedNodeWeight::AttributeValue(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Category(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Component(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Content(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Func(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::FuncArgument(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Ordering(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Prop(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::Secret(weight) => weight.vector_clock_write.clone(),
            DeprecatedNodeWeight::DependentValueRoot(weight) => weight.vector_clock_write.clone(),
        }
    }
}
