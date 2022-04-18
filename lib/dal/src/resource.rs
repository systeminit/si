use crate::DalContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use strum_macros::Display;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_belongs_to,
    ws_event::{WsEvent, WsPayload},
    BillingAccountId, Component, ComponentId, HistoryActor, HistoryEventError, ReadTenancyError,
    StandardModel, StandardModelError, System, SystemId, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("system id is required: -1 was used")]
    SystemIdRequired,
}

pub type ResourceResult<T> = Result<T, ResourceError>;

const GET_BY_COMPONENT_AND_SYSTEM: &str =
    include_str!("./queries/resource_get_by_component_and_system.sql");

pk!(ResourcePk);
pk!(ResourceId);

/// A Resource is the "real-world" representation of a specific [`Component`],
/// as it exists in the world, where the [`Component`] is the representation of
/// what we think it should look like.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pk: ResourcePk,
    id: ResourceId,
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
    /// the [`System`] in which that component being referred to.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM resource_create_v1($1, $2)",
                &[ctx.write_tenancy(), ctx.visibility()],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;

        object.set_component(ctx, component_id).await?;
        object.set_system(ctx, system_id).await?;

        Ok(object)
    }

    standard_model_belongs_to!(
        lookup_fn: component,
        set_fn: set_component,
        unset_fn: unset_component,
        table: "resource_belongs_to_component",
        model_table: "components",
        belongs_to_id: ComponentId,
        returns: Component,
        result: ResourceResult,
    );

    standard_model_belongs_to!(
        lookup_fn: system,
        set_fn: set_system,
        unset_fn: unset_system,
        table: "resource_belongs_to_system",
        model_table: "systems",
        belongs_to_id: SystemId,
        returns: System,
        result: ResourceResult,
    );

    #[allow(clippy::too_many_arguments)]
    pub async fn get_by_component_id_and_system_id(
        ctx: &DalContext<'_, '_>,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Option<Self>> {
        if system_id.is_none() {
            return Err(ResourceError::SystemIdRequired);
        }

        let row = ctx
            .txns()
            .pg()
            .query_opt(
                GET_BY_COMPONENT_AND_SYSTEM,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    component_id,
                    system_id,
                ],
            )
            .await?;
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        ctx: &DalContext<'_, '_>,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Self> {
        if system_id.is_none() {
            return Err(ResourceError::SystemIdRequired);
        }

        let resource =
            Resource::get_by_component_id_and_system_id(ctx, component_id, system_id).await?;

        if let Some(resource) = resource {
            Ok(resource)
        } else {
            Resource::new(ctx, component_id, system_id).await
        }
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
    pub data: serde_json::Value,
    pub health: ResourceHealth,
    pub entity_type: String,
}

impl From<(Resource, Option<FuncBindingReturnValue>)> for ResourceView {
    fn from((resource, fbrv): (Resource, Option<FuncBindingReturnValue>)) -> Self {
        // TODO: actually fill all of the data dynamically, most fields don't make much sense for now

        // TODO: do we want to have a special case for when the FuncBindingReturnValue is there, but the .value() returns None?
        if let Some((fbrv, result_json)) = fbrv.and_then(|f| f.value().cloned().map(|v| (f, v))) {
            Self {
                id: *resource.id(),
                created_at: fbrv.timestamp().created_at,
                updated_at: fbrv.timestamp().updated_at,
                error: Some("Boto Cor de Rosa Spotted at a Party".to_owned()),
                data: result_json,
                health: ResourceHealth::Error,
                entity_type: "idk bro".to_owned(),
            }
        } else {
            Self {
                id: *resource.id(),
                created_at: resource.timestamp().created_at,
                updated_at: resource.timestamp().updated_at,
                error: Some("Boto Cor de Rosa Spotted at a Party".to_owned()),
                data: serde_json::json!(null),
                health: ResourceHealth::Warning,
                entity_type: "idk bro".to_owned(),
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncId {
    component_id: ComponentId,
    system_id: SystemId,
}

impl WsEvent {
    pub fn resource_synced(
        component_id: ComponentId,
        system_id: SystemId,
        billing_account_ids: Vec<BillingAccountId>,
        history_actor: &HistoryActor,
    ) -> Self {
        WsEvent::new(
            billing_account_ids,
            history_actor.clone(),
            WsPayload::ResourceSynced(ResourceSyncId {
                component_id,
                system_id,
            }),
        )
    }
}
