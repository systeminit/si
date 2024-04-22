use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::AttributePrototypeArgumentError;
use crate::attribute::prototype::{AttributePrototypeError, AttributePrototypeEventualParent};
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::view::FuncArgumentView;
use crate::func::FuncKind;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::{
    AttributePrototype, ComponentId, DalContext, DeprecatedActionKind, DeprecatedActionPrototype,
    DeprecatedActionPrototypeError, Func, FuncId, SchemaVariant, SchemaVariantError,
    SchemaVariantId,
};

mod view;

pub use view::AttributePrototypeArgumentView;
pub use view::AttributePrototypeView;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAssociationsError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("deprecated action prototype error: {0}")]
    DeprecatedActionPrototype(#[from] DeprecatedActionPrototypeError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("unexpected func associations variant: {0:?} (expected: {1:?})")]
    UnexpectedFuncAssociationsVariant(FuncAssociationsDiscriminants, FuncAssociationsDiscriminants),
}

type FuncAssociationsResult<T> = Result<T, FuncAssociationsError>;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, EnumDiscriminants)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    #[serde(rename_all = "camelCase")]
    Action {
        kind: DeprecatedActionKind,
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
                let action_prototype_ids =
                    DeprecatedActionPrototype::list_for_func_id(ctx, func.id).await?;

                // TODO(nick): right now, we just grab the first one and it decides the action kind for all of them.
                // This should be configurable on a "per prototype" basis in the future.
                let kind = match action_prototype_ids.first() {
                    Some(action_prototype_id) => {
                        let action_prototype = DeprecatedActionPrototype::get_by_id_or_error(
                            ctx,
                            *action_prototype_id,
                        )
                        .await?;
                        action_prototype.kind
                    }
                    None => DeprecatedActionKind::Create,
                };

                (
                    Some(Self::Action {
                        kind,
                        schema_variant_ids,
                    }),
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

    pub fn get_action_internals(
        &self,
    ) -> FuncAssociationsResult<(DeprecatedActionKind, Vec<SchemaVariantId>)> {
        match self {
            FuncAssociations::Action {
                kind,
                schema_variant_ids,
            } => Ok((*kind, schema_variant_ids.to_owned())),
            associations => Err(FuncAssociationsError::UnexpectedFuncAssociationsVariant(
                associations.into(),
                FuncAssociationsDiscriminants::Action,
            )),
        }
    }

    pub fn get_attribute_internals(
        &self,
    ) -> FuncAssociationsResult<(Vec<AttributePrototypeView>, Vec<FuncArgumentView>)> {
        match self {
            FuncAssociations::Attribute {
                prototypes,
                arguments,
            } => Ok((prototypes.to_owned(), arguments.to_owned())),
            associations => Err(FuncAssociationsError::UnexpectedFuncAssociationsVariant(
                associations.into(),
                FuncAssociationsDiscriminants::Attribute,
            )),
        }
    }

    pub fn get_authentication_internals(&self) -> FuncAssociationsResult<Vec<SchemaVariantId>> {
        match self {
            FuncAssociations::Authentication { schema_variant_ids } => {
                Ok(schema_variant_ids.to_owned())
            }
            associations => Err(FuncAssociationsError::UnexpectedFuncAssociationsVariant(
                associations.into(),
                FuncAssociationsDiscriminants::Authentication,
            )),
        }
    }

    pub fn get_code_generation_internals(
        &self,
    ) -> FuncAssociationsResult<(
        Vec<SchemaVariantId>,
        Vec<ComponentId>,
        Vec<LeafInputLocation>,
    )> {
        match self {
            FuncAssociations::CodeGeneration {
                schema_variant_ids,
                component_ids,
                inputs,
            } => Ok((
                schema_variant_ids.to_owned(),
                component_ids.to_owned(),
                inputs.to_owned(),
            )),
            associations => Err(FuncAssociationsError::UnexpectedFuncAssociationsVariant(
                associations.into(),
                FuncAssociationsDiscriminants::CodeGeneration,
            )),
        }
    }

    pub fn get_qualification_internals(
        &self,
    ) -> FuncAssociationsResult<(
        Vec<SchemaVariantId>,
        Vec<ComponentId>,
        Vec<LeafInputLocation>,
    )> {
        match self {
            FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
                inputs,
            } => Ok((
                schema_variant_ids.to_owned(),
                component_ids.to_owned(),
                inputs.to_owned(),
            )),
            associations => Err(FuncAssociationsError::UnexpectedFuncAssociationsVariant(
                associations.into(),
                FuncAssociationsDiscriminants::Qualification,
            )),
        }
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
