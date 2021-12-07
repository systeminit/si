use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use self::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    edit_field::{
        value_and_visiblity_diff, EditField, EditFieldAble, EditFieldDataType, EditFieldError,
        EditFieldObjectKind, EditFields, HeaderWidget, RequiredValidator, SelectWidget, TextWidget,
        Validator, VisibilityDiff, Widget,
    },
    impl_standard_model, pk,
    schema::ui_menu::UiMenuId,
    standard_model, standard_model_accessor, standard_model_has_many, standard_model_many_to_many,
    BillingAccount, BillingAccountId, HistoryActor, HistoryEventError, LabelEntry, LabelList,
    Organization, OrganizationId, StandardModel, StandardModelError, Tenancy, Timestamp,
    Visibility, Workspace, WorkspaceId, WsEventError,
};

pub use ui_menu::UiMenu;
pub use variant::SchemaVariant;

pub mod builtins;
pub mod ui_menu;
pub mod variant;

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
    #[error("schema not found: {0}")]
    NotFound(SchemaId),
    #[error("ui menu not found: {0}")]
    UiMenuNotFound(UiMenuId),
    #[error("edit field error: {0}")]
    EditField(#[from] EditFieldError),
    #[error("schema variant error: {0}")]
    Variant(#[from] SchemaVariantError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

pk!(SchemaPk);
pk!(SchemaId);

#[derive(AsRefStr, Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchemaKind {
    Concept,
    Implementation,
    Concrete,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
                &[&tenancy, &visibility, &name, &kind.as_ref()],
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

    standard_model_has_many!(
        lookup_fn: ui_menus,
        table: "schema_ui_menu_belongs_to_schema",
        model_table: "schema_ui_menus",
        returns: UiMenu,
        result: SchemaResult,
    );

    standard_model_has_many!(
        lookup_fn: variants,
        table: "schema_variant_belongs_to_schema",
        model_table: "schema_variants",
        returns: SchemaVariant,
        result: SchemaVariantResult,
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

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SchemaResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;
        let object_kind = EditFieldObjectKind::Schema;

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

    fn kind_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SchemaResult<EditField> {
        let field_name = "kind";
        let target_fn = Self::kind;
        let object_kind = EditFieldObjectKind::Schema;

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
            Widget::Select(SelectWidget::new(
                LabelList::new(vec![
                    LabelEntry::new(
                        SchemaKind::Concrete.to_string(),
                        serde_json::to_value(SchemaKind::Concrete)?,
                    ),
                    LabelEntry::new(
                        SchemaKind::Concept.to_string(),
                        serde_json::to_value(SchemaKind::Concept)?,
                    ),
                    LabelEntry::new(
                        SchemaKind::Implementation.to_string(),
                        serde_json::to_value(SchemaKind::Implementation)?,
                    ),
                ]),
                Some(serde_json::to_value(SchemaKind::Concrete)?),
            )),
            value,
            visibility_diff,
            vec![Validator::Required(RequiredValidator)],
        ))
    }

    async fn variants_edit_field(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        object: &Self,
    ) -> SchemaResult<EditField> {
        let field_name = "variants";
        let object_kind = EditFieldObjectKind::Schema;

        let mut items: Vec<EditFields> = vec![];
        for variant in object.variants(txn, visibility).await?.into_iter() {
            let edit_fields =
                SchemaVariant::get_edit_fields(txn, tenancy, visibility, variant.id()).await?;
            items.push(edit_fields);
        }

        Ok(EditField::new(
            field_name,
            vec![],
            object_kind,
            object.id,
            EditFieldDataType::None,
            Widget::Header(HeaderWidget::new(vec![EditField::new(
                "schemaVariants",
                vec![field_name.to_string()],
                EditFieldObjectKind::Schema,
                object.id,
                EditFieldDataType::Array,
                Widget::Array(items.into()),
                None,
                VisibilityDiff::None,
                vec![Validator::Required(RequiredValidator)],
            )])),
            None,
            VisibilityDiff::None,
            vec![Validator::Required(RequiredValidator)],
        ))
    }

    async fn ui_edit_field(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        object: &Self,
    ) -> SchemaResult<EditField> {
        let field_name = "ui";
        let object_kind = EditFieldObjectKind::Schema;

        let mut items: Vec<EditFields> = vec![];
        for ui_menu in object.ui_menus(txn, visibility).await?.into_iter() {
            let edit_fields =
                UiMenu::get_edit_fields(txn, tenancy, visibility, ui_menu.id()).await?;
            items.push(edit_fields);
        }

        Ok(EditField::new(
            field_name,
            vec![],
            object_kind,
            object.id,
            EditFieldDataType::None,
            Widget::Header(HeaderWidget::new(vec![EditField::new(
                "menuItems",
                vec![field_name.to_string()],
                EditFieldObjectKind::Schema,
                object.id,
                EditFieldDataType::Array,
                Widget::Array(items.into()),
                None,
                VisibilityDiff::None,
                vec![Validator::Required(RequiredValidator)],
            )])),
            None,
            VisibilityDiff::None,
            vec![Validator::Required(RequiredValidator)],
        ))
    }
}

#[async_trait]
impl EditFieldAble for Schema {
    type Id = SchemaId;
    type Error = SchemaError;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &SchemaId,
    ) -> SchemaResult<EditFields> {
        let object = Schema::get_by_id(txn, tenancy, visibility, id)
            .await?
            .ok_or(SchemaError::NotFound(*id))?;
        let head_object: Option<Schema> = if visibility.in_change_set() {
            let head_visibility = Visibility::new_head(visibility.deleted);
            Schema::get_by_id(txn, tenancy, &head_visibility, id).await?
        } else {
            None
        };
        let change_set_object: Option<Schema> = if visibility.in_change_set() {
            let change_set_visibility =
                Visibility::new_change_set(visibility.change_set_pk, visibility.deleted);
            Schema::get_by_id(txn, tenancy, &change_set_visibility, id).await?
        } else {
            None
        };

        Ok(vec![
            Self::name_edit_field(visibility, &object, &head_object, &change_set_object)?,
            Self::kind_edit_field(visibility, &object, &head_object, &change_set_object)?,
            Self::variants_edit_field(txn, tenancy, visibility, &object).await?,
            Self::ui_edit_field(txn, tenancy, visibility, &object).await?,
        ])
    }

    async fn update_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        id: Self::Id,
        edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> SchemaResult<()> {
        let edit_field_id = edit_field_id.as_ref();
        let mut object = Schema::get_by_id(txn, tenancy, visibility, &id)
            .await?
            .ok_or(SchemaError::NotFound(id))?;
        // value: None = remove value, Some(v) = set value
        match edit_field_id {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value
                        .as_str()
                        .expect("value must be a string, and it aint");
                    object
                        .set_name(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => panic!("cannot set the value"),
            },
            "kind" => match value {
                Some(json_value) => {
                    let value: SchemaKind = serde_json::from_value(json_value)
                        .expect("value must be a string, and it aint");
                    object
                        .set_kind(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => panic!("cannot set the value"),
            },
            "variants.schemaVariants" => {
                let variant = SchemaVariant::new(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    "TODO: name me!",
                )
                .await?;
                variant
                    .set_schema(txn, nats, visibility, history_actor, object.id())
                    .await?;
            }
            "ui.menuItems" => {
                let new_ui_menu =
                    UiMenu::new(txn, nats, tenancy, visibility, history_actor).await?;
                new_ui_menu
                    .set_schema(txn, nats, visibility, history_actor, object.id())
                    .await?;
            }
            invalid => panic!("TODO: invalid field name: {}", invalid),
        }
        Ok(())
    }
}
