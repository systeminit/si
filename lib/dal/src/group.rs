use crate::WriteTenancy;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_has_many,
    standard_model_many_to_many, BillingAccount, BillingAccountError, BillingAccountPk, Capability,
    DalContext, HistoryEventError, StandardModel, StandardModelError, Timestamp, User, UserId,
    Visibility,
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
    #[error(transparent)]
    BillingAccount(#[from] Box<BillingAccountError>),
}

pub type GroupResult<T> = Result<T, GroupError>;

pk!(GroupPk);
pk!(GroupId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pk: GroupPk,
    id: GroupId,
    name: String,
    billing_account_pk: BillingAccountPk,
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
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        billing_account_pk: BillingAccountPk,
    ) -> GroupResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM group_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &billing_account_pk,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, GroupResult);
    standard_model_accessor!(billing_account_pk, Pk(BillingAccountPk), GroupResult);

    pub async fn billing_account(&self, ctx: &DalContext) -> GroupResult<BillingAccount> {
        Ok(BillingAccount::get_by_pk(ctx, &self.billing_account_pk)
            .await
            .map_err(Box::new)?)
    }

    standard_model_many_to_many!(
        lookup_fn: users,
        associate_fn: add_user,
        disassociate_fn: remove_user,
        disassociate_all_fn: remove_all_users,
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
