use action::ActionBinding;
use attribute::AttributeBinding;
use authentication::AuthBinding;
use itertools::Itertools;
use leaf::LeafBinding;
use serde::{Deserialize, Serialize};
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
use crate::{
    ChangeSetId, ComponentId, OutputSocketId, SchemaVariant, WorkspaceSnapshotError, WsEvent,
    WsEventError, WsEventResult, WsPayload,
};
pub use attribute_argument::AttributeArgumentBinding;
pub use attribute_argument::AttributeFuncArgumentSource;

use crate::attribute::prototype::AttributePrototypeError;
use crate::func::argument::FuncArgumentError;
use crate::func::FuncKind;
use crate::prop::PropError;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::{
    socket::input::InputSocketError, ActionPrototypeId, AttributePrototypeId, ComponentError,
    DalContext, Func, FuncError, FuncId, PropId, SchemaVariantError, SchemaVariantId,
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
    #[error("failed to remove attribute value for leaf")]
    FailedToRemoveLeafAttributeValue,
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("malformed input for binding: {0}")]
    MalformedInput(String),
    #[error("missing value source for attribute prototype argument id {0}")]
    MissingValueSource(AttributePrototypeArgumentId),
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
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
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

/// Represents the location where the function ultimately writes to
/// We currently only allow Attribute Funcs to be attached to Props
/// (or the attribute value in the case of a component) and Output Sockets
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeFuncDestination {
    Prop(PropId),
    OutputSocket(OutputSocketId),
}

impl AttributeFuncDestination {
    pub(crate) async fn find_schema_variant(
        &self,
        ctx: &DalContext,
    ) -> FuncBindingsResult<SchemaVariantId> {
        let schema_variant_id = match self {
            AttributeFuncDestination::Prop(prop_id) => {
                SchemaVariant::find_for_prop_id(ctx, *prop_id).await?
            }
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                SchemaVariant::find_for_output_socket_id(ctx, *output_socket_id).await?
            }
        };
        Ok(schema_variant_id)
    }
}

/// Represents at what level a given Prototype is attached to
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventualParent {
    SchemaVariant(SchemaVariantId),
    Component(ComponentId),
}

impl From<EventualParent> for Option<si_events::ComponentId> {
    fn from(value: EventualParent) -> Self {
        match value {
            EventualParent::SchemaVariant(_) => None,
            EventualParent::Component(component_id) => Some(component_id.into()),
        }
    }
}

impl From<EventualParent> for Option<si_events::SchemaVariantId> {
    fn from(value: EventualParent) -> Self {
        match value {
            EventualParent::SchemaVariant(schema_variant_id) => Some(schema_variant_id.into()),
            EventualParent::Component(_) => None,
        }
    }
}

impl From<AttributeFuncDestination> for Option<si_events::PropId> {
    fn from(value: AttributeFuncDestination) -> Self {
        match value {
            AttributeFuncDestination::Prop(prop_id) => Some(prop_id.into()),
            _ => None,
        }
    }
}

impl From<AttributeFuncDestination> for Option<si_events::OutputSocketId> {
    fn from(value: AttributeFuncDestination) -> Self {
        match value {
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                Some(output_socket_id.into())
            }
            _ => None,
        }
    }
}

impl From<FuncBinding> for si_frontend_types::FuncBinding {
    fn from(value: FuncBinding) -> Self {
        match value {
            FuncBinding::Action {
                schema_variant_id,
                action_prototype_id,
                func_id,
                kind,
            } => si_frontend_types::FuncBinding::Action {
                schema_variant_id: Some(schema_variant_id.into()),
                action_prototype_id: Some(action_prototype_id.into()),
                func_id: Some(func_id.into()),
                kind: Some(kind.into()),
            },
            FuncBinding::Attribute {
                func_id,
                attribute_prototype_id,
                eventual_parent,
                output_location,
                argument_bindings,
            } => si_frontend_types::FuncBinding::Attribute {
                func_id: Some(func_id.into()),
                attribute_prototype_id: Some(attribute_prototype_id.into()),
                component_id: eventual_parent.into(),
                schema_variant_id: eventual_parent.into(),
                prop_id: output_location.into(),
                output_socket_id: output_location.into(),
                argument_bindings: argument_bindings
                    .into_iter()
                    .map(|arg| si_frontend_types::AttributeArgumentBinding {
                        func_argument_id: arg.func_argument_id.into(),
                        attribute_prototype_argument_id: arg
                            .attribute_prototype_argument_id
                            .map(Into::into),
                        prop_id: arg.attribute_func_input_location.clone().into(),
                        input_socket_id: arg.attribute_func_input_location.clone().into(),
                    })
                    .collect_vec(),
            },
            FuncBinding::Authentication {
                schema_variant_id,
                func_id,
            } => si_frontend_types::FuncBinding::Authentication {
                schema_variant_id: schema_variant_id.into(),
                func_id: func_id.into(),
            },
            FuncBinding::CodeGeneration {
                func_id,
                attribute_prototype_id,
                eventual_parent,
                inputs,
            } => si_frontend_types::FuncBinding::CodeGeneration {
                schema_variant_id: eventual_parent.into(),
                component_id: eventual_parent.into(),
                func_id: Some(func_id.into()),
                attribute_prototype_id: Some(attribute_prototype_id.into()),
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
                attribute_prototype_id: Some(attribute_prototype_id.into()),
                inputs: inputs.into_iter().map(|input| input.into()).collect_vec(),
            },
        }
    }
}

/// A [`FuncBinding`] represents the intersection of a function and the [`SchemaVariant`] (or [`Component`])
/// specific information required to know when to run a function, what it's inputs are, and where the result of
/// the function is recorded  
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, EnumDiscriminants, Hash)]
pub enum FuncBinding {
    /// An Action function can only (currently) be attached to a [`Schema Variant`]
    /// The [`ActionPrototypeId`] represents the unique relationship to a particular [`SchemaVariant`]
    /// Interestingly, the [`ActionKind`] is a property on the [`ActionPrototype`], meaning changing the
    /// [`ActionKind`] does not require any changes to the [`Func`] itself.
    Action {
        // unique ids
        schema_variant_id: SchemaVariantId,
        action_prototype_id: ActionPrototypeId,
        func_id: FuncId,
        //thing that can be updated
        kind: ActionKind,
    },
    /// An Attribute function is a function that sets values within a [`SchemaVariant`] or [`Component`]'s Prop Tree
    /// Each Attribute Function has user defined arguments, configured to map to specific Props or InputSockets.
    /// This intersection of [`FuncArgument`] and the [`Prop`] or [`InputSocket`] mapping is an [`AttributeArgumentBinding`]
    Attribute {
        // unique ids
        func_id: FuncId,
        attribute_prototype_id: AttributePrototypeId,
        // things needed for create
        eventual_parent: EventualParent,

        // things that can be updated
        output_location: AttributeFuncDestination,
        argument_bindings: Vec<AttributeArgumentBinding>,
    },
    /// Auth Funcs only exist on Secret defining schema variants, and have no special configuration data aside from the
    /// [`SchemaVariantId`] and as such are only created or deleted (detached), they are not updated.
    Authentication {
        // unique ids
        schema_variant_id: SchemaVariantId,
        func_id: FuncId,
    },
    /// CodeGen funcs are ultimately just an Attribute Function, but the user can not control where they output to.
    /// They write to an Attribute Value beneath the Code Gen Root Prop Node
    CodeGeneration {
        // unique ids
        func_id: FuncId,
        attribute_prototype_id: AttributePrototypeId,
        // things needed for create
        eventual_parent: EventualParent,
        // thing that can be updated
        inputs: Vec<LeafInputLocation>,
    },
    /// Qualification funcs are ultimately just an Attribute Function, but the user can not control where they output to.
    /// They write to an Attribute Value beneath the Qualification Root Prop Node
    Qualification {
        // unique ids
        func_id: FuncId,
        attribute_prototype_id: AttributePrototypeId,
        // things needed for create
        eventual_parent: EventualParent,
        // thing that can be updated
        inputs: Vec<LeafInputLocation>,
    },
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncBindings {
    pub bindings: Vec<FuncBinding>,
}

impl FuncBindings {
    /// converts this enum to the front end type
    pub fn into_frontend_type(&self) -> si_frontend_types::FuncBindings {
        let mut front_end_bindings = Vec::with_capacity(self.bindings.len());
        for binding in &self.bindings {
            let front_end_binding = match binding.clone() {
                FuncBinding::Action {
                    schema_variant_id,
                    action_prototype_id,
                    func_id,
                    kind,
                } => si_frontend_types::FuncBinding::Action {
                    schema_variant_id: Some(schema_variant_id.into()),
                    action_prototype_id: Some(action_prototype_id.into()),
                    func_id: Some(func_id.into()),
                    kind: Some(kind.into()),
                },
                FuncBinding::Attribute {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent,
                    output_location,
                    argument_bindings,
                } => si_frontend_types::FuncBinding::Attribute {
                    func_id: Some(func_id.into()),
                    attribute_prototype_id: Some(attribute_prototype_id.into()),
                    component_id: eventual_parent.into(),
                    schema_variant_id: eventual_parent.into(),
                    prop_id: output_location.into(),
                    output_socket_id: output_location.into(),
                    argument_bindings: argument_bindings
                        .into_iter()
                        .map(|f| f.into_frontend_type())
                        .collect_vec(),
                },
                FuncBinding::Authentication {
                    schema_variant_id,
                    func_id,
                } => si_frontend_types::FuncBinding::Authentication {
                    schema_variant_id: schema_variant_id.into(),
                    func_id: func_id.into(),
                },
                FuncBinding::CodeGeneration {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent,
                    inputs,
                } => si_frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id: eventual_parent.into(),
                    component_id: eventual_parent.into(),
                    func_id: Some(func_id.into()),
                    attribute_prototype_id: Some(attribute_prototype_id.into()),
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
                    attribute_prototype_id: Some(attribute_prototype_id.into()),
                    inputs: inputs.into_iter().map(|input| input.into()).collect_vec(),
                },
            };
            front_end_bindings.push(front_end_binding);
        }
        si_frontend_types::FuncBindings {
            bindings: front_end_bindings,
        }
    }

    /// For a given [`FuncId`], gather all of the bindings for every place where it is being used
    #[instrument(
        level = "debug",
        skip(ctx),
        name = "func.binding.delete_all_bindings_for_func_id"
    )]
    pub async fn from_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<FuncBindings> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        let bindings: Vec<FuncBinding> = match func.kind {
            FuncKind::Action => ActionBinding::assemble_action_bindings(ctx, func_id).await?,
            FuncKind::Attribute => {
                AttributeBinding::assemble_attribute_bindings(ctx, func_id).await?
            }
            FuncKind::Authentication => AuthBinding::assemble_auth_bindings(ctx, func_id).await?,
            FuncKind::CodeGeneration => {
                LeafBinding::assemble_code_gen_bindings(ctx, func_id).await?
            }
            FuncKind::Qualification => {
                LeafBinding::assemble_qualification_bindings(ctx, func_id).await?
            }

            FuncKind::Intrinsic | FuncKind::SchemaVariantDefinition | FuncKind::Unknown => {
                //debug!(?func.kind, "no associations or input type needed for func kind");
                vec![]
            }
        };
        Ok(FuncBindings { bindings })
    }

    /// Removes all existing bindings for a given [`FuncId`]
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.delete_all_bindings_for_func_id"
    )]
    pub async fn delete_all_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<()> {
        let func_bindings = FuncBindings::from_func_id(ctx, func_id).await?;
        for binding in func_bindings.bindings {
            match binding {
                FuncBinding::Action {
                    action_prototype_id,
                    ..
                } => ActionBinding::delete_action_binding(ctx, action_prototype_id).await?,
                FuncBinding::Attribute {
                    attribute_prototype_id,
                    ..
                } => AttributeBinding::reset_attribute_binding(ctx, attribute_prototype_id).await?,
                FuncBinding::Authentication {
                    schema_variant_id,
                    func_id,
                } => AuthBinding::delete_auth_binding(ctx, func_id, schema_variant_id).await?,
                FuncBinding::CodeGeneration {
                    attribute_prototype_id,
                    ..
                } => LeafBinding::delete_leaf_func_binding(ctx, attribute_prototype_id).await?,
                FuncBinding::Qualification {
                    attribute_prototype_id,
                    ..
                } => LeafBinding::delete_leaf_func_binding(ctx, attribute_prototype_id).await?,
            };
        }
        Ok(())
    }

    /// Compile all the types for all of the bindings to return to the front end for type checking
    pub async fn compile_types(ctx: &DalContext, func_id: FuncId) -> FuncBindingsResult<String> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        let types: String = match func.kind {
            FuncKind::Action => ActionBinding::compile_action_types(ctx, func_id).await?,
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                LeafBinding::compile_leaf_func_types(ctx, func_id).await?
            }
            FuncKind::Attribute => AttributeBinding::compile_attribute_types(ctx, func_id).await?,
            FuncKind::Authentication
            | FuncKind::Intrinsic
            | FuncKind::SchemaVariantDefinition
            | FuncKind::Unknown => String::new(),
        };
        Ok(types)
    }

    /// Get the ActionBinding if it exists, otherwise return an error. Useful for tests
    pub fn get_action_internals(&self) -> FuncBindingsResult<Vec<(ActionKind, SchemaVariantId)>> {
        let mut actions = vec![];
        for binding in self.bindings.clone() {
            match binding {
                FuncBinding::Action {
                    schema_variant_id,
                    kind,
                    ..
                } => actions.push((kind, schema_variant_id)),
                other_binding => {
                    return Err(FuncBindingsError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Action,
                    ))
                }
            }
        }
        Ok(actions)
    }

    // this func is just for integration tests.
    #[allow(clippy::type_complexity)]
    /// Get the Attribute Binding if it exists, otherwise return an error. Useful for tests
    pub fn get_attribute_internals(
        &self,
    ) -> FuncBindingsResult<
        Vec<(
            AttributePrototypeId,
            EventualParent,
            AttributeFuncDestination,
            Vec<AttributeArgumentBinding>,
        )>,
    > {
        let mut attributes = vec![];
        for binding in self.bindings.clone() {
            match binding {
                FuncBinding::Attribute {
                    func_id: _,
                    attribute_prototype_id,
                    eventual_parent,
                    output_location,
                    argument_bindings,
                } => {
                    attributes.push((
                        attribute_prototype_id,
                        eventual_parent,
                        output_location,
                        argument_bindings,
                    ));
                }
                other_binding => {
                    return Err(FuncBindingsError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Authentication,
                    ))
                }
            }
        }
        Ok(attributes)
    }
}
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncBindingsWsEventPayload {
    change_set_id: ChangeSetId,
    #[serde(flatten)]
    bindings: si_frontend_types::FuncBindings,
    types: String,
}

impl WsEvent {
    pub async fn func_bindings_updated(
        ctx: &DalContext,
        bindings: si_frontend_types::FuncBindings,
        types: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncBindingsUpdated(FuncBindingsWsEventPayload {
                change_set_id: ctx.change_set_id(),
                bindings,
                types,
            }),
        )
        .await
    }
}
