use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use si_data::{NatsTxn, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use veritech::EncryptionKey;

use crate::{
    attribute_resolver::AttributeResolverContext,
    deculture::{
        attribute::prototype::AttributePrototype, attribute::value::AttributeValue,
        AttributeContextBuilderError,
    },
    edit_field::{
        value_and_visibility_diff, widget::prelude::*, EditField, EditFieldAble, EditFieldDataType,
        EditFieldError, EditFieldObjectKind, EditFields,
    },
    func::binding::{FuncBinding, FuncBindingError},
    impl_standard_model,
    label_list::ToLabelList,
    pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_has_many, standard_model_many_to_many, AttributeResolver, Func, HistoryActor,
    HistoryEventError, SchemaVariant, SchemaVariantId, StandardModel, StandardModelError, Tenancy,
    Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum PropError {
    #[error("AttributeContext error: {0}")]
    AttributeContext(#[from] AttributeContextBuilderError),
    // Can't #[from] here, or we'll end up with circular error definitions.
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(String),
    // Can't #[from] here, or we'll end up with circular error definitions.
    #[error("AttributeResolver error: {0}")]
    AttributeResolver(String),
    #[error("AttributeValue error: {0}")]
    AttributeValue(String),
    #[error(transparent)]
    EditField(#[from] EditFieldError),
    #[error("FuncBinding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("missing a func: {0}")]
    MissingFunc(String),
    #[error("prop not found: {0} ({1:?})")]
    NotFound(PropId, Visibility),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("cannot set parent for a non object, array, or map prop: id {0} is a {1}. Bug!")]
    ParentNotAllowed(PropId, PropKind),
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
    Object,
    String,
    Map,
}

impl ToLabelList for PropKind {}
impl ToSelectWidget for PropKind {}

impl From<PropKind> for WidgetKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Checkbox,
            PropKind::String | PropKind::Integer => Self::Text,
            PropKind::Object | PropKind::Map => Self::Header,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    pk: PropPk,
    id: PropId,
    name: String,
    kind: PropKind,
    widget_kind: WidgetKind,
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
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        kind: PropKind,
    ) -> PropResult<Self> {
        let name = name.as_ref();
        let widget_kind = WidgetKind::from(kind);
        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5)",
                &[
                    tenancy,
                    visibility,
                    &name,
                    &kind.as_ref(),
                    &widget_kind.as_ref(),
                ],
            )
            .await?;
        let object: Prop = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;

        // Set default prop value as 'unset'
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;
        let func_name = "si:unset".to_string();
        let mut funcs =
            Func::find_by_attr(txn, &schema_tenancy, visibility, "name", &func_name).await?;
        let func = funcs.pop().ok_or(PropError::MissingFunc(func_name))?;
        let (func_binding, created) = FuncBinding::find_or_create(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            serde_json::json![null],
            *func.id(),
            *func.backend_kind(),
        )
        .await?;

        if created {
            func_binding
                .execute(txn, nats, veritech.clone(), encryption_key)
                .await?;
        }

        let mut attribute_resolver_context = AttributeResolverContext::new();
        attribute_resolver_context.set_prop_id(*object.id());
        AttributeResolver::upsert(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            *func.id(),
            *func_binding.id(),
            attribute_resolver_context.clone(),
            None,
        )
        .await
        .map_err(|e| PropError::AttributeResolver(format!("{e}")))?;

        AttributePrototype::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            *func.id(),
            *func_binding.id(),
            attribute_resolver_context.into(),
            None,
            None,
        )
        .await
        .map_err(|e| PropError::AttributePrototype(format!("{e}")))?;

        Ok(object)
    }

    standard_model_accessor!(name, String, PropResult);
    standard_model_accessor!(kind, Enum(PropKind), PropResult);
    standard_model_accessor!(widget_kind, Enum(WidgetKind), PropResult);

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

    standard_model_belongs_to!(
        lookup_fn: parent_prop,
        set_fn: set_parent_prop_unchecked,
        unset_fn: unset_parent_prop,
        table: "prop_belongs_to_prop",
        model_table: "props",
        belongs_to_id: PropId,
        returns: Prop,
        result: PropResult,
    );

    standard_model_has_many!(
        lookup_fn: child_props,
        table: "prop_belongs_to_prop",
        model_table: "props",
        returns: Prop,
        result: PropResult,
    );

    // FIXME(nick,jacob): this needs to be able to automatically set the parent attribute resolver as well.
    // See `integration_test::attribute_resolver::remove_for_context` for more details since its test setup
    // involves setting parent attribute resolvers for two child string props in a prop object even though
    // they have the prop object as their parent prop.
    pub async fn set_parent_prop(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        parent_prop_id: PropId,
    ) -> PropResult<()> {
        let parent_prop = Prop::get_by_id(txn, self.tenancy(), visibility, &parent_prop_id)
            .await?
            .ok_or(PropError::NotFound(parent_prop_id, *visibility))?;
        match parent_prop.kind() {
            PropKind::Object | PropKind::Map | PropKind::Array => (),
            kind => {
                return Err(PropError::ParentNotAllowed(parent_prop_id, *kind));
            }
        }

        let our_attribute_value =
            AttributeValue::find_for_prop(txn, self.tenancy(), visibility, *self.id())
                .await
                .map_err(|e| PropError::AttributeValue(format!("{e}")))?;
        let parent_attribute_value =
            AttributeValue::find_for_prop(txn, self.tenancy(), visibility, parent_prop_id)
                .await
                .map_err(|e| PropError::AttributeValue(format!("{e}")))?;
        our_attribute_value
            .set_parent_attribute_value(
                txn,
                nats,
                visibility,
                history_actor,
                parent_attribute_value.id(),
            )
            .await
            .map_err(|e| PropError::AttributeValue(format!("{e}")))?;

        self.set_parent_prop_unchecked(txn, nats, visibility, history_actor, &parent_prop_id)
            .await
    }

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
        _veritech: veritech::Client,
        _encryption_key: &EncryptionKey,
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
