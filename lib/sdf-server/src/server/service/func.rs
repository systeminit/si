use crate::server::{impl_default_error_into_response, state::AppState};
use crate::service::func::get_func::GetFuncResponse;
use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::{
    attribute::context::{AttributeContextBuilder, AttributeContextBuilderError},
    func::{
        argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind},
        binding_return_value::FuncBindingReturnValueError,
    },
    prop_tree::PropTreeError,
    prototype_context::PrototypeContextError,
    schema::variant::SchemaVariantError,
    AttributeContext, AttributeContextError, AttributePrototype, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId, AttributePrototypeError, AttributePrototypeId,
    AttributeValueError, ComponentError, ComponentId, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncBindingError, FuncId, InternalProviderError, InternalProviderId,
    Prop, PropError, PropId, PropKind, PrototypeListForFuncError, SchemaVariantId, StandardModel,
    StandardModelError, TenancyError, TransactionsError, ValidationPrototype,
    ValidationPrototypeError, ValidationPrototypeId, WsEventError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod create_func;
pub mod get_func;
pub mod list_funcs;
pub mod list_input_sources;
pub mod revert_func;
pub mod save_and_exec;
pub mod save_func;

#[derive(Error, Debug)]
pub enum FuncError {
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("prop tree error: {0}")]
    PropTree(#[from] PropTreeError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("prototype context error: {0}")]
    PrototypeContext(#[from] PrototypeContextError),
    #[error("prototype list for func error: {0}")]
    PrototypeListForFunc(#[from] PrototypeListForFuncError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),

    #[error("Function not found")]
    FuncNotFound,
    #[error("Function is read-only")]
    NotWritable,
    #[error("Missing required options for creating a function")]
    MissingOptions,
    #[error("Cannot create that type of function")]
    FuncNotSupported,
    #[error("attribute value missing")]
    AttributeValueMissing,
    #[error("attribute prototype missing")]
    AttributePrototypeMissing,
    #[error("prop for value not found")]
    PropNotFound,
    #[error("func binding return value not found")]
    FuncBindingReturnValueMissing,
    #[error("func is not revertible")]
    FuncNotRevertible,
    #[error("func argument already exists for that name")]
    FuncArgumentAlreadyExists,
    #[error("func argument not found")]
    FuncArgNotFound,
    #[error("attribute prototype {0} has no prop_id")]
    AttributePrototypeMissingPropId(AttributePrototypeId),
    #[error("attribute prototype {0} schema_variant is missing")]
    AttributePrototypeMissingSchemaVariant(AttributePrototypeId),
    #[error("attribute prototype {0} schema is missing")]
    AttributePrototypeMissingSchema(AttributePrototypeId),
    #[error("attribute prototype {0} is missing its prop {1}")]
    AttributePrototypeMissingProp(AttributePrototypeId, PropId),
    #[error("attribute prototype {0} is missing argument {1}")]
    AttributePrototypeMissingArgument(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("attribute prototype argument {0} is internal provider id")]
    AttributePrototypeMissingInternalProviderId(AttributePrototypeArgumentId),
    #[error("func argument {0} missing attribute prototype argument for prototype {1}")]
    FuncArgumentMissingPrototypeArgument(FuncArgumentId, AttributePrototypeId),
    #[error("validation prototype {0} schema_variant is missing")]
    ValidationPrototypeMissingSchemaVariant(SchemaVariantId),
    #[error("validation prototype schema is missing")]
    ValidationPrototypeMissingSchema,
    #[error("component missing schema variant")]
    ComponentMissingSchemaVariant(ComponentId),
    #[error("schema variant missing schema")]
    SchemaVariantMissingSchema(SchemaVariantId),
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    #[error("unexpected func variant ({0:?}) creating attribute func")]
    UnexpectedFuncVariantCreatingAttributeFunc(FuncVariant),
    #[error("cannot bind func to different prop kinds")]
    FuncDestinationPropKindMismatch,
    #[error("Function execution failed: {0}")]
    FuncExecutionFailed(String),
    #[error("Function execution failed: this function is not connected to any assets, and was not executed")]
    FuncExecutionFailedNoPrototypes,
    #[error("Could not find schema variant for prop {0}")]
    SchemaVariantNotFoundForProp(PropId),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

// Variants don't map 1:1 onto FuncBackendKind, since some JsAttribute functions
// are a special case (Qualification, CodeGeneration etc)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FuncVariant {
    Attribute,
    CodeGeneration,
    Confirmation,
    Command,
    Qualification,
    Validation,
}

impl From<FuncVariant> for FuncBackendKind {
    fn from(value: FuncVariant) -> Self {
        match value {
            FuncVariant::Command => FuncBackendKind::JsCommand,
            FuncVariant::Validation => FuncBackendKind::Validation,
            FuncVariant::Attribute
            | FuncVariant::CodeGeneration
            | FuncVariant::Confirmation
            | FuncVariant::Qualification => FuncBackendKind::JsAttribute,
        }
    }
}

impl TryFrom<&Func> for FuncVariant {
    type Error = FuncError;

    fn try_from(func: &Func) -> Result<Self, Self::Error> {
        match (func.backend_kind(), func.backend_response_type()) {
            (FuncBackendKind::JsAttribute, response_type) => match response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncVariant::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncVariant::Qualification),
                FuncBackendResponseType::Confirmation => Ok(FuncVariant::Confirmation),
                _ => Ok(FuncVariant::Attribute),
            },
            (FuncBackendKind::JsCommand, _) => Ok(FuncVariant::Command),
            (FuncBackendKind::JsValidation, _) => Ok(FuncVariant::Validation),
            _ => Err(FuncError::FuncCannotBeTurnedIntoVariant(*func.id())),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentView {
    func_argument_id: FuncArgumentId,
    id: Option<AttributePrototypeArgumentId>,
    internal_provider_id: Option<InternalProviderId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    id: AttributePrototypeId,
    component_id: Option<ComponentId>,
    prop_id: PropId,
    prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub fn to_attribute_context(&self) -> FuncResult<AttributeContext> {
        Ok(match self.component_id {
            // Attribute contexts which set the defaults for a schema variant, have *only* the prop
            // set on the context
            None | Some(ComponentId::NONE) => AttributeContextBuilder::new()
                .set_prop_id(self.prop_id)
                .to_context()?,
            Some(component_id) => AttributeContextBuilder::new()
                .set_prop_id(self.prop_id)
                .set_component_id(component_id)
                .to_context()?,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationPrototypeView {
    id: ValidationPrototypeId,
    schema_variant_id: SchemaVariantId,
    prop_id: PropId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    #[serde(rename_all = "camelCase")]
    Qualification {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    CodeGeneration {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    Attribute {
        prototypes: Vec<AttributePrototypeView>,
        arguments: Vec<FuncArgumentView>,
    },
    #[serde(rename_all = "camelCase")]
    Confirmation {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    Validation {
        prototypes: Vec<ValidationPrototypeView>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentView {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}

async fn is_func_revertible(ctx: &DalContext, func: &Func) -> FuncResult<bool> {
    // refetch to get updated visibility
    let is_in_change_set = match Func::get_by_id(ctx, func.id()).await? {
        Some(func) => func.visibility().in_change_set(),
        None => return Ok(false),
    };
    // Clone a new ctx vith head visibility
    let ctx = ctx.clone_with_head();
    let head_func = Func::get_by_id(&ctx, func.id()).await?;

    Ok(head_func.is_some() && is_in_change_set)
}

async fn prototype_view_for_attribute_prototype(
    ctx: &DalContext,
    func_id: FuncId,
    proto: &AttributePrototype,
) -> FuncResult<AttributePrototypeView> {
    let prop_id = if proto.context.prop_id().is_some() {
        proto.context.prop_id()
    } else {
        return Err(FuncError::AttributePrototypeMissingPropId(*proto.id()));
    };

    let component_id = if proto.context.component_id().is_some() {
        Some(proto.context.component_id())
    } else {
        None
    };

    let prototype_arguments =
        FuncArgument::list_for_func_with_prototype_arguments(ctx, func_id, *proto.id())
            .await?
            .iter()
            .map(
                |(func_arg, maybe_proto_arg)| AttributePrototypeArgumentView {
                    func_argument_id: *func_arg.id(),
                    id: maybe_proto_arg.as_ref().map(|proto_arg| *proto_arg.id()),
                    internal_provider_id: maybe_proto_arg
                        .as_ref()
                        .map(|proto_arg| proto_arg.internal_provider_id()),
                },
            )
            .collect();

    Ok(AttributePrototypeView {
        id: *proto.id(),
        prop_id,
        component_id,
        prototype_arguments,
    })
}

async fn attribute_prototypes_into_schema_variants_and_components(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<(Vec<SchemaVariantId>, Vec<ComponentId>)> {
    let schema_variants_components =
        AttributePrototype::find_for_func_as_variant_and_component(ctx, func_id).await?;

    let mut schema_variant_ids = vec![];
    let mut component_ids = vec![];

    for (schema_variant_id, component_id) in schema_variants_components {
        if component_id == ComponentId::NONE {
            schema_variant_ids.push(schema_variant_id);
        } else {
            component_ids.push(component_id);
        }
    }

    Ok((schema_variant_ids, component_ids))
}

pub async fn get_func_view(ctx: &DalContext, func: &Func) -> FuncResult<GetFuncResponse> {
    let arguments = FuncArgument::list_for_func(ctx, *func.id()).await?;

    let (associations, input_type) = match func.backend_kind() {
        FuncBackendKind::JsAttribute => {
            let input_type = compile_argument_types(&arguments);

            let associations = match func.backend_response_type() {
                FuncBackendResponseType::CodeGeneration => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    Some(FuncAssociations::CodeGeneration {
                        schema_variant_ids,
                        component_ids,
                    })
                }
                FuncBackendResponseType::Confirmation => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    Some(FuncAssociations::Confirmation {
                        schema_variant_ids,
                        component_ids,
                    })
                }
                FuncBackendResponseType::Qualification => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    Some(FuncAssociations::Qualification {
                        schema_variant_ids,
                        component_ids,
                    })
                }
                _ => {
                    let protos = AttributePrototype::find_for_func(ctx, func.id()).await?;

                    let mut prototypes = Vec::with_capacity(protos.len());
                    for proto in &protos {
                        prototypes.push(
                            prototype_view_for_attribute_prototype(ctx, *func.id(), proto).await?,
                        );
                    }

                    Some(FuncAssociations::Attribute {
                        prototypes,
                        arguments: arguments
                            .iter()
                            .map(|arg| FuncArgumentView {
                                id: *arg.id(),
                                name: arg.name().to_owned(),
                                kind: arg.kind().to_owned(),
                                element_kind: arg.element_kind().cloned(),
                            })
                            .collect(),
                    })
                }
            };
            (associations, input_type)
        }
        FuncBackendKind::JsCommand => {
            let input_type = compile_command_types();
            (None, input_type)
        }
        FuncBackendKind::JsValidation => {
            let protos = ValidationPrototype::list_for_func(ctx, *func.id()).await?;
            let input_type = compile_validation_types(ctx, &protos).await?;

            let associations = Some(FuncAssociations::Validation {
                prototypes: protos
                    .iter()
                    .map(|proto| ValidationPrototypeView {
                        id: *proto.id(),
                        schema_variant_id: proto.context().schema_variant_id(),
                        prop_id: proto.context().prop_id(),
                    })
                    .collect(),
            });
            (associations, input_type)
        }

        _ => (None, String::new()),
    };

    let is_revertible = is_func_revertible(ctx, func).await?;
    let types = [
        compile_return_types(*func.backend_response_type()),
        &input_type,
        langjs_types(),
    ]
    .join("\n");

    Ok(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        variant: func.try_into()?,
        name: func
            .display_name()
            .unwrap_or_else(|| func.name())
            .to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.builtin(),
        is_revertible,
        associations,
        types,
    })
}

// TODO FIXME(paulo): cleanup code repetition

fn compile_return_types(ty: FuncBackendResponseType) -> &'static str {
    // TODO: avoid any, follow prop graph and build actual type
    // TODO: Could be generated automatically from some rust types, but which?
    match ty {
        FuncBackendResponseType::Boolean => "type Output = boolean | null;",
        FuncBackendResponseType::String => "type Output = string | null;",
        FuncBackendResponseType::Integer => "type Output = number | null;",
        FuncBackendResponseType::Qualification => "interface Output {
  result: 'success' | 'warning' | 'failure';
  message?: string;
}",
        FuncBackendResponseType::Confirmation => "interface Output {
  success: boolean;
  recommendedActions: string[];
}",
        FuncBackendResponseType::CodeGeneration => "interface Output {
  format: string;
  code: string;
}",
        FuncBackendResponseType::Validation => "interface Output {
  valid: boolean;
  message: string;
}",
        // Actual Workflow Kinds are 'conditional' | 'exceptional' | 'parallel'
        // But we don't actually properly use/support/test the other ones
        // There i
        FuncBackendResponseType::Workflow => "interface Output {
  name: string;
  kind: 'conditional';
  steps: Array<{ workflow: string; args: unknown | null; } | { command: string; args: unknown | null; }>;
}",
        FuncBackendResponseType::Command => "interface Output {
    status: 'ok' | 'warning' | 'error';
    payload?: { [key: string]: unknown } | null;
    message?: string;
}",
        FuncBackendResponseType::Json => "type Output = any;",
        // Note: there is no ts function returning those
        FuncBackendResponseType::Identity => "interface Output extends Input {}",
        FuncBackendResponseType::Array => "type Output = any[];",
        FuncBackendResponseType::Map => "type Output = any;",
        FuncBackendResponseType::Object => "type Output = any;",
        FuncBackendResponseType::Unset => "type Output = undefined | null;",
    }
}

async fn compile_validation_types(
    ctx: &DalContext,
    prototypes: &[ValidationPrototype],
) -> FuncResult<String> {
    let mut input_fields = Vec::new();
    // TODO: avoid any, follow prop graph and build actual type
    for prototype in prototypes {
        let prop = Prop::get_by_id(ctx, &prototype.context().prop_id())
            .await?
            .ok_or(PropError::NotFound(
                prototype.context().prop_id(),
                *ctx.visibility(),
            ))?;
        let ty = match prop.kind() {
            PropKind::Boolean => "boolean",
            PropKind::Integer => "number",
            PropKind::String => "string",
            PropKind::Array => "any[]",
            PropKind::Object => "any",
            PropKind::Map => "any",
        };
        input_fields.push(ty);
    }
    if input_fields.is_empty() {
        Ok("type Input = never;".to_owned())
    } else {
        let variants = input_fields.join(" | ");
        let types = format!("type Input = {variants};");
        Ok(types)
    }
}

fn compile_argument_types(arguments: &[FuncArgument]) -> String {
    let mut input_fields = HashMap::new();
    // TODO: avoid any, follow prop graph and build actual type
    for argument in arguments {
        let ty = match argument.kind() {
            FuncArgumentKind::Boolean => "boolean",
            FuncArgumentKind::Integer => "number",
            FuncArgumentKind::String => "string",
            FuncArgumentKind::Array => "any[]",
            FuncArgumentKind::Object => "any",
            FuncArgumentKind::Map => "any",
            FuncArgumentKind::Any => "any",
        };
        input_fields.insert(argument.name().to_owned(), ty);
    }
    let mut types = "interface Input {\n".to_owned();
    for (name, ty) in &input_fields {
        types.push_str(&format!("  {name}: {ty} | null;\n"));
    }
    types.push('}');
    types
}

// TODO FIXME(paulo): arguments for command functions are provided through a js function, so we can't predict this, we should fix it so the types are predictable
// Right now all workflow functions are builtins and the user can't create new workflow functions, so we can trust that they all are providing the same argument
//
// TODO: build properties types from prop tree
// Note: ComponentKind::Credential is unused and the implementation is broken, so let's ignore it for now
fn compile_command_types() -> String {
    "interface Input {
    kind: 'standard';
    properties: any;
}"
    .to_owned()
}

// TODO: stop duplicating definition
// TODO: use execa types instead of any
// TODO: add os, fs and path types (possibly fetch but I think it comes with DOM)
fn langjs_types() -> &'static str {
    "declare namespace YAML {
    function stringify(obj: unknown): string;
}
    declare namespace siExec {
    async function waitUntilEnd(execaFile: string, execaArgs?: string[], execaOptions?: any): Promise<any>;
}"
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
        .route("/save_and_exec", post(save_and_exec::save_and_exec))
        .route("/revert_func", post(revert_func::revert_func))
        .route(
            "/list_input_sources",
            get(list_input_sources::list_input_sources),
        )
}
