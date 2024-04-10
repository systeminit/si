use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::view::FuncArgumentView;
use crate::func::FuncKind;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::{
    AttributePrototype, ComponentId, DalContext, Func, FuncBackendResponseType, FuncId,
    SchemaVariant, SchemaVariantError, SchemaVariantId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAssociationsError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

type FuncAssociationsResult<T> = Result<T, FuncAssociationsError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    // TODO(nick): populate this or delete it.
}

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    #[serde(rename_all = "camelCase")]
    Action {
        schema_variant_ids: Vec<SchemaVariantId>,
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
}

impl FuncAssociations {
    #[instrument(name = "func.associations.from_func", level = "debug", skip_all)]
    pub async fn from_func(
        ctx: &DalContext,
        func: &Func,
    ) -> FuncAssociationsResult<(Option<Self>, String)> {
        let arguments = FuncArgument::list_for_func(ctx, func.id).await?;

        let (associations, input_type) = match func.kind {
            FuncKind::Action => {
                let schema_variant_ids = SchemaVariant::list_for_action_func(ctx, func.id).await?;
                (
                    Some(Self::Action { schema_variant_ids }),
                    // TODO(nick): get input type.
                    String::new(),
                )
            }
            FuncKind::Attribute => {
                // TODO(nick): get prototype views and types
                (
                    Some(Self::Attribute {
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
            FuncKind::Authentication => {
                let schema_variant_ids = SchemaVariant::list_for_auth_func(ctx, func.id).await?;
                (
                    Some(Self::Authentication { schema_variant_ids }),
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
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                let attribute_prototype_ids =
                    AttributePrototype::list_ids_for_func_id(ctx, func.id).await?;

                let mut schema_variant_ids = Vec::new();
                let mut component_ids = Vec::new();

                for attribute_prototype_id in attribute_prototype_ids {
                    let (
                        schema_variant_ids_for_attribute_prototype,
                        component_ids_for_attribute_prototype,
                    ) = AttributePrototype::schema_variants_and_components(
                        ctx,
                        attribute_prototype_id,
                    )
                    .await?;
                    schema_variant_ids.extend(schema_variant_ids_for_attribute_prototype);
                    component_ids.extend(component_ids_for_attribute_prototype);
                }

                let inputs = Self::list_leaf_function_inputs(ctx, func.id).await?;

                // TODO(nick): restore the ability to compile func input types.
                let input_type = "".to_string();
                // compile_leaf_function_input_types(ctx, &schema_variant_ids, &inputs).await?;

                (
                    Some(match func.backend_response_type {
                        FuncBackendResponseType::CodeGeneration => Self::CodeGeneration {
                            schema_variant_ids,
                            component_ids,
                            inputs,
                        },

                        FuncBackendResponseType::Qualification => Self::Qualification {
                            schema_variant_ids,
                            component_ids,
                            inputs: Self::list_leaf_function_inputs(ctx, func.id).await?,
                        },
                        _ => unreachable!("the match above ensures this is unreachable"),
                    }),
                    input_type,
                )
            }
            FuncKind::Intrinsic | FuncKind::SchemaVariantDefinition | FuncKind::Unknown => {
                debug!(?func.kind, "no associations or input type needed for func kind");
                (None::<FuncAssociations>, String::new())
            }
        };

        Ok((associations, input_type))
    }

    async fn list_leaf_function_inputs(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncAssociationsResult<Vec<LeafInputLocation>> {
        Ok(FuncArgument::list_for_func(ctx, func_id)
            .await?
            .iter()
            .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(&arg.name))
            .collect())
    }

    // async fn compile_leaf_function_input_types(
    //     ctx: &DalContext,
    //     schema_variant_ids: &[SchemaVariantId],
    //     inputs: &[LeafInputLocation],
    // ) -> FuncViewResult<String> {
    //     let mut ts_type = "type Input = {\n".to_string();
    //
    //     for input_location in inputs {
    //         let input_property = format!(
    //             "{}?: {} | null;\n",
    //             input_location.arg_name(),
    //             Self::get_per_variant_types_for_prop_path(
    //                 ctx,
    //                 schema_variant_ids,
    //                 &input_location.prop_path(),
    //             )
    //             .await?
    //         );
    //         ts_type.push_str(&input_property);
    //     }
    //     ts_type.push_str("};");
    //
    //     Ok(ts_type)
    // }
    //
    // async fn get_per_variant_types_for_prop_path(
    //     ctx: &DalContext,
    //     variant_ids: &[SchemaVariantId],
    //     path: &PropPath,
    // ) -> FuncViewResult<String> {
    //     let mut per_variant_types = vec![];
    //
    //     for variant_id in variant_ids {
    //         let prop = Prop::find_prop_by_path(ctx, *variant_id, path).await?;
    //         let ts_type = prop.ts_type(ctx).await?;
    //
    //         if !per_variant_types.contains(&ts_type) {
    //             per_variant_types.push(ts_type);
    //         }
    //     }
    //
    //     Ok(per_variant_types.join(" | "))
    // }
}
