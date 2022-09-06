use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_many_to_many,
    DalContext, HistoryEventError, SchemaVariant, SchemaVariantId, StandardModel,
    StandardModelError, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum QualificationCheckError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("qualification check not found: {0}")]
    NotFound(QualificationCheckId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
}

pub type QualificationCheckResult<T> = Result<T, QualificationCheckError>;

pk!(QualificationCheckPk);
pk!(QualificationCheckId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QualificationCheck {
    pk: QualificationCheckPk,
    id: QualificationCheckId,
    name: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: QualificationCheck,
    pk: QualificationCheckPk,
    id: QualificationCheckId,
    table_name: "qualification_checks",
    history_event_label_base: "qualification_check",
    history_event_message_name: "Qualification Check"
}

impl QualificationCheck {
    pub async fn new(
        ctx: &DalContext<'_, '_, '_>,
        name: impl AsRef<str>,
    ) -> QualificationCheckResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM qualification_check_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    standard_model_accessor!(name, String, QualificationCheckResult);

    standard_model_many_to_many!(
        lookup_fn: schema_variants,
        associate_fn: add_schema_variant,
        disassociate_fn: remove_schema_variant,
        table_name: "qualification_check_many_to_many_schema_variants",
        left_table: "qualification_checks",
        left_id: QualificationCheckId,
        right_table: "schema_variants",
        right_id: SchemaVariantId,
        which_table_is_this: "left",
        returns: SchemaVariant,
        result: QualificationCheckResult,
    );
}
