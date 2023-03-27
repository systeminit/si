use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::component::{ComponentResult, COMPONENT_STATUS_UPDATE_BY_PK};
use crate::standard_model::TypeHint;
use crate::{
    impl_standard_model, pk, standard_model, ComponentId, DalContext, HistoryActor,
    StandardModelError, Tenancy, Timestamp, UserPk, Visibility,
};

pk!(ComponentStatusPk);

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryActorTimestamp {
    pub actor: HistoryActor,
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ComponentStatus {
    pk: ComponentStatusPk,
    // This is a `ComponentId` as the underlying table is parallel to the components table
    id: ComponentId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    creation_timestamp: DateTime<Utc>,
    creation_user_pk: Option<UserPk>,
    update_timestamp: DateTime<Utc>,
    update_user_pk: Option<UserPk>,
}

impl_standard_model! {
    model: ComponentStatus,
    pk: ComponentStatusPk,
    id: ComponentId,
    table_name: "component_statuses",
    history_event_label_base: "component_status",
    history_event_message_name: "Component Status"
}

impl ComponentStatus {
    pub fn creation(&self) -> HistoryActorTimestamp {
        HistoryActorTimestamp {
            actor: self.actor(),
            timestamp: self.creation_timestamp,
        }
    }

    pub fn update(&self) -> HistoryActorTimestamp {
        HistoryActorTimestamp {
            actor: self.actor(),
            timestamp: self.update_timestamp,
        }
    }

    /// Persists updated 'update' timestamp/actor data by [`ComponentId`] and returns the update
    /// timestamp.
    ///
    /// # Errors
    ///
    /// Return [`Err`] if the upsert failed or if there was a connection issue to the database.
    pub async fn record_update_by_id(
        ctx: &DalContext,
        id: ComponentId,
    ) -> ComponentResult<DateTime<Utc>> {
        let actor_user_pk = Self::user_pk(ctx.history_actor());

        // TODO(fnichol): I would *highly* prefer to avoid 2 `UPDATE` statements, but our standard
        // model update code understands how to properly upsert a record to the correct visibility.
        // That is, we might be updating a record that exists only so far in HEAD, and therefore a
        // new change set record must be created. The first `update()` call guarentees this upsert
        // and the second call is effectively executing the "update-not-insert" code path, but
        // since we get arbitrary field updates for free and there's only one more field to update,
        // why not call it again?.
        //
        // If we decide to extract the standard model upsert logic, then a custom db function could
        // be written to use that and called once from here--I'm too nervous to duplicate the
        // upsert code to save on *1* more db statement call.
        let update_timestamp = standard_model::update(
            ctx,
            "component_statuses",
            "update_user_pk",
            &id,
            &actor_user_pk,
            TypeHint::BpChar,
        )
        .await?;
        let _updated_at = standard_model::update(
            ctx,
            "component_statuses",
            "update_timestamp",
            &id,
            &update_timestamp,
            TypeHint::TimestampWithTimeZone,
        )
        .await?;

        Ok(update_timestamp)
    }

    /// Persists updated 'update' timestamp/actor data and returns the update timestamp.
    ///
    /// # Errors
    ///
    /// Return [`Err`] if there was a connection issue to the database.
    pub async fn record_update(&mut self, ctx: &DalContext) -> ComponentResult<DateTime<Utc>> {
        let actor_user_pk = Self::user_pk(ctx.history_actor());

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(COMPONENT_STATUS_UPDATE_BY_PK, &[&self.pk, &actor_user_pk])
            .await?;
        let updated_at = row.try_get("updated_at").map_err(|_| {
            StandardModelError::ModelMissing("component_statuses".to_string(), self.pk.to_string())
        })?;
        let update_timestamp = row.try_get("update_timestamp").map_err(|_| {
            StandardModelError::ModelMissing("component_statuses".to_string(), self.pk.to_string())
        })?;
        self.timestamp.updated_at = updated_at;
        self.update_timestamp = update_timestamp;
        self.update_user_pk = actor_user_pk;

        Ok(update_timestamp)
    }

    fn actor(&self) -> HistoryActor {
        match self.creation_user_pk {
            Some(user_pk) => user_pk.into(),
            None => HistoryActor::SystemInit,
        }
    }

    fn user_pk(history_actor: &HistoryActor) -> Option<UserPk> {
        match history_actor {
            HistoryActor::User(user_pk) => Some(*user_pk),
            _ => None,
        }
    }
}
