use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::{AttributePrototypeError, AttributePrototypeEventualParent};
use crate::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId};
use crate::func::view::FuncArgumentView;
use crate::func::FuncKind;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::{
    AttributePrototype, AttributePrototypeId, ComponentId, DalContext, Func, FuncId, InputSocketId,
    OutputSocketId, PropId, SchemaVariant, SchemaVariantError, SchemaVariantId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAssociationsError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

type FuncAssociationsResult<T> = Result<T, FuncAssociationsError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentView {
    pub func_argument_id: FuncArgumentId,
    pub id: AttributePrototypeArgumentId,
    pub input_socket_id: Option<InputSocketId>,
}

impl AttributePrototypeArgumentView {
    pub async fn assemble(
        ctx: &DalContext,
        id: AttributePrototypeArgumentId,
    ) -> FuncAssociationsResult<Self> {
        let attribute_prototype_argument = AttributePrototypeArgument::get_by_id(ctx, id).await?;

        let input_socket_id =
            if let Some(value_source) = attribute_prototype_argument.value_source(ctx).await? {
                match value_source {
                    ValueSource::InputSocket(input_socket_id) => Some(input_socket_id),
                    ValueSource::OutputSocket(_)
                    | ValueSource::Prop(_)
                    | ValueSource::StaticArgumentValue(_) => None,
                }
            } else {
                None
            };

        let func_argument_id = AttributePrototypeArgument::func_argument_id_by_id(ctx, id).await?;

        Ok(Self {
            func_argument_id,
            id,
            input_socket_id,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    pub id: AttributePrototypeId,
    pub component_id: Option<ComponentId>,
    pub schema_variant_id: Option<SchemaVariantId>,
    pub prop_id: Option<PropId>,
    pub output_socket_id: Option<OutputSocketId>,
    pub prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub async fn assemble(
        ctx: &DalContext,
        id: AttributePrototypeId,
    ) -> FuncAssociationsResult<Self> {
        let attribute_prototype_argument_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, id).await?;

        let eventual_parent = AttributePrototype::eventual_parent(ctx, id).await?;
        let (component_id, schema_variant_id, prop_id, output_socket_id) = match eventual_parent {
            AttributePrototypeEventualParent::Component(component_id) => {
                (Some(component_id), None, None, None)
            }
            AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                schema_variant_id,
                _,
            ) => (None, Some(schema_variant_id), None, None),
            AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                schema_variant_id,
                output_socket_id,
            ) => (None, Some(schema_variant_id), None, Some(output_socket_id)),
            AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, prop_id) => {
                (None, Some(schema_variant_id), Some(prop_id), None)
            }
        };

        let mut prototype_arguments = Vec::new();
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            prototype_arguments.push(
                AttributePrototypeArgumentView::assemble(ctx, attribute_prototype_argument_id)
                    .await?,
            );
        }

        Ok(Self {
            id,
            component_id,
            schema_variant_id,
            prop_id,
            output_socket_id,
            prototype_arguments,
        })
    }
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
                    // TODO(nick): ensure the input type is correct.
                    String::new(),
                )
            }
            FuncKind::Attribute => {
                let attribute_prototype_ids =
                    AttributePrototype::list_ids_for_func_id(ctx, func.id).await?;

                let mut prototypes = Vec::new();
                for attribute_prototype_id in attribute_prototype_ids {
                    prototypes
                        .push(AttributePrototypeView::assemble(ctx, attribute_prototype_id).await?);
                }

                (
                    Some(Self::Attribute {
                        prototypes,
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
                    // TODO(nick): ensure the input type is correct.
                    "type Input = any".into(),
                )
            }
            FuncKind::Authentication => {
                let schema_variant_ids = SchemaVariant::list_for_auth_func(ctx, func.id).await?;
                (
                    Some(Self::Authentication { schema_variant_ids }),
                    // TODO(nick): ensure the input type is correct.
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
            FuncKind::CodeGeneration => {
                let attribute_prototype_ids =
                    AttributePrototype::list_ids_for_func_id(ctx, func.id).await?;

                let mut schema_variant_ids = Vec::new();
                let mut component_ids = Vec::new();

                for attribute_prototype_id in attribute_prototype_ids {
                    let eventual_parent =
                        AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;

                    match eventual_parent {
                        AttributePrototypeEventualParent::Component(component_id) => {
                            component_ids.push(component_id)
                        }
                        AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                            schema_variant_id,
                            _,
                        ) => schema_variant_ids.push(schema_variant_id),
                        AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                            schema_variant_id,
                            _,
                        ) => schema_variant_ids.push(schema_variant_id),
                        AttributePrototypeEventualParent::SchemaVariantFromProp(
                            schema_variant_id,
                            _,
                        ) => schema_variant_ids.push(schema_variant_id),
                    }
                }

                (
                    Some(Self::CodeGeneration {
                        schema_variant_ids,
                        component_ids,
                        inputs: Self::list_leaf_function_inputs(ctx, func.id).await?,
                    }),
                    // TODO(nick): ensure the input type is correct.
                    "".to_string(),
                )
            }
            FuncKind::Qualification => {
                let attribute_prototype_ids =
                    AttributePrototype::list_ids_for_func_id(ctx, func.id).await?;

                let mut schema_variant_ids = Vec::new();
                let mut component_ids = Vec::new();

                for attribute_prototype_id in attribute_prototype_ids {
                    let eventual_parent =
                        AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
                    match eventual_parent {
                        AttributePrototypeEventualParent::Component(component_id) => {
                            component_ids.push(component_id)
                        }
                        AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                            schema_variant_id,
                            _,
                        ) => schema_variant_ids.push(schema_variant_id),
                        AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                            schema_variant_id,
                            _,
                        ) => schema_variant_ids.push(schema_variant_id),
                        AttributePrototypeEventualParent::SchemaVariantFromProp(
                            schema_variant_id,
                            _,
                        ) => schema_variant_ids.push(schema_variant_id),
                    }
                }

                (
                    Some(Self::Qualification {
                        schema_variant_ids,
                        component_ids,
                        inputs: Self::list_leaf_function_inputs(ctx, func.id).await?,
                    }),
                    // TODO(nick): ensure the input type is correct.
                    "".to_string(),
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
