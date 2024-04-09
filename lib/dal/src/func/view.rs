use serde::{Deserialize, Serialize};
use telemetry::prelude::debug;
use thiserror::Error;

use crate::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind};
use crate::func::associations::FuncAssociations;
use crate::func::authoring::FuncAuthoringClient;
use crate::func::FuncKind;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::{
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncError, FuncId,
    SchemaVariantError,
};

pub mod summary;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncViewError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

type FuncViewResult<T> = Result<T, FuncViewError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentView {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FuncView {
    pub id: FuncId,
    pub kind: FuncKind,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub code: Option<String>,
    pub types: String,
    pub is_builtin: bool,
    pub is_revertible: bool,
    pub associations: Option<FuncAssociations>,
}

impl FuncView {
    pub async fn assemble(ctx: &DalContext, func: &Func) -> FuncViewResult<Self> {
        let arguments = FuncArgument::list_for_func(ctx, func.id).await?;

        let (_associations, _input_type) = match func.kind {
            FuncKind::Action => (None::<FuncAssociations>, String::new()),
            FuncKind::Attribute => (None::<FuncAssociations>, String::new()),
            FuncKind::Authentication => (None::<FuncAssociations>, String::new()),
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                // let (schema_variant_ids, component_ids) =
                //     attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                //         .await?;
                //
                // let inputs = Self::list_leaf_function_inputs(ctx, *func.id()).await?;
                //
                // // TODO(nick): restore the ability to compile func input types.
                // let input_type = "".to_string();
                // // compile_leaf_function_input_types(ctx, &schema_variant_ids, &inputs).await?;
                //
                // (
                //     Some(match func.backend_response_type() {
                //         FuncBackendResponseType::CodeGeneration => {
                //             FuncAssociations::CodeGeneration {
                //                 schema_variant_ids,
                //                 component_ids,
                //                 inputs,
                //             }
                //         }
                //
                //         FuncBackendResponseType::Qualification => FuncAssociations::Qualification {
                //             schema_variant_ids,
                //             component_ids,
                //             inputs: Self::list_leaf_function_inputs(ctx, *func.id()).await?,
                //         },
                //         _ => unreachable!("the match above ensures this is unreachable"),
                //     }),
                //     input_type,
                // )
                (None::<FuncAssociations>, String::new())
            }
            FuncKind::Intrinsic | FuncKind::SchemaVariantDefinition | FuncKind::Unknown => {
                debug!(?func.kind, "no associations or input type needed for func kind");
                (None::<FuncAssociations>, String::new())
            }
        };

        let (associations, input_type) = match &func.backend_kind {
            FuncBackendKind::JsAttribute => {
                let (associations, input_type) = match &func.backend_response_type {
                    FuncBackendResponseType::CodeGeneration
                    | FuncBackendResponseType::Qualification => (None, "".into()),
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
                let schema_variant_ids =
                    Func::list_schema_variants_for_auth_func(ctx, func.id).await?;

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

        let types = [
            FuncAuthoringClient::compile_return_types(
                func.backend_response_type,
                func.backend_kind,
            ),
            &input_type,
            FuncAuthoringClient::compile_langjs_types(),
        ]
        .join("\n");

        Ok(Self {
            id: func.id.to_owned(),
            kind: func.kind,
            display_name: func.display_name.as_ref().map(Into::into),
            name: func.name.to_owned(),
            description: func.description.as_ref().map(|d| d.to_owned()),
            code: func.code_plaintext()?,
            is_builtin: func.builtin,
            is_revertible: func.is_revertible(ctx).await?,
            associations,
            types,
        })
    }

    #[allow(dead_code)]
    async fn list_leaf_function_inputs(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncViewResult<Vec<LeafInputLocation>> {
        Ok(FuncArgument::list_for_func(ctx, func_id)
            .await?
            .iter()
            .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(&arg.name))
            .collect())
    }
}
