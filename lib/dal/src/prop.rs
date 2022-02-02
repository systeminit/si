use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use si_data::{NatsTxn, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    edit_field::{
        value_and_visibility_diff, widget::prelude::*, EditField, EditFieldAble, EditFieldDataType,
        EditFieldError, EditFieldObjectKind, EditFields,
    },
    impl_standard_model,
    label_list::ToLabelList,
    pk, standard_model, standard_model_accessor, standard_model_many_to_many, HistoryActor,
    HistoryEventError, SchemaVariant, SchemaVariantId, StandardModel, StandardModelError, Tenancy,
    Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum PropError {
    #[error(transparent)]
    EditField(#[from] EditFieldError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("prop not found: {0} ({1:?})")]
    NotFound(PropId, Visibility),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type PropResult<T> = Result<T, PropError>;

pk!(PropPk);
pk!(PropId);

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
pub enum PropKind {
    Array,
    Boolean,
    Integer,
    PropObject,
    String,
}

impl ToLabelList for PropKind {}
impl ToSelectWidget for PropKind {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    pk: PropPk,
    id: PropId,
    name: String,
    kind: PropKind,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Prop,
    pk: PropPk,
    id: PropId,
    table_name: "props",
    history_event_label_base: "prop",
    history_event_message_name: "Prop"
}

impl Prop {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        kind: PropKind,
    ) -> PropResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4)",
                &[tenancy, visibility, &name.as_ref(), &kind.as_ref()],
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

    standard_model_accessor!(name, String, PropResult);
    standard_model_accessor!(kind, Enum(PropKind), PropResult);

    standard_model_many_to_many!(
        lookup_fn: schema_variants,
        associate_fn: add_schema_variant,
        disassociate_fn: remove_schema_variant,
        table_name: "prop_many_to_many_schema_variants",
        left_table: "props",
        left_id: PropId,
        right_table: "schema_variants",
        right_id: SchemaVariantId,
        which_table_is_this: "left",
        returns: SchemaVariant,
        result: PropResult,
    );

    fn edit_field_object_kind() -> EditFieldObjectKind {
        EditFieldObjectKind::Prop
    }

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> PropResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;

        let (value, visibility_diff) = value_and_visibility_diff(
            visibility,
            Some(object),
            target_fn,
            head_object.as_ref(),
            change_set_object.as_ref(),
        )?;

        Ok(EditField::new(
            field_name,
            vec![],
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::String,
            Widget::Text(TextWidget::new()),
            value,
            visibility_diff,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }

    fn kind_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> PropResult<EditField> {
        let field_name = "kind";
        let target_fn = Self::kind;

        let (value, visibility_diff) = value_and_visibility_diff(
            visibility,
            Some(object),
            target_fn,
            head_object.as_ref(),
            change_set_object.as_ref(),
        )?;

        Ok(EditField::new(
            field_name,
            vec![],
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::String,
            Widget::Select(PropKind::to_select_widget_with_no_default()?),
            value,
            visibility_diff,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }
}

#[async_trait]
impl EditFieldAble for Prop {
    type Id = PropId;
    type Error = PropError;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &Self::Id,
    ) -> Result<EditFields, Self::Error> {
        let object = Self::get_by_id(txn, tenancy, visibility, id)
            .await?
            .ok_or(Self::Error::NotFound(*id, *visibility))?;
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

        let edit_fields = vec![
            Self::name_edit_field(visibility, &object, &head_object, &change_set_object)?,
            Self::kind_edit_field(visibility, &object, &head_object, &change_set_object)?,
        ];

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
        value: Option<serde_json::Value>,
    ) -> Result<(), Self::Error> {
        let mut object = Self::get_by_id(txn, tenancy, visibility, &id)
            .await?
            .ok_or(Self::Error::NotFound(id, *visibility))?;

        match edit_field_id.as_ref() {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string()).ok_or(
                        Self::Error::EditField(EditFieldError::InvalidValueType("string")),
                    )?;
                    object
                        .set_name(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "kind" => match value {
                Some(json_value) => {
                    let value: PropKind = serde_json::from_value(json_value).map_err(|_| {
                        Self::Error::EditField(EditFieldError::InvalidValueType("PropKind"))
                    })?;
                    object
                        .set_kind(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }

        Ok(())
    }
}
