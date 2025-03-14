use si_id::EntityId;

use si_events::workspace_snapshot::EntityKind as EntityKindEvents;

use crate::{
    diagram::{view::View, DiagramError},
    workspace_snapshot::EntityKindExt,
    DalContext, SchemaVariant, SchemaVariantError, WorkspaceSnapshotError,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum EntityKindError {
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
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
        ctx.workspace_snapshot()?
            .get_entity_kind_for_id(id)
            .await
            .map_err(Into::into)
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
