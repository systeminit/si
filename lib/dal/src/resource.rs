use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use strum_macros::Display;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_belongs_to,
    ws_event::{WsEvent, WsPayload},
    BillingAccountId, Component, ComponentId, HistoryActor, HistoryEventError, StandardModel,
    StandardModelError, System, SystemId, Tenancy, Timestamp, Visibility,
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
    StandardModelError(#[from] StandardModelError),
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
    tenancy: Tenancy,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM resource_create_v1($1, $2)",
                &[&tenancy, &visibility],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;

        object
            .set_component(txn, nats, visibility, history_actor, component_id)
            .await?;
        object
            .set_system(txn, nats, visibility, history_actor, system_id)
            .await?;

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
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Option<Self>> {
        let row = txn
            .query_opt(
                GET_BY_COMPONENT_AND_SYSTEM,
                &[&tenancy, &visibility, component_id, system_id],
            )
            .await?;
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Self> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let resource = Resource::get_by_component_id_and_system_id(
            txn,
            &schema_tenancy,
            visibility,
            component_id,
            system_id,
        )
        .await?;

        if let Some(resource) = resource {
            Ok(resource)
        } else {
            Resource::new(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                component_id,
                system_id,
            )
            .await
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
