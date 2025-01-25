use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        graph::{
            traits::entity_kind::EntityKindExt, WorkspaceSnapshotGraphError,
            WorkspaceSnapshotGraphResult,
        },
        node_weight::{category_node_weight::CategoryNodeKind, NodeWeightError},
    },
    NodeWeightDiscriminants,
};

use super::WorkspaceSnapshotGraphV4;

impl EntityKindExt for WorkspaceSnapshotGraphV4 {
    fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotGraphResult<EntityKind> {
        let node_weight = self.get_node_weight_by_id(id)?;
        Ok(match node_weight.into() {
            NodeWeightDiscriminants::Action => EntityKind::Action,
            NodeWeightDiscriminants::ActionPrototype => EntityKind::ActionPrototype,
            NodeWeightDiscriminants::ApprovalRequirementDefinition => {
                EntityKind::ApprovalRequirementDefinition
            }
            NodeWeightDiscriminants::AttributePrototypeArgument => {
                EntityKind::AttributePrototypeArgument
            }
            NodeWeightDiscriminants::AttributeValue => EntityKind::AttributeValue,
            NodeWeightDiscriminants::Category => {
                match node_weight.get_category_node_weight()?.kind() {
                    CategoryNodeKind::Action => EntityKind::CategoryAction,
                    CategoryNodeKind::Component => EntityKind::CategoryComponent,
                    CategoryNodeKind::DeprecatedActionBatch => {
                        EntityKind::CategoryDeprecatedActionBatch
                    }
                    CategoryNodeKind::Func => EntityKind::CategoryFunc,
                    CategoryNodeKind::Module => EntityKind::CategoryModule,
                    CategoryNodeKind::Schema => EntityKind::CategorySchema,
                    CategoryNodeKind::Secret => EntityKind::CategorySecret,
                    CategoryNodeKind::DependentValueRoots => {
                        EntityKind::CategoryDependentValueRoots
                    }
                    CategoryNodeKind::View => EntityKind::CategoryView,
                    CategoryNodeKind::DiagramObject => EntityKind::CategoryDiagramObject,
                }
            }
            NodeWeightDiscriminants::Component => EntityKind::Component,
            NodeWeightDiscriminants::Content => match node_weight
                .get_content_node_weight()?
                .content_address_discriminants()
            {
                ContentAddressDiscriminants::ActionPrototype => EntityKind::ActionPrototype,
                ContentAddressDiscriminants::AttributePrototype => EntityKind::AttributePrototype,
                ContentAddressDiscriminants::Component => EntityKind::Component,
                ContentAddressDiscriminants::Func => EntityKind::Func,
                // NOTE(nick): we are treating "FuncArg" and "FuncArgument" as the same entity.
                ContentAddressDiscriminants::FuncArg => EntityKind::FuncArgument,
                ContentAddressDiscriminants::Geometry => EntityKind::Geometry,
                ContentAddressDiscriminants::InputSocket => EntityKind::InputSocket,
                ContentAddressDiscriminants::JsonValue => EntityKind::JsonValue,
                ContentAddressDiscriminants::ManagementPrototype => EntityKind::ManagementPrototype,
                ContentAddressDiscriminants::Module => EntityKind::Module,
                ContentAddressDiscriminants::OutputSocket => EntityKind::OutputSocket,
                ContentAddressDiscriminants::Prop => EntityKind::Prop,
                ContentAddressDiscriminants::Root => EntityKind::Root,
                ContentAddressDiscriminants::Schema => EntityKind::Schema,
                ContentAddressDiscriminants::SchemaVariant => EntityKind::SchemaVariant,
                ContentAddressDiscriminants::Secret => EntityKind::Secret,
                ContentAddressDiscriminants::StaticArgumentValue => EntityKind::StaticArgumentValue,
                ContentAddressDiscriminants::ValidationOutput => EntityKind::ValidationOutput,
                ContentAddressDiscriminants::ValidationPrototype => EntityKind::ValidationPrototype,
                ContentAddressDiscriminants::View => EntityKind::View,
                invalid => {
                    return Err(WorkspaceSnapshotGraphError::NodeWeight(
                        NodeWeightError::InvalidContentAddressForWeightKind(
                            invalid.to_string(),
                            "Content".to_string(),
                        ),
                    ))
                }
            },
            NodeWeightDiscriminants::DependentValueRoot => EntityKind::DependentValueRoot,
            NodeWeightDiscriminants::DiagramObject => EntityKind::DiagramObject,
            NodeWeightDiscriminants::FinishedDependentValueRoot => {
                EntityKind::FinishedDependentValueRoot
            }
            NodeWeightDiscriminants::Func => EntityKind::Func,
            NodeWeightDiscriminants::FuncArgument => EntityKind::FuncArgument,
            NodeWeightDiscriminants::Geometry => EntityKind::Geometry,
            NodeWeightDiscriminants::InputSocket => EntityKind::InputSocket,
            NodeWeightDiscriminants::ManagementPrototype => EntityKind::ManagementPrototype,
            NodeWeightDiscriminants::Ordering => EntityKind::Ordering,
            NodeWeightDiscriminants::Prop => EntityKind::Prop,
            NodeWeightDiscriminants::SchemaVariant => EntityKind::SchemaVariant,
            NodeWeightDiscriminants::Secret => EntityKind::Secret,
            NodeWeightDiscriminants::View => EntityKind::View,
        })
    }
}
