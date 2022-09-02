use crate::WriteTenancy;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_has_many, standard_model_many_to_many, BillingAccount, BillingAccountId,
    Capability, DalContext, HistoryEventError, StandardModel, StandardModelError, Timestamp, User,
    UserId, Visibility,
};

#[derive(Error, Debug)]
pub enum GroupError {
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

pub type GroupResult<T> = Result<T, GroupError>;

pk!(GroupPk);
pk!(GroupId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pk: GroupPk,
    id: GroupId,
    name: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Group,
    pk: GroupPk,
    id: GroupId,
    table_name: "groups",
    history_event_label_base: "group",
    history_event_message_name: "Group"
}

impl Group {
    #[instrument(skip_all)]
    pub async fn new(ctx: &DalContext<'_, '_, '_>, name: impl AsRef<str>) -> GroupResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM group_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, GroupResult);

    standard_model_belongs_to!(
        lookup_fn: billing_account,
        set_fn: set_billing_account,
        unset_fn: unset_billing_account,
        table: "group_belongs_to_billing_account",
        model_table: "billing_accounts",
        belongs_to_id: BillingAccountId,
        returns: BillingAccount,
        result: GroupResult,
    );

    standard_model_many_to_many!(
        lookup_fn: users,
        associate_fn: add_user,
        disassociate_fn: remove_user,
        table_name: "group_many_to_many_users",
        left_table: "groups",
        left_id: GroupId,
        right_table: "users",
        right_id: UserId,
        which_table_is_this: "left",
        returns: User,
        result: GroupResult,
    );

    standard_model_has_many!(
        lookup_fn: capabilities,
        table: "capability_belongs_to_group",
        model_table: "capabilities",
        returns: Capability,
        result: GroupResult,
    );
}
