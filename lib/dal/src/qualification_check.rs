use crate::DalContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    edit_field::{
        value_and_visibility_diff, widget::prelude::*, EditField, EditFieldAble, EditFieldDataType,
        EditFieldError, EditFieldObjectKind,
    },
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_many_to_many,
    HistoryEventError, SchemaVariant, SchemaVariantId, StandardModel, StandardModelError,
    Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum QualificationCheckError {
    #[error(transparent)]
    EditField(#[from] EditFieldError),
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
        ctx: &DalContext<'_, '_>,
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

    fn edit_field_object_kind() -> EditFieldObjectKind {
        EditFieldObjectKind::QualificationCheck
    }

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> QualificationCheckResult<EditField> {
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
}

#[async_trait]
impl EditFieldAble for QualificationCheck {
    type Id = QualificationCheckId;
    type Error = QualificationCheckError;

    async fn get_edit_fields(
        ctx: &DalContext<'_, '_>,
        id: &QualificationCheckId,
    ) -> Result<Vec<EditField>, Self::Error> {
        let object = Self::get_by_id(ctx, id)
            .await?
            .ok_or(QualificationCheckError::NotFound(*id))?;
        let head_object = if ctx.visibility().in_change_set() {
            let _head_visibility = ctx.visibility().to_head();
            Self::get_by_id(ctx, id).await?
        } else {
            None
        };
        let change_set_object = if ctx.visibility().in_change_set() {
            let _change_set_visibility = ctx.visibility().to_change_set();
            Self::get_by_id(ctx, id).await?
        } else {
            None
        };

        let edit_fields = vec![Self::name_edit_field(
            ctx.visibility(),
            &object,
            &head_object,
            &change_set_object,
        )?];

        Ok(edit_fields)
    }

    #[instrument(skip_all)]
    async fn update_from_edit_field(
        ctx: &DalContext<'_, '_>,
        id: Self::Id,
        edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> Result<(), Self::Error> {
        let mut object = Self::get_by_id(ctx, &id)
            .await?
            .ok_or(QualificationCheckError::NotFound(id))?;

        match edit_field_id.as_ref() {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string()).ok_or(
                        Self::Error::EditField(EditFieldError::InvalidValueType("string")),
                    )?;
                    object.set_name(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }

        Ok(())
    }
}
