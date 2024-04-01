use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::authentication_prototype::AuthenticationPrototypeError;
use dal::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind};
use dal::schema::variant::SchemaVariantError;
use dal::{
    workspace_snapshot::WorkspaceSnapshotError, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncId, SchemaVariantId, TransactionsError,
};
use dal::{ChangeSetError, WsEventError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::{impl_default_error_into_response, state::AppState};
use crate::service::func::get_func::GetFuncResponse;

pub mod create_func;
pub mod get_func;
pub mod list_funcs;
pub mod save_func;

// pub mod delete_func;
// pub mod execute;
// pub mod list_input_sources;
// pub mod revert_func;
// pub mod save_and_exec;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    //     #[error("action func {0} assigned to multiple kinds")]
    //     ActionFuncMultipleKinds(FuncId),
    //     #[error("action kind missing on prototypes for action func {0}")]
    //     ActionKindMissing(FuncId),
    //     #[error(transparent)]
    //     ActionPrototype(#[from] ActionPrototypeError),
    //     #[error("attribute context error: {0}")]
    //     AttributeContext(#[from] AttributeContextError),
    //     #[error("attribute context builder error: {0}")]
    //     AttributeContextBuilder(#[from] AttributeContextBuilderError),
    //     #[error("attribute prototype error: {0}")]
    //     AttributePrototype(#[from] AttributePrototypeError),
    //     #[error("That attribute is already set by the function named \"{0}\"")]
    //     AttributePrototypeAlreadySetByFunc(String),
    //     #[error("attribute prototype argument error: {0}")]
    //     AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    //     #[error("attribute prototype missing")]
    //     AttributePrototypeMissing,
    //     #[error("attribute prototype {0} is missing argument {1}")]
    //     AttributePrototypeMissingArgument(AttributePrototypeId, AttributePrototypeArgumentId),
    //     #[error("attribute prototype {0} is missing its prop {1}")]
    //     AttributePrototypeMissingProp(AttributePrototypeId, PropId),
    //     #[error("attribute prototype {0} schema is missing")]
    //     AttributePrototypeMissingSchema(AttributePrototypeId),
    //     #[error("attribute prototype {0} schema_variant is missing")]
    //     AttributePrototypeMissingSchemaVariant(AttributePrototypeId),
    //     #[error("attribute value error: {0}")]
    //     AttributeValue(#[from] AttributeValueError),
    #[error("authentication prototype error: {0}")]
    AuthenticationPrototype(#[from] AuthenticationPrototypeError),
    //     #[error("attribute value missing")]
    //     AttributeValueMissing,
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    //     #[error("component error: {0}")]
    //     Component(#[from] ComponentError),
    //     #[error("component missing schema variant")]
    //     ComponentMissingSchemaVariant(ComponentId),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    //     #[error("editing reconciliation functions is not implemented")]
    //     EditingReconciliationFuncsNotImplemented,
    #[error(transparent)]
    Func(#[from] dal::func::FuncError),
    //     #[error("func argument not found")]
    //     FuncArgNotFound,
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    //     #[error("func argument already exists for that name")]
    //     FuncArgumentAlreadyExists,
    //     #[error("func argument {0} missing attribute prototype argument for prototype {1}")]
    //     FuncArgumentMissingPrototypeArgument(FuncArgumentId, AttributePrototypeId),
    //     #[error("func binding error: {0}")]
    //     FuncBinding(#[from] FuncBindingError),
    //     #[error("func binding return value error: {0}")]
    //     FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    //     #[error("func binding return value not found")]
    //     FuncBindingReturnValueMissing,
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    //     // XXX: we will be able to remove this error once we make output sockets typed
    //     #[error("Cannot bind function to both an output socket and a prop")]
    //     FuncDestinationPropAndOutputSocket,
    //     #[error("cannot bind func to different prop kinds")]
    //     FuncDestinationPropKindMismatch,
    //     #[error("Function execution: {0}")]
    //     FuncExecution(#[from] FuncExecutionError),
    //     #[error("Function execution failed: {0}")]
    //     FuncExecutionFailed(String),
    //     #[error("Function execution failed: this function is not connected to any assets, and was not executed")]
    //     FuncExecutionFailedNoPrototypes,
    //     #[error("Function still has associations: {0}")]
    //     FuncHasAssociations(FuncId),
    #[error("Function named \"{0}\" already exists in this changeset")]
    FuncNameExists(String),
    #[error("The function name \"{0}\" is reserved")]
    FuncNameReserved(String),
    //     #[error("func is not revertible")]
    //     FuncNotRevertible,
    //     #[error("Cannot create that type of function")]
    //     FuncNotSupported,
    //     #[error("Function options are incompatible with variant")]
    //     FuncOptionsAndVariantMismatch,
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    //     #[error("failed to join async task; bug!")]
    //     Join(#[from] JoinError),
    //     #[error("Missing required options for creating a function")]
    //     MissingOptions,
    #[error("Function is read-only")]
    NotWritable,
    //     #[error(transparent)]
    //     Pg(#[from] si_data_pg::PgError),
    // #[error(transparent)]
    // PgPool(#[from] Box<si_data_pg::PgPoolError>),
    //     #[error("prop error: {0}")]
    //     Prop(#[from] PropError),
    #[error("prop for value not found")]
    PropNotFound,
    //     #[error("prop tree error: {0}")]
    //     PropTree(#[from] PropTreeError),
    //     #[error("prototype context error: {0}")]self
    //     PrototypeContext(#[from] PrototypeContextError),
    //     #[error("prototype list for func error: {0}")]
    //     PrototypeListForFunc(#[from] PrototypeListForFuncError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    //     #[error("schema variant missing schema")]
    //     SchemaVariantMissingSchema(SchemaVariantId),
    //     #[error("Could not find schema variant for prop {0}")]
    //     SchemaVariantNotFoundForProp(PropId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    //     StandardModel(#[from] StandardModelError),
    //     #[error("tenancy error: {0}")]
    //     Tenancy(#[from] TenancyError),
    #[error("unexpected func variant ({0:?}) creating attribute func")]
    UnexpectedFuncVariantCreatingAttributeFunc(FuncVariant),
    //     #[error("A validation already exists for that attribute")]
    //     ValidationAlreadyExists,
    //     #[error("validation prototype error: {0}")]
    //     ValidationPrototype(#[from] ValidationPrototypeError),
    //     #[error("validation prototype schema is missing")]
    //     ValidationPrototypeMissingSchema,
    //     #[error("validation prototype {0} schema_variant is missing")]
    //     ValidationPrototypeMissingSchemaVariant(SchemaVariantId),
    #[error(transparent)]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

//impl From<si_data_pg::PgPoolError> for FuncError {
//    fn from(value: si_data_pg::PgPoolError) -> Self {
//        Self::PgPool(Box::new(value))
//    }
//}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

// Variants don't map 1:1 onto FuncBackendKind, since some JsAttribute functions
// are a special case (Qualification, CodeGeneration etc)
#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum FuncVariant {
    Action,
    Attribute,
    Authentication,
    CodeGeneration,
    Qualification,
    Reconciliation,
    Validation,
}

impl From<FuncVariant> for FuncBackendKind {
    fn from(value: FuncVariant) -> Self {
        match value {
            FuncVariant::Reconciliation => FuncBackendKind::JsReconciliation,
            FuncVariant::Action => FuncBackendKind::JsAction,
            FuncVariant::Validation => FuncBackendKind::JsValidation,
            FuncVariant::Attribute | FuncVariant::CodeGeneration | FuncVariant::Qualification => {
                FuncBackendKind::JsAttribute
            }
            FuncVariant::Authentication => FuncBackendKind::JsAuthentication,
        }
    }
}

impl TryFrom<&Func> for FuncVariant {
    type Error = FuncError;

    fn try_from(func: &Func) -> Result<Self, Self::Error> {
        match (func.backend_kind, func.backend_response_type) {
            (FuncBackendKind::JsAttribute, response_type) => match response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncVariant::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncVariant::Qualification),
                _ => Ok(FuncVariant::Attribute),
            },
            (FuncBackendKind::JsReconciliation, _) => Ok(FuncVariant::Reconciliation),
            (FuncBackendKind::JsAction, _) => Ok(FuncVariant::Action),
            (FuncBackendKind::JsValidation, _) => Ok(FuncVariant::Validation),
            (FuncBackendKind::JsAuthentication, _) => Ok(FuncVariant::Authentication),
            (FuncBackendKind::Array, _)
            | (FuncBackendKind::Boolean, _)
            | (FuncBackendKind::Diff, _)
            | (FuncBackendKind::Identity, _)
            | (FuncBackendKind::Integer, _)
            | (FuncBackendKind::JsSchemaVariantDefinition, _)
            | (FuncBackendKind::Map, _)
            | (FuncBackendKind::Object, _)
            | (FuncBackendKind::String, _)
            | (FuncBackendKind::Unset, _)
            | (FuncBackendKind::Validation, _) => {
                Err(FuncError::FuncCannotBeTurnedIntoVariant(func.id))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {}

// #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct ValidationPrototypeView {
//     schema_variant_id: SchemaVariantId,
//     prop_id: PropId,
// }
//
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    //     #[serde(rename_all = "camelCase")]
    //     Action {
    //         schema_variant_ids: Vec<SchemaVariantId>,
    //         kind: Option<ActionKind>,
    //     },
    #[serde(rename_all = "camelCase")]
    Attribute {
        prototypes: Vec<AttributePrototypeView>,
        arguments: Vec<FuncArgumentView>,
    },
    #[serde(rename_all = "camelCase")]
    Authentication {
        schema_variant_ids: Vec<SchemaVariantId>,
    },
    //     #[serde(rename_all = "camelCase")]
    //     CodeGeneration {
    //         schema_variant_ids: Vec<SchemaVariantId>,
    //         component_ids: Vec<ComponentId>,
    //         inputs: Vec<LeafInputLocation>,
    //     },
    //     #[serde(rename_all = "camelCase")]
    //     Qualification {
    //         schema_variant_ids: Vec<SchemaVariantId>,
    //         component_ids: Vec<ComponentId>,
    //         inputs: Vec<LeafInputLocation>,
    //     },
    //     #[serde(rename_all = "camelCase")]
    //     SchemaVariantDefinitions {
    //         schema_variant_ids: Vec<SchemaVariantId>,
    //     },
    //     #[serde(rename_all = "camelCase")]
    //     Validation {
    //         prototypes: Vec<ValidationPrototypeView>,
    //     },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentView {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}
//
// async fn is_func_revertible(ctx: &DalContext, func: &Func) -> FuncResult<bool> {
//     // refetch to get updated visibility
//     let is_in_change_set = match Func::get_by_id(ctx, func.id()).await? {
//         Some(func) => func.visibility().in_change_set(),
//         None => return Ok(false),
//     };
//     // Clone a new ctx vith head visibility
//     let ctx = ctx.clone_with_head();
//     let head_func = Func::get_by_id(&ctx, func.id()).await?;

//     Ok(head_func.is_some() && is_in_change_set)
// }

// async fn action_prototypes_into_schema_variants_and_components(
//     ctx: &DalContext,
//     func_id: FuncId,
// ) -> FuncResult<(Option<ActionKind>, Vec<SchemaVariantId>)> {
//     let mut variant_ids = vec![];
//     let mut action_kind: Option<ActionKind> = None;

//     for proto in ActionPrototype::find_for_func(ctx, func_id).await? {
//         if let Some(action_kind) = &action_kind {
//             if action_kind != proto.kind() {
//                 return Err(FuncError::ActionFuncMultipleKinds(func_id));
//             }
//         } else {
//             action_kind = Some(*proto.kind());
//         }

//         if proto.schema_variant_id().is_some() {
//             variant_ids.push(proto.schema_variant_id());
//         }
//     }

//     if !variant_ids.is_empty() && action_kind.is_none() {
//         return Err(FuncError::ActionKindMissing(func_id));
//     }

//     Ok((action_kind, variant_ids))
// }

// async fn attribute_prototypes_into_schema_variants_and_components(
//     ctx: &DalContext,
//     func_id: FuncId,
// ) -> FuncResult<(Vec<SchemaVariantId>, Vec<ComponentId>)> {
//     let schema_variants_components =
//         AttributePrototype::find_for_func_as_variant_and_component(ctx, func_id).await?;

//     let mut schema_variant_ids = vec![];
//     let mut component_ids = vec![];

//     for (schema_variant_id, component_id) in schema_variants_components {
//         if component_id == ComponentId::NONE {
//             schema_variant_ids.push(schema_variant_id);
//         } else {
//             component_ids.push(component_id);
//         }
//     }

//     Ok((schema_variant_ids, component_ids))
// }

// pub async fn get_leaf_function_inputs(
//     ctx: &DalContext,
//     func_id: FuncId,
// ) -> FuncResult<Vec<LeafInputLocation>> {
//     Ok(FuncArgument::list_for_func(ctx, func_id)
//         .await?
//         .iter()
//         .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(arg.name()))
//         .collect())
// }
//
pub async fn get_func_view(ctx: &DalContext, func: &Func) -> FuncResult<GetFuncResponse> {
    let arguments = FuncArgument::list_for_func(ctx, func.id).await?;

    let (associations, input_type) = match &func.backend_kind {
        FuncBackendKind::JsAttribute => {
            let (associations, input_type) = match &func.backend_response_type {
                FuncBackendResponseType::CodeGeneration
                | FuncBackendResponseType::Qualification => {
                    (None, "".into())
                    //                         let (schema_variant_ids, component_ids) =
                    //                             attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                    //                                 .await?;
                    //
                    //                         let inputs = get_leaf_function_inputs(ctx, *func.id()).await?;
                    //                         let input_type =
                    //                             compile_leaf_function_input_types(ctx, &schema_variant_ids, &inputs)
                    //                                 .await?;
                    //
                    //                         (
                    //                             Some(match func.backend_response_type() {
                    //                                 FuncBackendResponseType::CodeGeneration => {
                    //                                     FuncAssociations::CodeGeneration {
                    //                                         schema_variant_ids,
                    //                                         component_ids,
                    //                                         inputs,
                    //                                     }
                    //                                 }
                    //
                    //                                 FuncBackendResponseType::Qualification => {
                    //                                     FuncAssociations::Qualification {
                    //                                         schema_variant_ids,
                    //                                         component_ids,
                    //                                         inputs: get_leaf_function_inputs(ctx, *func.id()).await?,
                    //                                     }
                    //                                 }
                    //                                 _ => unreachable!("the match above ensures this is unreachable"),
                    //                             }),
                    //                             input_type,
                    //                         )
                }
                _ => {
                    // let protos = AttributePrototype::find_for_func(ctx, func.id()).await?;

                    //                         let mut prototypes = Vec::with_capacity(protos.len());
                    //                         for proto in &protos {
                    //                             prototypes.push(
                    //                                 prototype_view_for_attribute_prototype(ctx, *func.id(), proto).await?,
                    //                             );
                    //                         }

                    // let ts_types = compile_attribute_function_types(ctx, &prototypes).await?;

                    (
                        Some(FuncAssociations::Attribute {
                            prototypes: vec![],
                            arguments: arguments
                                .iter()
                                .map(|arg| FuncArgumentView {
                                    id: arg.id,
                                    name: arg.name.to_owned(),
                                    kind: arg.kind,
                                    element_kind: arg.element_kind.to_owned(),
                                })
                                .collect(),
                        }),
                        "type Input = any".into(),
                    )
                }
            };
            (associations, input_type)
        }
        //         FuncBackendKind::JsAction => {
        //             let (kind, schema_variant_ids) =
        //                 action_prototypes_into_schema_variants_and_components(ctx, *func.id()).await?;
        //
        //             let ts_types = compile_action_types(ctx, &schema_variant_ids).await?;
        //
        //             let associations = Some(FuncAssociations::Action {
        //                 schema_variant_ids,
        //                 kind,
        //             });
        //
        //             (associations, ts_types)
        //         }
        //         FuncBackendKind::JsReconciliation => {
        //             return Err(FuncError::EditingReconciliationFuncsNotImplemented);
        //         }
        //         FuncBackendKind::JsValidation => {
        //             let protos = ValidationPrototype::list_for_func(ctx, *func.id()).await?;
        //             let input_type = compile_validation_types(ctx, &protos).await?;
        //
        //             let associations = Some(FuncAssociations::Validation {
        //                 prototypes: protos
        //                     .iter()
        //                     .map(|proto| ValidationPrototypeView {
        //                         schema_variant_id: proto.context().schema_variant_id(),
        //                         prop_id: proto.context().prop_id(),
        //                     })
        //                     .collect(),
        //             });
        //             (associations, input_type)
        //         }
        FuncBackendKind::JsAuthentication => {
            let schema_variant_ids = Func::list_schema_variants_for_auth_func(ctx, func.id).await?;

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
    //
    //     let is_revertible = is_func_revertible(ctx, func).await?;
    let types = [
        compile_return_types(func.backend_response_type, func.backend_kind),
        &input_type,
        langjs_types(),
    ]
    .join("\n");

    Ok(GetFuncResponse {
        id: func.id.to_owned(),
        variant: func.try_into()?,
        display_name: func.display_name.as_ref().map(Into::into),
        name: func.name.to_owned(),
        description: func.description.as_ref().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.builtin,
        is_revertible: false,
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
            include_str!("./ts_types/asset_types_with_secrets.d.ts"),
            "\n",
            include_str!("./ts_types/joi.d.ts"),
            "\n",
            "type Output = any;"
        ),
    }
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
        //         .route(
        //             "/get_func_last_execution",
        //             get(get_func::get_latest_func_execution),
        //         )
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
    //         .route("/delete_func", post(delete_func::delete_func))
    //         .route("/save_and_exec", post(save_and_exec::save_and_exec))
    //         .route("/execute", post(execute::execute))
    //         .route("/revert_func", post(revert_func::revert_func))
    //         .route(
    //             "/list_input_sources",
    //             get(list_input_sources::list_input_sources),
    //         )
}
