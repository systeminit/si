use crate::{LabelEntry, LabelList, WriteTenancy};
use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_has_many, DalContext, HistoryEventError, Node, NodeError, NodeKind,
    ReadTenancyError, Schema, SchemaError, SchemaId, SchemaVariant, SchemaVariantId, StandardModel,
    StandardModelError, Timestamp, Visibility, Workspace, WorkspaceId,
};

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspaceId),
}

pub type SystemResult<T> = Result<T, SystemError>;

pk!(SystemPk);
pk!(SystemId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct System {
    pk: SystemPk,
    id: SystemId,
    name: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: System,
    pk: SystemPk,
    id: SystemId,
    table_name: "systems",
    history_event_label_base: "system",
    history_event_message_name: "System"
}

impl System {
    pub async fn new(ctx: &DalContext<'_, '_>, name: impl AsRef<str>) -> SystemResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM system_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, SystemResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "system_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: SystemResult,
    );

    standard_model_belongs_to!(
        lookup_fn: schema_variant,
        set_fn: set_schema_variant,
        unset_fn: unset_schema_variant,
        table: "system_belongs_to_schema_variant",
        model_table: "schema_variants",
        belongs_to_id: SchemaVariantId,
        returns: SchemaVariant,
        result: SystemResult,
    );

    standard_model_belongs_to!(
        lookup_fn: workspace,
        set_fn: set_workspace,
        unset_fn: unset_workspace,
        table: "system_belongs_to_workspace",
        model_table: "workspaces",
        belongs_to_id: WorkspaceId,
        returns: Workspace,
        result: SystemResult,
    );

    standard_model_has_many!(
        lookup_fn: node,
        table: "node_belongs_to_system",
        model_table: "nodes",
        returns: Node,
        result: SystemResult,
    );

    #[instrument(skip_all)]
    pub async fn new_with_node(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        workspace_id: &WorkspaceId,
    ) -> SystemResult<(Self, Node)> {
        let name = name.as_ref();

        let schema = Schema::find_by_attr(ctx, "name", &"system".to_string())
            .await?
            .pop()
            .ok_or(SystemError::SchemaNotFound)?;
        let schema_variant = schema.default_variant(ctx).await?;

        let system = Self::new(ctx, name).await?;
        system.set_schema(ctx, schema.id()).await?;
        system.set_schema_variant(ctx, schema_variant.id()).await?;
        let node = Node::new(ctx, &NodeKind::System).await?;
        node.set_system(ctx, system.id()).await?;

        system.set_workspace(ctx, workspace_id).await?;

        Ok((system, node))
    }

    #[instrument(skip_all)]
    pub async fn list_for_workspace(
        ctx: &DalContext<'_, '_>,
        wid: &WorkspaceId,
    ) -> SystemResult<LabelList<SystemId>> {
        let system_labels: Vec<_> = Workspace::get_by_id(ctx, wid)
            .await?
            .ok_or(SystemError::WorkspaceNotFound(*wid))?
            .systems(ctx)
            .await?
            .into_iter()
            .map(|system| LabelEntry::new(system.name(), *system.id()))
            .collect();
        Ok(LabelList::new(system_labels))
    }
}
