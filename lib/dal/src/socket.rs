use serde::{Deserialize, Serialize};
use si_data::PgError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, label_list::ToLabelList, pk, standard_model, standard_model_accessor,
    standard_model_belongs_to, standard_model_many_to_many, DalContext, ExternalProvider,
    ExternalProviderId, HistoryEventError, InternalProvider, InternalProviderId, SchemaVariant,
    SchemaVariantId, SchematicKind, StandardModel, StandardModelError, Timestamp, Visibility,
    WriteTenancy,
};

#[derive(Error, Debug)]
pub enum SocketError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("socket not found: {0}")]
    NotFound(SocketId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
}

pub type SocketResult<T> = Result<T, SocketError>;

pk!(SocketPk);
pk!(SocketId);

#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketKind {
    Provider,
}

#[derive(
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketArity {
    Many,
    One,
}

impl ToLabelList for SocketArity {}

#[derive(
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketEdgeKind {
    Component,
    Configures,
    Deployment,
    Includes,
    Output,
}

impl ToLabelList for SocketEdgeKind {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Socket {
    pk: SocketPk,
    id: SocketId,
    name: String,
    kind: SocketKind,
    edge_kind: SocketEdgeKind,
    schematic_kind: SchematicKind,
    color: Option<i64>,
    arity: SocketArity,
    required: bool,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Socket,
    pk: SocketPk,
    id: SocketId,
    table_name: "sockets",
    history_event_label_base: "socket",
    history_event_message_name: "Socket"
}

impl Socket {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        kind: SocketKind,
        edge_kind: &SocketEdgeKind,
        arity: &SocketArity,
        schematic_kind: &SchematicKind,
    ) -> SocketResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM socket_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &kind.as_ref(),
                    &edge_kind.as_ref(),
                    &arity.as_ref(),
                    &schematic_kind.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    standard_model_accessor!(name, String, SocketResult);
    standard_model_accessor!(kind, Enum(SocketKind), SocketResult);
    standard_model_accessor!(edge_kind, Enum(SocketEdgeKind), SocketResult);
    standard_model_accessor!(arity, Enum(SocketArity), SocketResult);
    standard_model_accessor!(schematic_kind, Enum(SchematicKind), SocketResult);
    standard_model_accessor!(required, bool, SocketResult);
    standard_model_accessor!(color, OptionBigInt<i64>, SocketResult);

    standard_model_many_to_many!(
        lookup_fn: types,
        associate_fn: add_type,
        disassociate_fn: remove_type,
        table_name: "socket_many_to_many_schema_variants",
        left_table: "sockets",
        left_id: SocketId,
        right_table: "schema_variants",
        right_id: SchemaVariantId,
        which_table_is_this: "left",
        returns: SchemaVariant,
        result: SocketResult,
    );

    standard_model_belongs_to!(
        lookup_fn: internal_provider,
        set_fn: set_internal_provider,
        unset_fn: unset_internal_provider,
        table: "socket_belongs_to_internal_provider",
        model_table: "internal_providers",
        belongs_to_id: InternalProviderId,
        returns: InternalProvider,
        result: SocketResult,
    );

    standard_model_belongs_to!(
        lookup_fn: external_provider,
        set_fn: set_external_provider,
        unset_fn: unset_external_provider,
        table: "socket_belongs_to_external_provider",
        model_table: "external_providers",
        belongs_to_id: ExternalProviderId,
        returns: ExternalProvider,
        result: SocketResult,
    );
}
