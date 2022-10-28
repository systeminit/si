//! This module contains [`Resource`], which there can only exist _one_ or _zero_ of for a given
//! [`Component`](crate::Component) and [`System`](crate::System).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, Value};
use si_data_pg::PgError;
use strum_macros::Display;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor,
    ws_event::{WsEvent, WsPayload},
    ActionPrototype, ActionPrototypeError, Component, ComponentError, ComponentId, DalContext,
    HistoryEventError, StandardModel, StandardModelError, SystemId, Timestamp, Visibility,
    WorkflowPrototype, WorkflowPrototypeError, WorkflowPrototypeId, WorkflowRunner,
    WorkflowRunnerError, WriteTenancy, WsEventError,
};

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    WorkflowPrototype(#[from] WorkflowPrototypeError),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),
    #[error(transparent)]
    Component(#[from] Box<ComponentError>),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),

    #[error("found unset component id (must use a \"set\" component id)")]
    FoundUnsetComponentId,
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("no schema for component {0}")]
    NoSchema(ComponentId),
    #[error("no schema variant for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("workflow prototype not found {0}")]
    WorkflowPrototypeNotFound(WorkflowPrototypeId),
}

pub type ResourceResult<T> = Result<T, ResourceError>;

const GET_BY_COMPONENT_AND_SYSTEM: &str =
    include_str!("queries/resource_get_by_component_and_system.sql");

pk!(ResourcePk);
pk!(ResourceId);

impl From<Resource> for veritech_client::ResourceView {
    fn from(res: Resource) -> Self {
        Self { data: res.data }
    }
}

/// A Resource is the "real-world" representation of a specific [`Component`],
/// as it exists in the world, where the [`Component`] is the representation of
/// what we think it should look like.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pk: ResourcePk,
    id: ResourceId,
    data: Value,
    component_id: ComponentId,
    system_id: SystemId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Resource,
    pk: ResourcePk,
    id: ResourceId,
    table_name: "resources",
    history_event_label_base: "resource",
    history_event_message_name: "Resource"
}

impl Resource {
    /// For a [`Resource`] to be uniquely identified, we need to know both
    /// which [`Component`] it is the "real world" representation of, and also
    /// the [`System`](crate::System) in which that component being referred to.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        data: Value,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ResourceResult<Self> {
        if component_id == ComponentId::NONE {
            return Err(ResourceError::FoundUnsetComponentId);
        }

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM resource_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &data,
                    &component_id,
                    &system_id,
                ],
            )
            .await?;

        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ResourceError::ComponentNotFound(component_id))?;
        component
            .set_value_by_json_pointer(ctx, "/root/resource", Some(serde_json::to_string(&data)?))
            .await
            .map_err(Box::new)?;

        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(data, Json<JsonValue>, ResourceResult);

    pub async fn upsert(
        ctx: &DalContext,
        data: Value,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ResourceResult<(Self, bool)> {
        let resource = Self::get_by_component_and_system(ctx, component_id, system_id).await?;
        if let Some(mut resource) = resource {
            let component = Component::get_by_id(ctx, &component_id)
                .await?
                .ok_or(ResourceError::ComponentNotFound(component_id))?;
            component
                .set_value_by_json_pointer(
                    ctx,
                    "/root/resource",
                    Some(serde_json::to_string(&data)?),
                )
                .await
                .map_err(Box::new)?;

            if resource.data != data {
                resource.set_data(ctx, data.clone()).await?;
                Ok((resource, true))
            } else {
                Ok((resource, false))
            }
        } else {
            Ok((Self::new(ctx, data, component_id, system_id).await?, true))
        }
    }

    pub async fn refresh(
        ctx: &DalContext,
        component: &Component,
        system_id: SystemId,
    ) -> ResourceResult<()> {
        let schema_variant = component
            .schema_variant(ctx)
            .await
            .map_err(Box::new)?
            .ok_or_else(|| ResourceError::NoSchemaVariant(*component.id()))?;
        let schema = component
            .schema(ctx)
            .await
            .map_err(Box::new)?
            .ok_or_else(|| ResourceError::NoSchema(*component.id()))?;
        let action = match ActionPrototype::find_by_name(
            ctx,
            "refresh",
            *schema.id(),
            *schema_variant.id(),
            SystemId::NONE,
        )
        .await?
        {
            Some(action) => action,
            None => return Ok(()),
        };

        let prototype = WorkflowPrototype::get_by_id(ctx, &action.workflow_prototype_id())
            .await?
            .ok_or_else(|| {
                ResourceError::WorkflowPrototypeNotFound(action.workflow_prototype_id())
            })?;
        let run_id: usize = rand::random();
        let (_runner, _state, _func_binding_return_values, _created_resources, _updated_resources) =
            WorkflowRunner::run(ctx, run_id, *prototype.id(), *component.id()).await?;
        WsEvent::resource_refreshed(ctx, *component.id(), system_id)
            .publish(ctx)
            .await?;
        Ok(())
    }

    pub async fn get_by_component_and_system(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ResourceResult<Option<Self>> {
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(
                GET_BY_COMPONENT_AND_SYSTEM,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                ],
            )
            .await?;
        let object = standard_model::option_object_from_row(maybe_row)?;
        Ok(object)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResourceHealth {
    Ok,
    Warning,
    Error,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub id: ResourceId,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub error: Option<String>,
    pub data: Value,
    pub health: ResourceHealth,
    pub entity_type: String,
}

impl ResourceView {
    pub fn new(resource: Resource) -> Self {
        // TODO: actually fill all of the data dynamically, most fields don't make much sense for now

        Self {
            id: *resource.id(),
            created_at: resource.timestamp().created_at,
            updated_at: resource.timestamp().updated_at,
            error: None,
            data: resource.data,
            health: ResourceHealth::Ok,
            entity_type: "idk bro".to_owned(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRefreshId {
    component_id: ComponentId,
    system_id: SystemId,
}

impl WsEvent {
    pub fn resource_refreshed(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::ResourceRefreshed(ResourceRefreshId {
                component_id,
                system_id,
            }),
        )
    }
}
