use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_belongs_to, Component, ComponentId, HistoryActor, HistoryEventError,
    StandardModel, StandardModelError, System, SystemId, Tenancy, Timestamp, Visibility,
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

const FIND_FOR_COMPONENT_AND_SYSTEM: &str =
    include_str!("./queries/resource_find_for_component_and_system.sql");

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
    #[tracing::instrument(skip(txn, nats))]
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
    pub async fn find_for_component_id_and_system_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: &ComponentId,
        system_id: &SystemId,
    ) -> ResourceResult<Vec<Self>> {
        let rows = txn
            .query(
                FIND_FOR_COMPONENT_AND_SYSTEM,
                &[&tenancy, &visibility, component_id, system_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }
}
