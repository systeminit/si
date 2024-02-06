use std::collections::HashMap;

use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinError;

use dal::authentication_prototype::{AuthenticationPrototype, AuthenticationPrototypeError};
use dal::func::execution::FuncExecutionError;
use dal::{
    attribute::context::{AttributeContextBuilder, AttributeContextBuilderError},
    func::{
        argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind},
        binding_return_value::FuncBindingReturnValueError,
    },
    prop_tree::PropTreeError,
    prototype_context::PrototypeContextError,
    schema::variant::SchemaVariantError,
    ActionKind, ActionPrototype, ActionPrototypeError, AttributeContext, AttributeContextError,
    AttributePrototype, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
    AttributePrototypeError, AttributePrototypeId, AttributeValueError, ChangeSetError,
    ComponentError, ComponentId, DalContext, ExternalProviderError, ExternalProviderId, Func,
    FuncBackendKind, FuncBackendResponseType, FuncBindingError, FuncId, FuncVariant,
    InternalProvider, InternalProviderError, InternalProviderId, LeafInputLocation, Prop,
    PropError, PropId, PrototypeListForFuncError, SchemaVariant, SchemaVariantId, StandardModel,
    StandardModelError, TenancyError, TransactionsError, WsEventError,
};

use crate::server::{impl_default_error_into_response, state::AppState};
use crate::service::func::get_func::GetFuncResponse;

pub mod create_func;
pub mod delete_func;
pub mod execute;
pub mod get_func;
pub mod list_funcs;
pub mod list_input_sources;
pub mod revert_func;
pub mod save_and_exec;
pub mod save_func;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("action func {0} assigned to multiple kinds")]
    ActionFuncMultipleKinds(FuncId),
    #[error("action kind missing on prototypes for action func {0}")]
    ActionKindMissing(FuncId),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("That attribute is already set by the function named \"{0}\"")]
    AttributePrototypeAlreadySetByFunc(String),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype missing")]
    AttributePrototypeMissing,
    #[error("attribute prototype {0} is missing argument {1}")]
    AttributePrototypeMissingArgument(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("attribute prototype argument {0} is internal provider id")]
    AttributePrototypeMissingInternalProviderId(AttributePrototypeArgumentId),
    #[error("attribute prototype {0} is missing its prop {1}")]
    AttributePrototypeMissingProp(AttributePrototypeId, PropId),
    #[error("attribute prototype {0} has no PropId or ExternalProviderId")]
    AttributePrototypeMissingPropIdOrExternalProviderId(AttributePrototypeId),
    #[error("attribute prototype {0} schema is missing")]
    AttributePrototypeMissingSchema(AttributePrototypeId),
    #[error("attribute prototype {0} schema_variant is missing")]
    AttributePrototypeMissingSchemaVariant(AttributePrototypeId),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value missing")]
    AttributeValueMissing,
    #[error("authentication prototype error: {0}")]
    AuthenticationPrototypeError(#[from] AuthenticationPrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component missing schema variant")]
    ComponentMissingSchemaVariant(ComponentId),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("editing reconciliation functions is not implemented")]
    EditingReconciliationFuncsNotImplemented,
    #[error("editing validation functions is not implemented")]
    EditingValidationFuncsNotImplemented,
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("func argument not found")]
    FuncArgNotFound,
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument already exists for that name")]
    FuncArgumentAlreadyExists,
    #[error("func argument {0} missing attribute prototype argument for prototype {1}")]
    FuncArgumentMissingPrototypeArgument(FuncArgumentId, AttributePrototypeId),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func binding return value not found")]
    FuncBindingReturnValueMissing,
    // XXX: we will be able to remove this error once we make output sockets typed
    #[error("Cannot bind function to both an output socket and a prop")]
    FuncDestinationPropAndOutputSocket,
    #[error("cannot bind func to different prop kinds")]
    FuncDestinationPropKindMismatch,
    #[error("Function execution: {0}")]
    FuncExecution(#[from] FuncExecutionError),
    #[error("Function execution failed: {0}")]
    FuncExecutionFailed(String),
    #[error("Function execution failed: this function is not connected to any assets, and was not executed")]
    FuncExecutionFailedNoPrototypes,
    #[error("Function still has associations: {0}")]
    FuncHasAssociations(FuncId),
    #[error("Function named \"{0}\" already exists in this changeset")]
    FuncNameExists(String),
    #[error("The function name \"{0}\" is reserved")]
    FuncNameReserved(String),
    #[error("Function not found")]
    FuncNotFound,
    #[error("func is not revertible")]
    FuncNotRevertible,
    #[error("Function not runnable")]
    FuncNotRunnable,
    #[error("Cannot create that type of function")]
    FuncNotSupported,
    #[error("Function options are incompatible with variant")]
    FuncOptionsAndVariantMismatch,
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("failed to join async task; bug!")]
    Join(#[from] JoinError),
    #[error("Missing required options for creating a function")]
    MissingOptions,
    #[error("Function is read-only")]
    NotWritable,
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] Box<si_data_pg::PgPoolError>),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("prop for value not found")]
    PropNotFound,
    #[error("prop tree error: {0}")]
    PropTree(#[from] PropTreeError),
    #[error("prototype context error: {0}")]
    PrototypeContext(#[from] PrototypeContextError),
    #[error("prototype list for func error: {0}")]
    PrototypeListForFunc(#[from] PrototypeListForFuncError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant missing schema")]
    SchemaVariantMissingSchema(SchemaVariantId),
    #[error("Could not find schema variant for prop {0}")]
    SchemaVariantNotFoundForProp(PropId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error("unexpected func variant ({0:?}) creating attribute func")]
    UnexpectedFuncVariantCreatingAttributeFunc(FuncVariant),
    #[error("A validation already exists for that attribute")]
    ValidationAlreadyExists,
    #[error("validation prototype schema is missing")]
    ValidationPrototypeMissingSchema,
    #[error("validation prototype {0} schema_variant is missing")]
    ValidationPrototypeMissingSchemaVariant(SchemaVariantId),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

impl From<si_data_pg::PgPoolError> for FuncError {
    fn from(value: si_data_pg::PgPoolError) -> Self {
        Self::PgPool(Box::new(value))
    }
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentView {
    func_argument_id: FuncArgumentId,
    func_argument_name: Option<String>,
    id: Option<AttributePrototypeArgumentId>,
    internal_provider_id: Option<InternalProviderId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    id: AttributePrototypeId,
    component_id: Option<ComponentId>,
    prop_id: Option<PropId>,
    external_provider_id: Option<ExternalProviderId>,
    prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub fn to_attribute_context(&self) -> FuncResult<AttributeContext> {
        let mut builder = AttributeContextBuilder::new();
        if let Some(component_id) = self.component_id {
            builder.set_component_id(component_id);
        }
        if let Some(prop_id) = self.prop_id {
            builder.set_prop_id(prop_id);
        }
        if let Some(external_provider_id) = self.external_provider_id {
            builder.set_external_provider_id(external_provider_id);
        }

        Ok(builder.to_context()?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationPrototypeView {
    schema_variant_id: SchemaVariantId,
    prop_id: PropId,
}

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    #[serde(rename_all = "camelCase")]
    Action {
        schema_variant_ids: Vec<SchemaVariantId>,
        kind: Option<ActionKind>,
    },
    #[serde(rename_all = "camelCase")]
    Attribute {
        prototypes: Vec<AttributePrototypeView>,
        arguments: Vec<FuncArgumentView>,
    },
    #[serde(rename_all = "camelCase")]
    Authentication {
        schema_variant_ids: Vec<SchemaVariantId>,
    },
    #[serde(rename_all = "camelCase")]
    CodeGeneration {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    Qualification {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    SchemaVariantDefinitions {
        schema_variant_ids: Vec<SchemaVariantId>,
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
        Some(proto.context.prop_id())
    } else {
        None
    };

    let external_provider_id = if proto.context.external_provider_id().is_some() {
        Some(proto.context.external_provider_id())
    } else {
        None
    };

    if prop_id.is_none() && external_provider_id.is_none() {
        return Err(FuncError::AttributePrototypeMissingPropIdOrExternalProviderId(*proto.id()));
    }

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
                    func_argument_name: Some(func_arg.name().to_owned()),
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
        external_provider_id,
        prototype_arguments,
    })
}

async fn action_prototypes_into_schema_variants_and_components(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<(Option<ActionKind>, Vec<SchemaVariantId>)> {
    let mut variant_ids = vec![];
    let mut action_kind: Option<ActionKind> = None;

    for proto in ActionPrototype::find_for_func(ctx, func_id).await? {
        if let Some(action_kind) = &action_kind {
            if action_kind != proto.kind() {
                return Err(FuncError::ActionFuncMultipleKinds(func_id));
            }
        } else {
            action_kind = Some(*proto.kind());
        }

        if proto.schema_variant_id().is_some() {
            variant_ids.push(proto.schema_variant_id());
        }
    }

    if !variant_ids.is_empty() && action_kind.is_none() {
        return Err(FuncError::ActionKindMissing(func_id));
    }

    Ok((action_kind, variant_ids))
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

pub async fn get_leaf_function_inputs(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<Vec<LeafInputLocation>> {
    Ok(FuncArgument::list_for_func(ctx, func_id)
        .await?
        .iter()
        .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(arg.name()))
        .collect())
}

pub async fn get_func_view(ctx: &DalContext, func: &Func) -> FuncResult<GetFuncResponse> {
    let arguments = FuncArgument::list_for_func(ctx, *func.id()).await?;

    let (associations, input_type) = match func.backend_kind() {
        FuncBackendKind::JsAttribute => {
            let (associations, input_type) = match func.backend_response_type() {
                FuncBackendResponseType::CodeGeneration
                | FuncBackendResponseType::Qualification => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    let inputs = get_leaf_function_inputs(ctx, *func.id()).await?;
                    let input_type =
                        compile_leaf_function_input_types(ctx, &schema_variant_ids, &inputs)
                            .await?;

                    (
                        Some(match func.backend_response_type() {
                            FuncBackendResponseType::CodeGeneration => {
                                FuncAssociations::CodeGeneration {
                                    schema_variant_ids,
                                    component_ids,
                                    inputs,
                                }
                            }

                            FuncBackendResponseType::Qualification => {
                                FuncAssociations::Qualification {
                                    schema_variant_ids,
                                    component_ids,
                                    inputs: get_leaf_function_inputs(ctx, *func.id()).await?,
                                }
                            }
                            _ => unreachable!("the match above ensures this is unreachable"),
                        }),
                        input_type,
                    )
                }
                _ => {
                    let protos = AttributePrototype::find_for_func(ctx, func.id()).await?;

                    let mut prototypes = Vec::with_capacity(protos.len());
                    for proto in &protos {
                        prototypes.push(
                            prototype_view_for_attribute_prototype(ctx, *func.id(), proto).await?,
                        );
                    }

                    let ts_types = compile_attribute_function_types(ctx, &prototypes).await?;

                    (
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
                        }),
                        ts_types,
                    )
                }
            };
            (associations, input_type)
        }
        FuncBackendKind::JsAction => {
            let (kind, schema_variant_ids) =
                action_prototypes_into_schema_variants_and_components(ctx, *func.id()).await?;

            let ts_types = compile_action_types(ctx, &schema_variant_ids).await?;

            let associations = Some(FuncAssociations::Action {
                schema_variant_ids,
                kind,
            });

            (associations, ts_types)
        }
        FuncBackendKind::JsReconciliation => {
            return Err(FuncError::EditingReconciliationFuncsNotImplemented);
        }
        FuncBackendKind::JsValidation => {
            return Err(FuncError::EditingValidationFuncsNotImplemented);
        }
        FuncBackendKind::JsAuthentication => {
            let schema_variant_ids = AuthenticationPrototype::find_for_func(ctx, *func.id())
                .await?
                .iter()
                .map(|p| p.schema_variant_id())
                .collect();

            (
                Some(FuncAssociations::Authentication { schema_variant_ids }),
                concat!(
                    "type Input = Record<string, unknown>;\n",
                    "\n",
                    "declare namespace requestStorage {\n",
                    "    function setEnv(key: string, value: any);\n",
                    "    function setItem(key: string, value: any);\n",
                    "    function deleteEnv(key: string);\n",
                    "    function deleteItem(key: string);\n",
                    "}",
                )
                .to_owned(),
            )
        }
        _ => (None, String::new()),
    };

    let is_revertible = is_func_revertible(ctx, func).await?;
    let types = [
        compile_return_types(*func.backend_response_type(), *func.backend_kind()),
        &input_type,
        langjs_types(),
    ]
    .join("\n");

    Ok(GetFuncResponse {
        id: func.id().to_owned(),
        variant: func.try_into()?,
        display_name: func.display_name().map(Into::into),
        name: func.name().to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.builtin(),
        is_revertible,
        associations,
        types,
    })
}

pub fn compile_return_types(ty: FuncBackendResponseType, kind: FuncBackendKind) -> &'static str {
    if matches!(kind, FuncBackendKind::JsAttribute)
        && !matches!(
            ty,
            FuncBackendResponseType::CodeGeneration | FuncBackendResponseType::Qualification
        )
    {
        return ""; // attribute functions have their output compiled dynamically
    }

    match ty {
        FuncBackendResponseType::Boolean => "type Output = boolean | null;",
        FuncBackendResponseType::String => "type Output = string | null;",
        FuncBackendResponseType::Integer => "type Output = number | null;",
        FuncBackendResponseType::Qualification => {
            "type Output {
  result: 'success' | 'warning' | 'failure';
  message?: string | null;
}"
        }
        FuncBackendResponseType::CodeGeneration => {
            "type Output {
  format: string;
  code: string;
}"
        }
        FuncBackendResponseType::Validation => {
            "type Output {
  valid: boolean;
  message: string;
}"
        }
        FuncBackendResponseType::Reconciliation => {
            "type Output {
  updates: { [key: string]: unknown };
  actions: string[];
  message: string | null;
}"
        }
        FuncBackendResponseType::Action => {
            "type Output {
    status: 'ok' | 'warning' | 'error';
    payload?: { [key: string]: unknown } | null;
    message?: string | null;
}"
        }
        FuncBackendResponseType::Json => "type Output = any;",
        // Note: there is no ts function returning those
        FuncBackendResponseType::Identity => "interface Output extends Input {}",
        FuncBackendResponseType::Array => "type Output = any[];",
        FuncBackendResponseType::Map => "type Output = Record<string, any>;",
        FuncBackendResponseType::Object => "type Output = any;",
        FuncBackendResponseType::Unset => "type Output = undefined | null;",
        FuncBackendResponseType::Void => "type Output = void;",
        FuncBackendResponseType::SchemaVariantDefinition => concat!(
            include_str!("./ts_types/asset_builder.d.ts"),
            "\n",
            include_str!("./ts_types/joi.d.ts"),
            "\n",
            "type Output = any;"
        ),
    }
}

pub fn compile_return_types_2(ty: FuncBackendResponseType, kind: FuncBackendKind) -> &'static str {
    if matches!(kind, FuncBackendKind::JsAttribute)
        && !matches!(
            ty,
            FuncBackendResponseType::CodeGeneration | FuncBackendResponseType::Qualification
        )
    {
        return ""; // attribute functions have their output compiled dynamically
    }

    match ty {
        FuncBackendResponseType::Boolean => "type Output = boolean | null;",
        FuncBackendResponseType::String => "type Output = string | null;",
        FuncBackendResponseType::Integer => "type Output = number | null;",
        FuncBackendResponseType::Qualification => {
            "type Output {
  result: 'success' | 'warning' | 'failure';
  message?: string | null;
}"
        }
        FuncBackendResponseType::CodeGeneration => {
            "type Output {
  format: string;
  code: string;
}"
        }
        FuncBackendResponseType::Validation => {
            "type Output {
  valid: boolean;
  message: string;
}"
        }
        FuncBackendResponseType::Reconciliation => {
            "type Output {
  updates: { [key: string]: unknown };
  actions: string[];
  message: string | null;
}"
        }
        FuncBackendResponseType::Action => {
            "type Output {
    status: 'ok' | 'warning' | 'error';
    payload?: { [key: string]: unknown } | null;
    message?: string | null;
}"
        }
        FuncBackendResponseType::Json => "type Output = any;",
        // Note: there is no ts function returning those
        FuncBackendResponseType::Identity => "interface Output extends Input {}",
        FuncBackendResponseType::Array => "type Output = any[];",
        FuncBackendResponseType::Map => "type Output = Record<string, any>;",
        FuncBackendResponseType::Object => "type Output = any;",
        FuncBackendResponseType::Unset => "type Output = undefined | null;",
        FuncBackendResponseType::Void => "type Output = void;",
        FuncBackendResponseType::SchemaVariantDefinition => concat!(
            include_str!("./ts_types/asset_types_with_secrets.d.ts"),
            "\n",
            include_str!("./ts_types/joi.d.ts"),
            "\n",
            "type Output = any;"
        ),
    }
}

async fn get_per_variant_types_for_prop_path(
    ctx: &DalContext,
    variant_ids: &[SchemaVariantId],
    path: &[&str],
) -> FuncResult<String> {
    let mut per_variant_types = vec![];

    for variant_id in variant_ids {
        let prop = SchemaVariant::find_prop_in_tree(ctx, *variant_id, path).await?;
        let ts_type = prop.ts_type(ctx).await?;

        if !per_variant_types.contains(&ts_type) {
            per_variant_types.push(ts_type);
        }
    }

    Ok(per_variant_types.join(" | "))
}

async fn compile_leaf_function_input_types(
    ctx: &DalContext,
    schema_variant_ids: &[SchemaVariantId],
    inputs: &[LeafInputLocation],
) -> FuncResult<String> {
    let mut ts_type = "type Input = {\n".to_string();

    for input_location in inputs {
        let input_property = format!(
            "{}?: {} | null;\n",
            input_location.arg_name(),
            get_per_variant_types_for_prop_path(
                ctx,
                schema_variant_ids,
                &input_location.prop_path(),
            )
            .await?
        );
        ts_type.push_str(&input_property);
    }
    ts_type.push_str("};");

    Ok(ts_type)
}

async fn compile_attribute_function_types(
    ctx: &DalContext,
    prototype_views: &[AttributePrototypeView],
) -> FuncResult<String> {
    let mut input_ts_types = "type Input = {\n".to_string();

    let mut output_ts_types = vec![];
    let mut argument_types = HashMap::new();
    for prototype_view in prototype_views {
        for arg in &prototype_view.prototype_arguments {
            if let Some(ip_id) = arg.internal_provider_id {
                let ip = InternalProvider::get_by_id(ctx, &ip_id)
                    .await?
                    .ok_or(InternalProviderError::NotFound(ip_id))?;

                let ts_type = if ip.prop_id().is_none() {
                    "object".to_string()
                } else {
                    Prop::get_by_id(ctx, ip.prop_id())
                        .await?
                        .ok_or(PropError::NotFound(
                            *ip.prop_id(),
                            ctx.visibility().to_owned(),
                        ))?
                        .ts_type(ctx)
                        .await?
                };

                if !argument_types.contains_key(&arg.func_argument_name) {
                    argument_types.insert(arg.func_argument_name.clone(), vec![ts_type]);
                } else if let Some(ts_types_for_arg) =
                    argument_types.get_mut(&arg.func_argument_name)
                {
                    if !ts_types_for_arg.contains(&ts_type) {
                        ts_types_for_arg.push(ts_type)
                    }
                }
            }

            let output_type = if let Some(output_prop_id) = prototype_view.prop_id {
                Prop::get_by_id(ctx, &output_prop_id)
                    .await?
                    .ok_or(PropError::NotFound(
                        output_prop_id,
                        ctx.visibility().to_owned(),
                    ))?
                    .ts_type(ctx)
                    .await?
            } else {
                "any".to_string()
            };

            if !output_ts_types.contains(&output_type) {
                output_ts_types.push(output_type);
            }
        }
    }
    for (arg_name, ts_types) in argument_types.iter() {
        input_ts_types.push_str(
            format!(
                "{}?: {} | null;\n",
                arg_name.as_ref().unwrap_or(&"".to_string()).to_owned(),
                ts_types.join(" | ")
            )
            .as_str(),
        );
    }
    input_ts_types.push_str("};");

    let output_ts = format!("type Output = {};", output_ts_types.join(" | "));

    Ok(format!("{}\n{}", input_ts_types, output_ts))
}

// Note: ComponentKind::Credential is unused and the implementation is broken, so let's ignore it for now
async fn compile_action_types(
    ctx: &DalContext,
    variant_ids: &[SchemaVariantId],
) -> FuncResult<String> {
    let mut ts_types = vec![];
    for variant_id in variant_ids {
        let prop = SchemaVariant::find_prop_in_tree(ctx, *variant_id, &["root"]).await?;
        ts_types.push(prop.ts_type(ctx).await?);
    }

    Ok(format!(
        "type Input {{
    kind: 'standard';
    properties: {};
}}",
        ts_types.join(" | "),
    ))
}

// TODO: stop duplicating definition
// TODO: use execa types instead of any
// TODO: add os, fs and path types (possibly fetch but I think it comes with DOM)
fn langjs_types() -> &'static str {
    "declare namespace YAML {
    function stringify(obj: unknown): string;
}

    declare namespace zlib {
        function gzip(inputstr: string, callback: any);
    }

    declare namespace requestStorage {
        function getEnv(key: string): string;
        function getItem(key: string): any;
        function getEnvKeys(): string[];
        function getKeys(): string[];
    }

    declare namespace siExec {

    interface WatchArgs {
        cmd: string,
        args?: readonly string[],
        execaOptions?: Options<string>,
        retryMs?: number,
        maxRetryCount?: number,
        callback: (child: execa.ExecaReturnValue<string>) => Promise<boolean>,
    }

    interface WatchResult {
        result: SiExecResult,
        failed?: 'deadlineExceeded' | 'commandFailed',
    }

    type SiExecResult = ExecaReturnValue<string>;

    async function waitUntilEnd(execaFile: string, execaArgs?: string[], execaOptions?: any): Promise<any>;
    async function watch(options: WatchArgs, deadlineCount?: number): Promise<WatchResult>;
}"
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        .route(
            "/get_func_last_execution",
            get(get_func::get_latest_func_execution),
        )
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
        .route("/delete_func", post(delete_func::delete_func))
        .route("/save_and_exec", post(save_and_exec::save_and_exec))
        .route("/execute", post(execute::execute))
        .route("/revert_func", post(revert_func::revert_func))
        .route(
            "/list_input_sources",
            get(list_input_sources::list_input_sources),
        )
}
