use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    edit_field::{
        value_and_visiblity_diff, value_and_visiblity_diff_copy, CheckboxWidget, EditField,
        EditFieldAble, EditFieldDataType, EditFieldError, EditFieldObjectKind, EditFields,
        RequiredValidator, TextWidget, ToSelectWidget, Validator, Widget,
    },
    impl_standard_model,
    label_list::ToLabelList,
    pk, standard_model, standard_model_accessor, standard_model_many_to_many, HistoryActor,
    HistoryEventError, SchemaVariant, SchemaVariantId, StandardModel, StandardModelError, Tenancy,
    Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum SocketError {
    #[error(transparent)]
    EditField(#[from] EditFieldError),
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
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketArity {
    Many,
    One,
}

impl ToLabelList for SocketArity {}
impl ToSelectWidget for SocketArity {}

#[derive(
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketEdgeKind {
    Component,
    Configures,
    Deployment,
}

impl ToLabelList for SocketEdgeKind {}
impl ToSelectWidget for SocketEdgeKind {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Socket {
    pk: SocketPk,
    id: SocketId,
    name: String,
    edge_kind: SocketEdgeKind,
    arity: SocketArity,
    required: bool,
    #[serde(flatten)]
    tenancy: Tenancy,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        edge_kind: &SocketEdgeKind,
        arity: &SocketArity,
    ) -> SocketResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM socket_create_v1($1, $2, $3, $4, $5)",
                &[
                    tenancy,
                    visibility,
                    &name,
                    &edge_kind.as_ref(),
                    &arity.as_ref(),
                ],
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

    standard_model_accessor!(name, String, SocketResult);
    standard_model_accessor!(edge_kind, Enum(SocketEdgeKind), SocketResult);
    standard_model_accessor!(arity, Enum(SocketArity), SocketResult);
    standard_model_accessor!(required, bool, SocketResult);

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

    fn edit_field_object_kind() -> EditFieldObjectKind {
        EditFieldObjectKind::Socket
    }

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SocketResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;

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
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::String,
            Widget::Text(TextWidget::new()),
            value,
            visibility_diff,
            vec![Validator::Required(RequiredValidator)],
        ))
    }

    fn edge_kind_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SocketResult<EditField> {
        let field_name = "edge_kind";
        let target_fn = Self::edge_kind;

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
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::String,
            Widget::Select(SocketEdgeKind::to_select_widget_with_no_default()?),
            value,
            visibility_diff,
            vec![Validator::Required(RequiredValidator)],
        ))
    }

    fn arity_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SocketResult<EditField> {
        let field_name = "arity";
        let target_fn = Self::arity;

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
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::String,
            Widget::Select(SocketArity::to_select_widget_with_no_default()?),
            value,
            visibility_diff,
            vec![Validator::Required(RequiredValidator)],
        ))
    }

    fn required_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SocketResult<EditField> {
        let field_name = "required";
        let target_fn = Self::required;

        let (value, visibility_diff) = value_and_visiblity_diff_copy(
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
            EditFieldDataType::Boolean,
            Widget::Checkbox(CheckboxWidget::new()),
            value,
            visibility_diff,
            vec![],
        ))
    }
}

#[async_trait]
impl EditFieldAble for Socket {
    type Id = SocketId;
    type Error = SocketError;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &SocketId,
    ) -> Result<EditFields, Self::Error> {
        let object = Self::get_by_id(txn, tenancy, visibility, id)
            .await?
            .ok_or(SocketError::NotFound(*id))?;
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
            Self::edge_kind_edit_field(visibility, &object, &head_object, &change_set_object)?,
            Self::arity_edit_field(visibility, &object, &head_object, &change_set_object)?,
            Self::required_edit_field(visibility, &object, &head_object, &change_set_object)?,
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
            .ok_or(SocketError::NotFound(id))?;

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
            "edge_kind" => match value {
                Some(json_value) => {
                    let value: SocketEdgeKind =
                        serde_json::from_value(json_value).map_err(|_| {
                            Self::Error::EditField(EditFieldError::InvalidValueType(
                                "SocketEdgeKind",
                            ))
                        })?;
                    object
                        .set_edge_kind(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "arity" => match value {
                Some(json_value) => {
                    let value: SocketArity = serde_json::from_value(json_value).map_err(|_| {
                        Self::Error::EditField(EditFieldError::InvalidValueType("SocketArity"))
                    })?;
                    object
                        .set_arity(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "required" => match value {
                Some(json_value) => {
                    let value = json_value.as_bool().ok_or(Self::Error::EditField(
                        EditFieldError::InvalidValueType("boolean"),
                    ))?;
                    object
                        .set_required(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }

        Ok(())
    }
}
