use si_events::workspace_snapshot::EntityKind as EntityKindEvents;
use si_id::EntityId;
use thiserror::Error;

use crate::{
    DalContext,
    SchemaVariant,
    SchemaVariantError,
    WorkspaceSnapshotError,
    diagram::{
        DiagramError,
        view::View,
    },
    workspace_snapshot::EntityKindExt,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum EntityKindError {
    #[error("diagram error: {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("node not found for entity id {0}")]
    NodeNotFound(EntityId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}
pub type EntityKindResult<T> = Result<T, EntityKindError>;
pub struct EntityKind;

impl EntityKind {
    pub async fn get_entity_kind_for_id(
        ctx: &DalContext,
        id: EntityId,
    ) -> EntityKindResult<EntityKindEvents> {
        ctx.workspace_snapshot()?.get_entity_kind_for_id(id).await
    }

    pub async fn get_entity_name_for_id(
        ctx: &DalContext,
        id: EntityId,
    ) -> EntityKindResult<Option<String>> {
        let entity_kind = ctx.workspace_snapshot()?.get_entity_kind_for_id(id).await?;
        let name = match entity_kind {
            // As we add the ability to set requirements on different
            // entity kinds, update this
            EntityKindEvents::Action
            | EntityKindEvents::ActionPrototype
            | EntityKindEvents::ApprovalRequirementDefinition
            | EntityKindEvents::AttributePrototype
            | EntityKindEvents::AttributePrototypeArgument
            | EntityKindEvents::AttributeValue
            | EntityKindEvents::CategoryAction
            | EntityKindEvents::CategoryComponent
            | EntityKindEvents::CategoryDependentValueRoots
            | EntityKindEvents::CategoryDeprecatedActionBatch
            | EntityKindEvents::CategoryDiagramObject
            | EntityKindEvents::CategoryFunc
            | EntityKindEvents::CategoryModule
            | EntityKindEvents::CategorySchema
            | EntityKindEvents::CategorySecret
            | EntityKindEvents::CategoryView
            | EntityKindEvents::Component
            | EntityKindEvents::DependentValueRoot
            | EntityKindEvents::DiagramObject
            | EntityKindEvents::FinishedDependentValueRoot
            | EntityKindEvents::Func
            | EntityKindEvents::FuncArgument
            | EntityKindEvents::Geometry
            | EntityKindEvents::InputSocket
            | EntityKindEvents::JsonValue
            | EntityKindEvents::ManagementPrototype
            | EntityKindEvents::Module
            | EntityKindEvents::Ordering
            | EntityKindEvents::OutputSocket
            | EntityKindEvents::Prop
            | EntityKindEvents::Root
            | EntityKindEvents::Schema
            | EntityKindEvents::Secret
            | EntityKindEvents::StaticArgumentValue
            | EntityKindEvents::ValidationOutput
            | EntityKindEvents::SubGraphRoot
            | EntityKindEvents::ExternalTarget
            | EntityKindEvents::DeprecatedAction
            | EntityKindEvents::DeprecatedActionRunner
            | EntityKindEvents::DeprecatedActionBatch
            | EntityKindEvents::ValidationPrototype => None,
            EntityKindEvents::SchemaVariant => {
                let variant_name = SchemaVariant::get_by_id(ctx, id.into_inner().into())
                    .await?
                    .display_name()
                    .to_owned();
                Some(variant_name)
            }
            EntityKindEvents::View => {
                let view_name = View::get_by_id(ctx, id.into_inner().into())
                    .await?
                    .name()
                    .to_owned();
                Some(view_name)
            }
        };
        Ok(name)
    }
}

impl From<DiagramError> for EntityKindError {
    fn from(value: DiagramError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for EntityKindError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}
