//! This module contains [`CodeGenerationPrototype`], which is used to represent the
//! [`Func`](crate::Func) underneath the "/root/code" [`Prop`](crate::Prop) tree for a
//! [`SchemaVariant`](crate::SchemaVariant).

use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, Value};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::argument::{FuncArgumentError, FuncArgumentId};
use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::schema::variant::SchemaVariantError;
use crate::{
    func::FuncId, impl_standard_model, pk, standard_model, standard_model_accessor,
    AttributeContext, AttributeContextBuilderError, AttributePrototypeArgument,
    AttributePrototypeArgumentError, AttributePrototypeError, AttributeReadContext, AttributeValue,
    AttributeValueError, CodeLanguage, ComponentId, Func, FuncError, HistoryEventError,
    InternalProvider, InternalProviderError, Prop, PropError, PropKind, SchemaVariant,
    SchemaVariantId, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
    WsEvent, WsPayload,
};
use crate::{DalContext, PropId};

const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("queries/code_generation_prototype_list_for_schema_variant.sql");

#[derive(Error, Debug)]
pub enum CodeGenerationPrototypeError {
    #[error(transparent)]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error(transparent)]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("must provide valid schema variant, found unset schema variant id")]
    InvalidSchemaVariant,
}

pub type CodeGenerationPrototypeResult<T> = Result<T, CodeGenerationPrototypeError>;

pk!(CodeGenerationPrototypePk);
pk!(CodeGenerationPrototypeId);

/// A [`CodeGenerationPrototype`] joins a [`Func`](crate::Func) to a [`SchemaVariant`](crate::SchemaVariant)
/// in order to generate code based on the current state of the corresponding [`Prop`](crate::Prop)
/// tree.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CodeGenerationPrototype {
    pk: CodeGenerationPrototypePk,
    id: CodeGenerationPrototypeId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,

    /// The [`Func`](crate::Func) used for execution. For all [`prototypes`](self) for a given
    /// [`SchemaVariant`](crate::SchemaVariant), there cannot be duplicate [`Funcs`](crate::Func).
    func_id: FuncId,
    /// The arguments to the [`Func`](crate::Func).
    args: Value,
    /// The [`Prop`](crate::Prop) that provides the tree containing generated code. The children
    /// of this [`Prop`](crate::Prop) correspond to the fields off of the corresponding
    /// [`CodeGenerated`](veritech_client::CodeGenerated) object.
    tree_prop_id: PropId,
    /// The child [`Prop`](crate::Prop) of the tree containing the "code" of the latest code
    /// generation result.
    code_prop_id: PropId,
    /// The child [`Prop`](crate::Prop) of the tree containing the "format" of the latest code
    /// generation result.
    format_prop_id: PropId,
    /// The [`SchemaVariant`](crate::SchemaVariant) that the [`Prop`](crate::Prop) belongs to
    /// underneath "/root/code". This field technically isn't necessary since a
    /// [`PropId`](crate::Prop) can only belong to one or zero [`variants`](crate::SchemaVariant).
    /// This field is used for lookups.
    schema_variant_id: SchemaVariantId,
}

impl_standard_model! {
    model: CodeGenerationPrototype,
    pk: CodeGenerationPrototypePk,
    id: CodeGenerationPrototypeId,
    table_name: "code_generation_prototypes",
    history_event_label_base: "code_generation_prototype",
    history_event_message_name: "Code Generation Prototype"
}

impl CodeGenerationPrototype {
    /// Create a new [`CodeGenerationPrototype`] with a corresponding [`Prop`](crate::Prop)
    /// underneath the "/root/code" field for a [`SchemaVariant`](crate::SchemaVariant).
    /// If "args" are not provided, a default set of "args" will be created.
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        func_argument_id: FuncArgumentId,
        schema_variant_id: SchemaVariantId,
        format: CodeLanguage,
    ) -> CodeGenerationPrototypeResult<Self> {
        if schema_variant_id.is_none() {
            return Err(CodeGenerationPrototypeError::InvalidSchemaVariant);
        }

        // Collect the root prop for the schema variant as we will need it to setup new props
        // and intelligence.
        let root_prop = SchemaVariant::root_prop(ctx, schema_variant_id).await?;

        // The new prop is named after the func name since func names must be unique for a given
        // tenancy and visibility. If that changes, then this may break.
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let mut tree_prop = Prop::new(ctx, func.name(), PropKind::Object, None).await?;
        tree_prop.set_hidden(ctx, true).await?;
        tree_prop
            .set_parent_prop(ctx, root_prop.code_prop_id)
            .await?;
        let tree_prop_id = *tree_prop.id();

        // Now, create the two child props of the new prop. These represent the code generation
        // response fields.
        let mut child_code_prop = Prop::new(ctx, "code", PropKind::String, None).await?;
        child_code_prop.set_hidden(ctx, true).await?;
        child_code_prop.set_parent_prop(ctx, tree_prop_id).await?;
        let child_code_prop_id = *child_code_prop.id();

        let mut child_format_prop = Prop::new(ctx, "format", PropKind::String, None).await?;
        child_format_prop.set_hidden(ctx, true).await?;
        child_format_prop.set_parent_prop(ctx, tree_prop_id).await?;
        let child_format_prop_id = *child_format_prop.id();

        // NOTE(nick): we may not need these on the table itself anymore since they will all be
        // set to default.
        let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
        let args = serde_json::to_value(&code_generation_args)?;

        // Finalize the schema variant (again).
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;
        schema_variant.finalize(ctx).await?;

        // FIXME(nick): once we fix the bug where child props of prop objects with functions that
        // set nested complex objects does not result in the internal providers for those child
        // props being updated we can use the function on the tree prop instead of the code prop.
        // For now, let's manually set the format prop and then set the function on the code prop.
        let format_attribute_context = AttributeContext::builder()
            .set_prop_id(child_format_prop_id)
            .to_context()?;
        let format_attribute_value =
            AttributeValue::find_for_context(ctx, format_attribute_context.into())
                .await?
                .ok_or_else(|| {
                    AttributeValueError::NotFoundForReadContext(format_attribute_context.into())
                })?;
        let tree_attribute_value = format_attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| AttributeValueError::ParentNotFound(*format_attribute_value.id()))?;

        // Following the steps in the "FIXME" above, use the format parameter as the value here.
        // We will eventually no longer use this parameter as the function itself should set the
        // output format in the future as it should be dynamic.
        AttributeValue::update_for_context(
            ctx,
            *format_attribute_value.id(),
            Some(*tree_attribute_value.id()),
            format_attribute_context,
            Some(serde_json::to_value(format)?),
            None,
        )
        .await?;

        // Following the steps in the "FIXME" above, set the function on the child code field.
        let code_attribute_read_context = AttributeReadContext {
            prop_id: Some(child_code_prop_id),
            ..AttributeReadContext::default()
        };
        let code_attribute_value =
            AttributeValue::find_for_context(ctx, code_attribute_read_context)
                .await?
                .ok_or(AttributeValueError::NotFoundForReadContext(
                    code_attribute_read_context,
                ))?;
        let mut code_attribute_prototype = code_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        code_attribute_prototype.set_func_id(ctx, func_id).await?;
        let domain_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, root_prop.domain_prop_id)
                .await?
                .ok_or(InternalProviderError::NotFoundForProp(
                    root_prop.domain_prop_id,
                ))?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *code_attribute_prototype.id(),
            func_argument_id,
            *domain_implicit_internal_provider.id(),
        )
        .await?;

        // Finally, we can create the code generation prototype.
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM code_generation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &args,
                    &tree_prop_id,
                    &child_code_prop_id,
                    &child_format_prop_id,
                    &schema_variant_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    // FIXME(nick): this is not right at all. However, we want "save func" and "create func"
    // work. Creating code prototypes in progress is likely useful. Thus, once
    // we get serious about authoring code prototypes, we will need to think
    // about transitory states further. As a result, this function should _only_ be used for
    // function authoring.
    #[instrument(skip_all)]
    pub async fn new_temporary(
        ctx: &DalContext,
        func_id: FuncId,
        args: Option<Value>,
    ) -> CodeGenerationPrototypeResult<Self> {
        let args = match args {
            Some(args) => args,
            _ => {
                let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
                serde_json::to_value(&code_generation_args)?
            }
        };

        let tree_prop_id = PropId::NONE;
        let code_prop_id = PropId::NONE;
        let format_prop_id = PropId::NONE;
        let schema_variant_id = SchemaVariantId::NONE;

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM code_generation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &args,
                    &tree_prop_id,
                    &code_prop_id,
                    &format_prop_id,
                    &schema_variant_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(func_id, Pk(FuncId), CodeGenerationPrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, CodeGenerationPrototypeResult);

    pub fn tree_prop_id(&self) -> PropId {
        self.tree_prop_id
    }

    pub fn code_prop_id(&self) -> PropId {
        self.code_prop_id
    }

    pub fn format_prop_id(&self) -> PropId {
        self.format_prop_id
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    #[instrument(skip_all)]
    pub async fn list_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> CodeGenerationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_SCHEMA_VARIANT,
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeGeneratedPayload {
    component_id: ComponentId,
}

impl WsEvent {
    pub fn code_generated(ctx: &DalContext, component_id: ComponentId) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::CodeGenerated(CodeGeneratedPayload { component_id }),
        )
    }
}
