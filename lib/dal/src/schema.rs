use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_many_to_many, BillingAccount, BillingAccountId, HistoryActor, HistoryEventError, Organization, OrganizationId, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility, Workspace, WorkspaceId, WsEvent, WsPayload, WsEventError};

#[derive(Error, Debug)]
pub enum SchemaError {
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
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

pk!(SchemaPk);
pk!(SchemaId);

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchemaKind {
    Concept,
    Implementation,
    Concrete,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaMenu {
    name: String,
    category: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pk: SchemaPk,
    id: SchemaId,
    name: String,
    kind: SchemaKind,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    ui_menu: Option<SchemaMenu>,
    ui_hidden: bool,
}

impl_standard_model! {
    model: Schema,
    pk: SchemaPk,
    id: SchemaId,
    table_name: "schemas",
    history_event_label_base: "schema",
    history_event_message_name: "Schema"
}

impl Schema {
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        kind: &SchemaKind,
    ) -> SchemaResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM schema_create_v1($1, $2, $3, $4)",
                &[&tenancy, &visibility, &name, &kind.to_string()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            &txn,
            &nats,
            &tenancy,
            &visibility,
            &history_actor,
            row,
        )
        .await?;
        WsEvent::schema_created(&object).publish(&nats).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, SchemaResult);
    standard_model_accessor!(kind, Enum(SchemaKind), SchemaResult);

    standard_model_many_to_many!(
        lookup_fn: billing_accounts,
        associate_fn: add_billing_account,
        disassociate_fn: remove_billing_account,
        table_name: "schema_many_to_many_billing_account",
        left_table: "schema",
        left_id: SchemaId,
        right_table: "billing_accounts",
        right_id: BillingAccountId,
        which_table_is_this: "left",
        returns: BillingAccount,
        result: SchemaResult,
    );

    standard_model_many_to_many!(
        lookup_fn: organizations,
        associate_fn: add_organization,
        disassociate_fn: remove_organization,
        table_name: "schema_many_to_many_organization",
        left_table: "schemas",
        left_id: SchemaId,
        right_table: "organizations",
        right_id: OrganizationId,
        which_table_is_this: "left",
        returns: Organization,
        result: SchemaResult,
    );

    standard_model_many_to_many!(
        lookup_fn: workspaces,
        associate_fn: add_workspace,
        disassociate_fn: remove_workspace,
        table_name: "schema_many_to_many_workspace",
        left_table: "schemas",
        left_id: SchemaId,
        right_table: "workspaces",
        right_id: WorkspaceId,
        which_table_is_this: "left",
        returns: Workspace,
        result: SchemaResult,
    );

    standard_model_many_to_many!(
        lookup_fn: in_menu_for_schemas,
        associate_fn: add_to_menu_for_schema,
        disassociate_fn: remove_from_menu_for_schema,
        table_name: "schema_many_to_many_in_menu_for_schema",
        left_table: "schemas",
        left_id: SchemaId,
        right_table: "schemas",
        right_id: SchemaId,
        which_table_is_this: "left",
        returns: Schema,
        result: SchemaResult,
    );

    standard_model_many_to_many!(
        lookup_fn: implements,
        associate_fn: add_implements_schema,
        disassociate_fn: remove_implements_schema,
        table_name: "schema_many_to_many_implements",
        left_table: "schemas",
        left_id: SchemaId,
        right_table: "schemas",
        right_id: SchemaId,
        which_table_is_this: "left",
        returns: Schema,
        result: SchemaResult,
    );
}

impl WsEvent {
    pub fn schema_created(schema: &Schema) -> Self {
        let billing_account_ids = WsEvent::billing_account_id_from_tenancy(&schema.tenancy);
        WsEvent::new(
            billing_account_ids,
            WsPayload::SchemaCreated(schema.pk),
        )
    }
}
