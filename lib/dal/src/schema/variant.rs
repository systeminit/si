use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    edit_field::{
        value_and_visiblity_diff, EditField, EditFieldAble, EditFieldDataType, EditFieldError,
        EditFieldObjectKind, EditFields, RequiredValidator, TextWidget, Validator, Widget,
    },
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    HistoryActor, HistoryEventError, Schema, SchemaId, StandardModel, StandardModelError, Tenancy,
    Timestamp, Visibility, WsEventError,
};

#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error(transparent)]
    EditField(#[from] EditFieldError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("schema not found: {0}")]
    NotFound(SchemaVariantId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

pk!(SchemaVariantPk);
pk!(SchemaVariantId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    name: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: SchemaVariant,
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    table_name: "schema_variants",
    history_event_label_base: "schema_variant",
    history_event_message_name: "Schema Variant"
}

impl SchemaVariant {
    #[instrument(skip_all)]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> SchemaVariantResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM schema_variant_create_v1($1, $2, $3)",
                &[tenancy, visibility, &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, SchemaVariantResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "schema_variant_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: SchemaVariantResult,
    );

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SchemaVariantResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;
        let object_kind = EditFieldObjectKind::SchemaVariant;

        let (value, visibility_diff) = value_and_visiblity_diff(
            visibility,
            Some(object),
            target_fn,
            head_object.as_ref(),
            change_set_object.as_ref(),
        )?;

        Ok(EditField::new(
            field_name,
            vec![],
            object_kind,
            object.id,
            EditFieldDataType::String,
            Widget::Text(TextWidget::new()),
            value,
            visibility_diff,
            vec![Validator::Required(RequiredValidator)],
        ))
    }
}

#[async_trait]
impl EditFieldAble for SchemaVariant {
    type Id = SchemaVariantId;
    type ErrorKind = SchemaVariantError;

    #[instrument(skip_all)]
    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &Self::Id,
    ) -> Result<EditFields, Self::ErrorKind> {
        let object = Self::get_by_id(txn, tenancy, visibility, id)
            .await?
            .ok_or(SchemaVariantError::NotFound(*id))?;
        let head_object = if visibility.in_change_set() {
            let head_visibility = visibility.to_head();
            Self::get_by_id(txn, tenancy, &head_visibility, id).await?
        } else {
            None
        };
        let change_set_object = if visibility.in_change_set() {
            let change_set_visibility = visibility.to_change_set();
            Self::get_by_id(txn, tenancy, &change_set_visibility, id).await?
        } else {
            None
        };

        let edit_fields = vec![Self::name_edit_field(
            visibility,
            &object,
            &head_object,
            &change_set_object,
        )?];

        Ok(edit_fields)
    }

    #[instrument(skip_all)]
    async fn update_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        id: Self::Id,
        edit_field_id: String,
        value: Option<Value>,
    ) -> Result<(), Self::ErrorKind> {
        let mut object = Self::get_by_id(txn, tenancy, visibility, &id)
            .await?
            .ok_or(SchemaVariantError::NotFound(id))?;

        match edit_field_id.as_ref() {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value
                        .as_str()
                        .map(|s| s.to_string())
                        .expect("TODO: value is not a string");
                    object
                        .set_name(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => panic!("TODO: value for name not provided, cannot set value"),
            },
            invalid => panic!("TODO: invalid field name: {}", invalid),
        }

        Ok(())
    }
}
