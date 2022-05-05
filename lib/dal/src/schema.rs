use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::socket::SocketError;
use crate::WriteTenancy;
use crate::{
    component::ComponentKind,
    edit_field::{
        value_and_visibility_diff, widget::prelude::*, EditField, EditFieldAble, EditFieldDataType,
        EditFieldError, EditFieldObjectKind, VisibilityDiff,
    },
    func::binding::FuncBindingError,
    impl_standard_model,
    node::NodeKind,
    pk,
    schema::ui_menu::UiMenuId,
    standard_model, standard_model_accessor, standard_model_has_many, standard_model_many_to_many,
    AttributeContextBuilderError, AttributePrototypeError, AttributeValueError, BillingAccount,
    BillingAccountId, CodeGenerationPrototypeError, Component, DalContext, FuncError,
    HistoryEventError, LabelEntry, LabelList, Organization, OrganizationId, PropError,
    QualificationPrototypeError, ReadTenancyError, ResourcePrototypeError, SchematicKind,
    StandardModel, StandardModelError, Timestamp, ValidationPrototypeError, Visibility, Workspace,
    WorkspaceId, WsEventError,
};

pub use ui_menu::UiMenu;
pub use variant::root_prop::RootProp;
pub use variant::{SchemaVariant, SchemaVariantId};

pub mod ui_menu;
pub mod variant;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("AttributeContextBuilder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("code generation prototype error: {0}")]
    CodeGenerationPrototype(#[from] CodeGenerationPrototypeError),
    #[error("edit field error: {0}")]
    EditField(#[from] EditFieldError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func not found: {0}")]
    FuncNotFound(String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("no default variant for schema id: {0}")]
    NoDefaultVariant(SchemaId),
    #[error("schema not found: {0}")]
    NotFound(SchemaId),
    #[error("schema not found by name: {0}")]
    NotFoundByName(String),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("resource prototype error: {0}")]
    ResourcePrototype(#[from] ResourcePrototypeError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ui menu not found: {0}")]
    UiMenuNotFound(UiMenuId),
    #[error("schema variant error: {0}")]
    Variant(#[from] SchemaVariantError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

pk!(SchemaPk);
pk!(SchemaId);

#[derive(
    AsRefStr, Copy, Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchemaKind {
    /// High level abstract idea within SI domain (application, service, system...)
    Concept,
    /// Pairs with a concept schema (kubernetes_service is an implementation of a si_service...)
    Implementation,
    /// Low level model of a particular external domain (docker image, k8s deployment...)
    Concrete,
}

impl From<&SchemaKind> for SchematicKind {
    fn from(kind: &SchemaKind) -> Self {
        match kind {
            SchemaKind::Concept => Self::Deployment,
            SchemaKind::Concrete | SchemaKind::Implementation => Self::Component,
        }
    }
}

impl From<SchemaKind> for SchematicKind {
    fn from(kind: SchemaKind) -> Self {
        match kind {
            SchemaKind::Concept => Self::Deployment,
            SchemaKind::Concrete | SchemaKind::Implementation => Self::Component,
        }
    }
}

impl From<SchemaKind> for NodeKind {
    fn from(kind: SchemaKind) -> Self {
        match kind {
            SchemaKind::Concept => Self::Deployment,
            SchemaKind::Concrete | SchemaKind::Implementation => Self::Component,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Schema {
    pk: SchemaPk,
    id: SchemaId,
    name: String,
    kind: SchemaKind,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    ui_hidden: bool,
    default_schema_variant_id: Option<SchemaVariantId>,
    component_kind: ComponentKind,
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
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        kind: &SchemaKind,
        component_kind: &ComponentKind,
    ) -> SchemaResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM schema_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &kind.as_ref(),
                    &component_kind.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, SchemaResult);
    standard_model_accessor!(kind, Enum(SchemaKind), SchemaResult);
    standard_model_accessor!(component_kind, Enum(ComponentKind), SchemaResult);
    standard_model_accessor!(ui_hidden, bool, SchemaResult);
    standard_model_accessor!(
        default_schema_variant_id,
        OptionBigInt<SchemaVariantId>,
        SchemaResult
    );

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
        lookup_fn: components,
        table: "component_belongs_to_schema",
        model_table: "components",
        returns: Component,
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

    pub async fn default_variant(&self, ctx: &DalContext<'_, '_>) -> SchemaResult<SchemaVariant> {
        match self.default_schema_variant_id() {
            Some(schema_variant_id) => Ok(SchemaVariant::get_by_id(ctx, schema_variant_id)
                .await?
                .ok_or_else(|| SchemaError::NoDefaultVariant(*self.id()))?),
            None => Err(SchemaError::NoDefaultVariant(*self.id())),
        }
    }

    pub async fn default_schema_variant_id_for_name(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
    ) -> SchemaResult<SchemaVariantId> {
        let name = name.as_ref();
        let schemas = Schema::find_by_attr(ctx, "name", &name).await?;
        let schema = schemas
            .first()
            .ok_or_else(|| SchemaError::NotFoundByName(name.into()))?;
        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or_else(|| SchemaError::NoDefaultVariant(*schema.id()))?;

        Ok(*schema_variant_id)
    }

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SchemaResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;
        let object_kind = EditFieldObjectKind::Schema;

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
            object_kind,
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
    ) -> SchemaResult<EditField> {
        let field_name = "kind";
        let target_fn = Self::kind;
        let object_kind = EditFieldObjectKind::Schema;

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
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }

    async fn variants_edit_field(
        ctx: &DalContext<'_, '_>,
        object: &Self,
    ) -> SchemaResult<EditField> {
        let field_name = "variants";
        let object_kind = EditFieldObjectKind::Schema;

        let mut items: Vec<EditField> = vec![];
        for variant in object.variants(ctx).await?.into_iter() {
            let edit_fields = SchemaVariant::get_edit_fields(ctx, variant.id()).await?;
            items.extend(edit_fields);
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
                vec![], // TODO: actually validate to generate ValidationErrors
            )])),
            None,
            VisibilityDiff::None,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }

    async fn ui_edit_field(ctx: &DalContext<'_, '_>, object: &Self) -> SchemaResult<EditField> {
        let field_name = "ui";
        let object_kind = EditFieldObjectKind::Schema;

        let mut items: Vec<EditField> = vec![];
        for ui_menu in object.ui_menus(ctx).await?.into_iter() {
            let edit_fields = UiMenu::get_edit_fields(ctx, ui_menu.id()).await?;
            items.extend(edit_fields);
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
                vec![], // TODO: actually validate to generate ValidationErrors
            )])),
            None,
            VisibilityDiff::None,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }
}

#[async_trait]
impl EditFieldAble for Schema {
    type Id = SchemaId;
    type Error = SchemaError;

    async fn get_edit_fields(
        ctx: &DalContext<'_, '_>,
        id: &SchemaId,
    ) -> SchemaResult<Vec<EditField>> {
        let object = Schema::get_by_id(ctx, id)
            .await?
            .ok_or(SchemaError::NotFound(*id))?;
        let head_object: Option<Schema> = if ctx.visibility().in_change_set() {
            let head_visibility = ctx.visibility().to_head();
            let ctx = ctx.clone_with_new_visibility(head_visibility);
            Schema::get_by_id(&ctx, id).await?
        } else {
            None
        };
        let change_set_object: Option<Schema> = if ctx.visibility().in_change_set() {
            let change_set_visibility = ctx.visibility().to_change_set();
            let ctx = ctx.clone_with_new_visibility(change_set_visibility);
            Schema::get_by_id(&ctx, id).await?
        } else {
            None
        };

        Ok(vec![
            Self::name_edit_field(ctx.visibility(), &object, &head_object, &change_set_object)?,
            Self::kind_edit_field(ctx.visibility(), &object, &head_object, &change_set_object)?,
            Self::variants_edit_field(ctx, &object).await?,
            Self::ui_edit_field(ctx, &object).await?,
        ])
    }

    async fn update_from_edit_field(
        ctx: &DalContext<'_, '_>,
        id: Self::Id,
        edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> SchemaResult<()> {
        let edit_field_id = edit_field_id.as_ref();
        let mut object = Schema::get_by_id(ctx, &id)
            .await?
            .ok_or(SchemaError::NotFound(id))?;
        // value: None = remove value, Some(v) = set value
        match edit_field_id {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string()).ok_or(
                        Self::Error::EditField(EditFieldError::InvalidValueType("string")),
                    )?;
                    object.set_name(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "kind" => match value {
                Some(json_value) => {
                    let value: SchemaKind = serde_json::from_value(json_value).map_err(|_| {
                        Self::Error::EditField(EditFieldError::InvalidValueType("string"))
                    })?;
                    object.set_kind(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "variants.schemaVariants" => {
                let (_variant, _) = SchemaVariant::new(ctx, *object.id(), "TODO: name me!").await?;
            }
            "ui.menuItems" => {
                let new_ui_menu = UiMenu::new(ctx, &(*object.kind()).into()).await?;
                new_ui_menu.set_schema(ctx, object.id()).await?;
            }
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }
        Ok(())
    }
}
