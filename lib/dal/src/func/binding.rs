use itertools::Itertools;
use serde::{Deserialize, Serialize};
use si_events::{ComponentId, SchemaVariantId};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::action::prototype::{ActionKind, ActionPrototypeError};
use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::AttributeValueError;
use crate::socket::output::OutputSocketError;
use crate::{OutputSocketId, WorkspaceSnapshotError, WsEventError};
use attribute_argument::AttributeArgumentBinding;

use crate::attribute::prototype::AttributePrototypeError;
use crate::func::argument::FuncArgumentError;
use crate::func::FuncKind;
use crate::prop::PropError;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::{
    socket::input::InputSocketError, ActionPrototypeId, AttributePrototypeId, ComponentError,
    ComponentId, DalContext, Func, FuncError, FuncId, PropId, SchemaVariantError, SchemaVariantId,
};

use super::argument::FuncArgumentId;

pub mod action;
pub mod attribute;
pub mod attribute_argument;
pub mod authentication;
pub mod leaf;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncBindingsError {
    #[error("action with kind ({0}) already exists for schema variant ({1}), cannot have two non-manual actions for the same kind in the same schema variant")]
    ActionKindAlreadyExists(ActionKind, SchemaVariantId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("action prototype missing")]
    ActionPrototypeMissing,
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype missing")]
    AttributePrototypeMissing,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("cannot compile types for func: {0}")]
    CannotCompileTypes(FuncId),
    #[error("component error: {0}")]
    ComponentError(#[from] ComponentError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("no input location given for attribute prototype id ({0}) and func argument id ({1})")]
    NoInputLocationGiven(AttributePrototypeId, FuncArgumentId),
    #[error("no output location given for func: {0}")]
    NoOutputLocationGiven(FuncId),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("unexpected func binding variant: {0:?} (expected: {1:?})")]
    UnexpectedFuncBindingVariant(FuncBindingDiscriminants, FuncBindingDiscriminants),
    #[error("unexpected func kind ({0}) creating attribute func")]
    UnexpectedFuncKind(FuncKind),
    #[error("unexpected value source ({0:?}) for attribute prototype argument: {1}")]
    UnexpectedValueSource(ValueSource, AttributePrototypeArgumentId),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}
type FuncBindingsResult<T> = Result<T, FuncBindingsError>;

enum AttributeFuncOutputLocation {
    Prop(PropId),
    OutputSocket(OutputSocketId),
}
pub enum EventualParent {
    SchemaVariant(SchemaVariantId),
    Component(ComponentId),
}
impl Into<Option<si_events::ComponentId>> for EventualParent {
    fn into(self) -> si_events::ComponentId {
        match self {
            EventualParent::SchemaVariant(_) => None,
            EventualParent::Component(component_id) => Some(component_id.into()),
        }
    }
}

impl Into<Option<si_events::SchemaVariantId>> for EventualParent {
    fn into(self) -> si_events::SchemaVariantId {
        match self {
            EventualParent::SchemaVariant(schema_variant_id) => Some(schema_variant_id.into()),
            EventualParent::Component(_) => None,
        }
    }
}
impl Into<Option<si_events::PropId>> for AttributeFuncOutputLocation {
    fn into(self) -> Option<si_events::PropId> {
        match self {
            AttributeFuncOutputLocation::Prop(prop_id) => Some(prop_id.into()),
            AttributeFuncOutputLocation::OutputSocket(_) => None,
        }
    }
}
impl Into<Options<si_events::OutputSocketId>> for AttributeFuncOutputLocation {
    fn into(self) -> Options<si_events::OutputSocketId> {
        match self {
            AttributeFuncOutputLocation::Prop(_) => None,
            AttributeFuncOutputLocation::OutputSocket(output_socket_id) => {
                Some(output_socket_id.into())
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, EnumDiscriminants)]
pub enum FuncBinding {
    Action {
        // unique ids
        schema_variant_id: SchemaVariantId,
        action_prototype_id: Option<ActionPrototypeId>,
        func_id: FuncId,
        //thing that can be updated
        kind: ActionKind,
    },
    Attribute {
        // unique ids
        func_id: FuncId,
        attribute_prototype_id: Option<AttributePrototypeId>,
        // things needed for create
        eventual_parent: EventualParent,

        // things that can be updated
        output_location: AttributeFuncOutputLocation,
        argument_bindings: Vec<AttributeArgumentBinding>,
    },
    #[serde(rename_all = "camelCase")]
    Authentication {
        // unique ids
        schema_variant_id: SchemaVariantId,
        func_id: FuncId,
    },
    #[serde(rename_all = "camelCase")]
    CodeGeneration {
        // unique ids
        func_id: FuncId,
        attribute_prototype_id: Option<AttributePrototypeId>,
        // things needed for create
        eventual_parent: EventualParent,
        // thing that can be updated
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    Qualification {
        // unique ids
        func_id: FuncId,
        attribute_prototype_id: Option<AttributePrototypeId>,
        // things needed for create
        eventual_parent: EventualParent,
        // thing that can be updated
        inputs: Vec<LeafInputLocation>,
    },
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncBindings {
    pub func_id: FuncId,
    pub bindings: Vec<FuncBinding>,
}

impl FuncBindings {
    pub fn into_frontend_type(&self) -> si_frontend_types::FuncBindings {
        let mut front_end_bindings = Vec::with_capacity(self.bindings.len());
        for binding in self.bindings {
            let front_end_binding = match binding {
                FuncBinding::Action {
                    schema_variant_id,
                    action_prototype_id,
                    func_id,
                    kind,
                } => si_frontend_types::FuncBinding::Action {
                    schema_variant_id: Some(schema_variant_id),
                    action_prototype_id: attribute_prototype_id.map(Into::into),
                    func_id,
                    kind: Some(kind),
                },
                FuncBinding::Attribute {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent,
                    output_location,
                    argument_bindings,
                } => si_frontend_types::FuncBinding::Attribute {
                    func_id: func_id.map(Into::into),
                    attribute_prototype_id: attribute_prototype_id
                        ..map(Into::<si_events::ulid::Ulid>::into),
                    component_id: eventual_parent.into(),
                    schema_variant_id: eventual_parent.into(),
                    prop_id: match output_location {
                        AttributeFuncOutputLocation::Prop(prop_id) => Some(prop_id.into()),
                        AttributeFuncOutputLocation::OutputSocket(_) => None,
                    },
                    output_socket_id: match output_location {
                        AttributeFuncOutputLocation::Prop(_) => None,
                        AttributeFuncOutputLocation::OutputSocket(output_socket_id) => {
                            Some(output_socket_id.into())
                        }
                    },
                    argument_bindings: argument_bindings
                        .into_iter()
                        .map(|f| f.into())
                        .collect_vec(),
                },
                FuncBinding::Authentication {
                    schema_variant_id,
                    func_id,
                } => si_frontend_types::FuncBinding::Authentication {
                    schema_variant_id: Some(schema_variant_id.into()),
                    func_id: (),
                },
                FuncBinding::CodeGeneration {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent,
                    inputs,
                } => si_frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id: eventual_parent.into(),
                    component_id: eventual_parent.into(),
                    func_id: func_id.map(Into::into),
                    attribute_prototype_id: attribute_prototype_id.map(Into::into),
                    inputs: inputs.into_iter().map(|input| input.into()).collect_vec(),
                },
                FuncBinding::Qualification {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent,
                    inputs,
                } => si_frontend_types::FuncBinding::Qualification {
                    schema_variant_id: eventual_parent.into(),
                    component_id: eventual_parent.into(),
                    func_id: Some(func_id.into()),
                    attribute_prototype_id: attribute_prototype_id.map(Into::into),
                    inputs: inputs.into_iter().map(|input| input.into()).collect_vec(),
                },
            };
            front_end_bindings.push(front_end_binding);
        }
        si_frontend_types::FuncBindings {
            bindings: front_end_bindings,
        }
    }
    pub async fn from_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<FuncBindings> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        let bindings: Vec<FuncBinding> = match func.kind {
            FuncKind::Action => action::assemble_action_bindings(ctx, func_id).await?,
            FuncKind::Attribute => attribute::assemble_attribute_bindings(ctx, func_id).await?,
            FuncKind::Authentication => {
                authentication::assemble_auth_bindings(ctx, func_id).await?
            }
            FuncKind::CodeGeneration => leaf::assemble_code_gen_bindings(ctx, func_id).await?,
            FuncKind::Qualification => leaf::assemble_qualification_bindings(ctx, func_id).await?,

            FuncKind::Intrinsic | FuncKind::SchemaVariantDefinition | FuncKind::Unknown => {
                debug!(?func.kind, "no associations or input type needed for func kind");
                vec![]
            }
        };
        Ok(FuncBindings { func_id, bindings })
    }
    pub async fn compile_types(ctx: &DalContext, func_id: FuncId) -> FuncBindingsResult<String> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        let types: String = match func.kind {
            FuncKind::Action => action::compile_action_types(ctx, func_id).await?,
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                leaf::compile_leaf_func_types(ctx, func_id).await?
            }
            FuncKind::Attribute => attribute::compile_attribute_types(ctx, func_id).await?,
            FuncKind::Authentication
            | FuncKind::Intrinsic
            | FuncKind::SchemaVariantDefinition
            | FuncKind::Unknown => String::new(),
        };
        Ok(types)
    }
}
