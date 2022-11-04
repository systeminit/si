//! This module contains [`CodeGenerationPrototype`], which is used to generate code based on the status
//! of a [`Prop`](crate::Prop) tree specified by a [`SchemaVariant`](crate::SchemaVariant).
//!
//! These prototypes are used in the [`CodeGeneration`](crate::job::definition::CodeGeneration) job
//! in order to perform the generation for a given [`ComponentId`](crate::Component) and set the
//! resulting value on the corresponding [`Prop`](crate::Prop) underneath "/root/code".

use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, Value};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::schema::variant::SchemaVariantError;
use crate::{
    func::FuncId, impl_standard_model, pk, standard_model, standard_model_accessor, CodeLanguage,
    ComponentId, Func, FuncError, HistoryEventError, Prop, PropError, PropKind, SchemaVariant,
    SchemaVariantId, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
    WsEvent, WsPayload,
};
use crate::{DalContext, PropId};

const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("queries/code_generation_prototype_list_for_schema_variant.sql");
const FIND_FOR_PROP: &str = include_str!("queries/code_generation_prototype_find_for_prop.sql");

#[derive(Error, Debug)]
pub enum CodeGenerationPrototypeError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
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
    /// The format of the output of the [`Func`](crate::Func).
    output_format: CodeLanguage,
    /// The [`Prop`](crate::Prop) that provides the tree needed for code
    /// generation.
    prop_id: PropId,
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
        args: Option<Value>,
        output_format: CodeLanguage,
        schema_variant_id: SchemaVariantId,
    ) -> CodeGenerationPrototypeResult<Self> {
        if schema_variant_id.is_none() {
            return Err(CodeGenerationPrototypeError::InvalidSchemaVariant);
        }

        let args = match args {
            Some(args) => args,
            None => {
                let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
                serde_json::to_value(&code_generation_args)?
            }
        };

        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;
        let code_prop = schema_variant.code_prop(ctx).await?;

        // The new prop is named after the func name since func names must be unique for a given
        // tenancy and visibility. If that changes, then this may break.
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let new_prop = Prop::new(ctx, func.name(), PropKind::String, None).await?;
        new_prop.set_parent_prop(ctx, *code_prop.id()).await?;
        let new_prop_id = *new_prop.id();

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM code_generation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &args,
                    &output_format.as_ref(),
                    &new_prop_id,
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
        output_format: CodeLanguage,
    ) -> CodeGenerationPrototypeResult<Self> {
        let args = match args {
            Some(args) => args,
            _ => {
                let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
                serde_json::to_value(&code_generation_args)?
            }
        };

        let prop_id = PropId::NONE;
        let schema_variant_id = SchemaVariantId::NONE;

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM code_generation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &args,
                    &output_format.as_ref(),
                    &prop_id,
                    &schema_variant_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(func_id, Pk(FuncId), CodeGenerationPrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, CodeGenerationPrototypeResult);
    standard_model_accessor!(
        output_format,
        Enum(CodeLanguage),
        CodeGenerationPrototypeResult
    );

    pub fn prop_id(&self) -> PropId {
        self.prop_id
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

    #[instrument(skip_all)]
    pub async fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> CodeGenerationPrototypeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                FIND_FOR_PROP,
                &[ctx.read_tenancy(), ctx.visibility(), &prop_id],
            )
            .await?;
        Ok(standard_model::object_from_row(row)?)
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
