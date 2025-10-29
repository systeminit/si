use std::collections::HashMap;

use action::ActionBinding;
use attribute::AttributeBinding;
pub use attribute_argument::{
    AttributeArgumentBinding,
    AttributeFuncArgumentSource,
};
use authentication::AuthBinding;
use itertools::Itertools;
use leaf::LeafBinding;
use management::ManagementBinding;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types::LeafBindingPrototype;
use si_id::ManagementPrototypeId;
use si_split_graph::SplitGraphError;
use strum::{
    Display,
    EnumDiscriminants,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributePrototypeId,
    ComponentError,
    ComponentId,
    DalContext,
    Func,
    FuncError,
    FuncId,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    SchemaError,
    SchemaId,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    WorkspaceSnapshotError,
    WsEventError,
    action::prototype::{
        ActionKind,
        ActionPrototypeError,
    },
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgumentError,
                AttributePrototypeArgumentId,
                value_source::ValueSource,
            },
        },
        value::AttributeValueError,
    },
    func::{
        FuncKind,
        argument::{
            FuncArgumentError,
            FuncArgumentId,
        },
        binding::attribute::AttributeBindingMalformedInput,
        leaf::LeafKind,
    },
    management::prototype::ManagementPrototypeError,
    prop::PropError,
    schema::leaf::LeafPrototypeError,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::{
        dependent_value_root::DependentValueRootError,
        graph::WorkspaceSnapshotGraphError,
    },
};

pub mod action;
pub mod attribute;
pub mod attribute_argument;
pub mod authentication;
pub mod leaf;
pub mod management;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncBindingError {
    #[error(
        "action with kind ({0}) already exists for schema variant ({1}), cannot have two non-manual actions for the same kind in the same schema variant"
    )]
    ActionKindAlreadyExists(ActionKind, SchemaVariantId),
    #[error(
        "action with kind ({0}) already exists for schema ({1}), cannot have two non-manual actions for the same kind in the same schema"
    )]
    ActionKindAlreadyExistsForSchema(ActionKind, SchemaId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("action prototype missing")]
    ActionPrototypeMissing,
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute prototype missing")]
    AttributePrototypeMissing,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("cached modules error: {0}")]
    CachedModules(#[from] crate::cached_module::CachedModuleError),
    #[error("cannot compile types for func: {0}")]
    CannotCompileTypes(FuncId),
    #[error("cannot set intrinsic func for component: {0}")]
    CannotSetIntrinsicForComponent(ComponentId),
    #[error("component error: {0}")]
    ComponentError(#[from] Box<ComponentError>),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] Box<FuncArgumentError>),
    #[error("func argument missing for func arg: {0} with name: {1}")]
    FuncArgumentMissing(FuncArgumentId, String),
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("invalid attribute prototype argument source: {0}")]
    InvalidAttributePrototypeArgumentSource(AttributeFuncArgumentSource),
    #[error("invalid attribute prototype destination: {0}")]
    InvalidAttributePrototypeDestination(AttributeFuncDestination),
    #[error("invalid intrinsic binding")]
    InvalidIntrinsicBinding,
    #[error("leaf prototype error: {0}")]
    LeafPrototypeError(#[from] Box<LeafPrototypeError>),
    #[error("malformed input for binding: {0:?}")]
    MalformedInput(AttributeBindingMalformedInput),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] Box<ManagementPrototypeError>),
    #[error("management prototype has no parent: {0}")]
    ManagementPrototypeNoParent(ManagementPrototypeId),
    #[error("no input location given for attribute prototype id ({0}) and func argument id ({1})")]
    NoInputLocationGiven(AttributePrototypeId, FuncArgumentId),
    #[error("no output location given for func: {0}")]
    NoOutputLocationGiven(FuncId),
    #[error("no schemas specified as eventual parent of func")]
    NoSchemas,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] Box<OutputSocketError>),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
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
    WsEvent(#[from] Box<WsEventError>),
}

impl FuncBindingError {
    pub fn is_create_graph_cycle(&self) -> bool {
        match self {
            Self::AttributePrototype(err) => matches!(
                err.as_ref(),
                AttributePrototypeError::WorkspaceSnapshot(
                    WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                        WorkspaceSnapshotGraphError::CreateGraphCycle,
                    ) | WorkspaceSnapshotError::SplitGraph(SplitGraphError::WouldCreateGraphCycle),
                )
            ),
            Self::AttributePrototypeArgument(err) => matches!(
                err.as_ref(),
                AttributePrototypeArgumentError::WorkspaceSnapshot(
                    WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                        WorkspaceSnapshotGraphError::CreateGraphCycle,
                    ) | WorkspaceSnapshotError::SplitGraph(SplitGraphError::WouldCreateGraphCycle),
                )
            ),
            Self::SchemaVariant(err) => match err.as_ref() {
                SchemaVariantError::AttributePrototypeArgument(err) => matches!(
                    err.as_ref(),
                    AttributePrototypeArgumentError::WorkspaceSnapshot(
                        WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                            WorkspaceSnapshotGraphError::CreateGraphCycle,
                        ) | WorkspaceSnapshotError::SplitGraph(
                            SplitGraphError::WouldCreateGraphCycle
                        ),
                    )
                ),
                _ => false,
            },
            _ => false,
        }
    }
}

type FuncBindingResult<T> = Result<T, FuncBindingError>;

/// Represents the location where the function ultimately writes to
/// We currently only allow Attribute Funcs to be attached to Props
/// (or the attribute value in the case of a component) and Output Sockets
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum AttributeFuncDestination {
    Prop(PropId),
    OutputSocket(OutputSocketId),
    InputSocket(InputSocketId),
}

impl AttributeFuncDestination {
    pub async fn find_schema_variant(
        &self,
        ctx: &DalContext,
    ) -> FuncBindingResult<SchemaVariantId> {
        let schema_variant_id = match self {
            AttributeFuncDestination::Prop(prop_id) => {
                SchemaVariant::find_for_prop_id(ctx, *prop_id).await?
            }
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                SchemaVariant::find_for_output_socket_id(ctx, *output_socket_id).await?
            }
            AttributeFuncDestination::InputSocket(input_socket_id) => {
                SchemaVariant::find_for_input_socket_id(ctx, *input_socket_id).await?
            }
        };
        Ok(schema_variant_id)
    }

    pub async fn get_name_of_destination(&self, ctx: &DalContext) -> FuncBindingResult<String> {
        let name = match self {
            AttributeFuncDestination::Prop(prop_id) => Prop::get_by_id(ctx, *prop_id).await?.name,
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                OutputSocket::get_by_id(ctx, *output_socket_id)
                    .await?
                    .name()
                    .to_string()
            }
            AttributeFuncDestination::InputSocket(input_socket_id) => {
                InputSocket::get_by_id(ctx, *input_socket_id)
                    .await?
                    .name()
                    .to_string()
            }
        };
        Ok(name)
    }
}

/// Represents at what level a given Prototype is attached to
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventualParent {
    SchemaVariant(SchemaVariantId),
    Component(ComponentId),
    Schemas(Vec<SchemaId>),
}
impl EventualParent {
    /// Returns an error if the [`EventualParent`] is a locked [`SchemaVariant`]
    pub(crate) async fn error_if_locked(&self, ctx: &DalContext) -> FuncBindingResult<()> {
        match self {
            EventualParent::SchemaVariant(schema_variant_id) => {
                Ok(SchemaVariant::error_if_locked(ctx, *schema_variant_id).await?)
            }
            EventualParent::Schemas(_) | EventualParent::Component(_) => Ok(()),
        }
    }
}

impl From<EventualParent> for Option<si_events::ComponentId> {
    fn from(value: EventualParent) -> Self {
        match value {
            EventualParent::SchemaVariant(_) | EventualParent::Schemas(_) => None,
            EventualParent::Component(component_id) => Some(component_id),
        }
    }
}

impl From<&EventualParent> for Option<si_events::ComponentId> {
    fn from(value: &EventualParent) -> Self {
        match value {
            EventualParent::SchemaVariant(_) | EventualParent::Schemas(_) => None,
            EventualParent::Component(component_id) => Some(*component_id),
        }
    }
}

impl From<EventualParent> for Option<si_events::SchemaVariantId> {
    fn from(value: EventualParent) -> Self {
        match value {
            EventualParent::SchemaVariant(schema_variant_id) => Some(schema_variant_id),
            EventualParent::Component(_) | EventualParent::Schemas(_) => None,
        }
    }
}

impl From<&EventualParent> for Option<si_events::SchemaVariantId> {
    fn from(value: &EventualParent) -> Self {
        match value {
            EventualParent::SchemaVariant(schema_variant_id) => Some(*schema_variant_id),
            EventualParent::Component(_) | EventualParent::Schemas(_) => None,
        }
    }
}

impl From<AttributeFuncDestination> for Option<si_events::PropId> {
    fn from(value: AttributeFuncDestination) -> Self {
        match value {
            AttributeFuncDestination::Prop(prop_id) => Some(prop_id),
            _ => None,
        }
    }
}

impl From<AttributeFuncDestination> for Option<si_events::OutputSocketId> {
    fn from(value: AttributeFuncDestination) -> Self {
        match value {
            AttributeFuncDestination::OutputSocket(output_socket_id) => Some(output_socket_id),
            _ => None,
        }
    }
}

impl From<FuncBinding> for si_frontend_types::FuncBinding {
    fn from(value: FuncBinding) -> Self {
        match value {
            FuncBinding::Action(action) => si_frontend_types::FuncBinding::Action {
                schema_variant_id: Some(action.schema_variant_id),
                action_prototype_id: Some(action.action_prototype_id),
                func_id: Some(action.func_id),
                kind: Some(action.kind.into()),
            },
            FuncBinding::Attribute(attribute) => si_frontend_types::FuncBinding::Attribute {
                func_id: Some(attribute.func_id),
                attribute_prototype_id: Some(attribute.attribute_prototype_id),
                component_id: (&attribute.eventual_parent).into(),
                schema_variant_id: (&attribute.eventual_parent).into(),
                prop_id: attribute.output_location.into(),
                output_socket_id: attribute.output_location.into(),
                argument_bindings: attribute
                    .argument_bindings
                    .into_iter()
                    .map(|f| f.into_frontend_type())
                    .collect_vec(),
            },
            FuncBinding::Authentication(auth) => si_frontend_types::FuncBinding::Authentication {
                schema_variant_id: auth.schema_variant_id,
                func_id: Some(auth.func_id),
            },
            FuncBinding::Management(mgmt) => si_frontend_types::FuncBinding::Management {
                schema_ids: mgmt.schema_ids,
                schema_variant_id: mgmt.schema_variant_id,
                management_prototype_id: Some(mgmt.management_prototype_id),
                func_id: Some(mgmt.func_id),
            },
            FuncBinding::CodeGeneration(code_gen) => {
                si_frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id: (&code_gen.eventual_parent).into(),
                    component_id: (&code_gen.eventual_parent).into(),
                    func_id: Some(code_gen.func_id),
                    attribute_prototype_id: code_gen.leaf_binding_prototype.into(),
                    leaf_prototype_id: code_gen.leaf_binding_prototype.into(),
                    inputs: code_gen
                        .inputs
                        .into_iter()
                        .map(|input| input.into())
                        .collect_vec(),
                }
            }
            FuncBinding::Qualification(qualification) => {
                si_frontend_types::FuncBinding::Qualification {
                    schema_variant_id: (&qualification.eventual_parent).into(),
                    component_id: (&qualification.eventual_parent).into(),
                    func_id: Some(qualification.func_id),
                    attribute_prototype_id: qualification.leaf_binding_prototype.into(),
                    leaf_prototype_id: qualification.leaf_binding_prototype.into(),
                    inputs: qualification
                        .inputs
                        .into_iter()
                        .map(|input| input.into())
                        .collect_vec(),
                }
            }
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
    Action(ActionBinding),
    /// An Attribute function is a function that sets values within a [`SchemaVariant`] or [`Component`]'s Prop Tree
    /// Each Attribute Function has user defined arguments, configured to map to specific Props or InputSockets.
    /// This intersection of [`FuncArgument`] and the [`Prop`] or [`InputSocket`] mapping is an [`AttributeArgumentBinding`]
    Attribute(AttributeBinding),
    /// Auth Funcs only exist on Secret defining schema variants, and have no special configuration data aside from the
    /// [`SchemaVariantId`] and as such are only created or deleted (detached), they are not updated.
    Authentication(AuthBinding),
    /// CodeGen funcs are ultimately just an Attribute Function, but the user can not control where they output to.
    /// They write to an Attribute Value beneath the Code Gen Root Prop Node
    CodeGeneration(LeafBinding),
    Management(ManagementBinding),
    /// Qualification funcs are ultimately just an Attribute Function, but the user can not control where they output to.
    /// They write to an Attribute Value beneath the Qualification Root Prop Node
    Qualification(LeafBinding),
}

impl FuncBinding {
    /// Takes the [`self`] [`FuncBinding`] and a provided [`FuncId`], deletes the existing binding and recreates it for the
    /// given [`FuncId`].
    #[instrument(
        level = "info",
        skip(self, ctx),
        name = "func.bindings.port_binding_to_new_func"
    )]
    pub async fn port_binding_to_new_func(
        &self,
        ctx: &DalContext,
        new_func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let new_binding = match self {
            FuncBinding::Action(action) => {
                action.port_binding_to_new_func(ctx, new_func_id).await?
            }
            FuncBinding::Attribute(attribute) => {
                attribute.port_binding_to_new_func(ctx, new_func_id).await?
            }
            FuncBinding::Authentication(auth) => {
                auth.port_binding_to_new_func(ctx, new_func_id).await?
            }
            FuncBinding::CodeGeneration(code_gen) => {
                LeafBinding::port_binding_to_new_func(
                    ctx,
                    new_func_id,
                    code_gen.leaf_binding_prototype,
                    LeafKind::CodeGeneration,
                    code_gen.eventual_parent.clone(),
                    &code_gen.inputs,
                )
                .await?
            }
            FuncBinding::Qualification(qualification) => {
                LeafBinding::port_binding_to_new_func(
                    ctx,
                    new_func_id,
                    qualification.leaf_binding_prototype,
                    LeafKind::Qualification,
                    qualification.eventual_parent.clone(),
                    &qualification.inputs,
                )
                .await?
            }
            FuncBinding::Management(mgmt) => {
                mgmt.port_binding_to_new_func(ctx, new_func_id).await?
            }
        };
        Ok(new_binding)
    }

    pub fn get_schema_variant(&self) -> Option<SchemaVariantId> {
        match self {
            FuncBinding::Action(action) => Some(action.schema_variant_id),
            FuncBinding::Attribute(attribute) => {
                if let EventualParent::SchemaVariant(schema_variant_id) = attribute.eventual_parent
                {
                    Some(schema_variant_id)
                } else {
                    None
                }
            }
            FuncBinding::Authentication(auth) => Some(auth.schema_variant_id),
            FuncBinding::CodeGeneration(code_gen) => {
                if let EventualParent::SchemaVariant(schema_variant_id) = code_gen.eventual_parent {
                    Some(schema_variant_id)
                } else {
                    None
                }
            }
            FuncBinding::Qualification(qualification) => {
                if let EventualParent::SchemaVariant(schema_variant_id) =
                    qualification.eventual_parent
                {
                    Some(schema_variant_id)
                } else {
                    None
                }
            }
            FuncBinding::Management(mgmt) => mgmt.schema_variant_id,
        }
    }

    pub async fn for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let func = Func::get_by_id(ctx, func_id).await?;
        let bindings = match func.kind {
            FuncKind::Action => ActionBinding::assemble_action_bindings(ctx, func_id).await?,
            FuncKind::Attribute => {
                AttributeBinding::assemble_attribute_bindings(ctx, func_id).await?
            }
            FuncKind::Authentication => AuthBinding::assemble_auth_bindings(ctx, func_id).await?,
            FuncKind::CodeGeneration => {
                LeafBinding::assemble_leaf_func_bindings(ctx, func_id, LeafKind::CodeGeneration)
                    .await?
            }
            FuncKind::Qualification => {
                LeafBinding::assemble_leaf_func_bindings(ctx, func_id, LeafKind::Qualification)
                    .await?
            }
            FuncKind::Intrinsic => {
                AttributeBinding::assemble_intrinsic_bindings(ctx, func_id).await?
            }
            FuncKind::SchemaVariantDefinition | FuncKind::Unknown => vec![],
            FuncKind::Management => {
                ManagementBinding::assemble_management_bindings(ctx, func_id).await?
            }
        };
        Ok(bindings)
    }

    /// Returns a pruned set of bindings, where each binding is for the default variant, or if an unlocked variant exists for the schema,
    /// only return that one
    pub async fn get_bindings_for_default_and_unlocked_schema_variants(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<(SchemaVariantId, FuncBinding)>> {
        let mut schema_variant_map: HashMap<SchemaId, (SchemaVariantId, FuncBinding)> =
            HashMap::new();
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            if let Some(schema_variant_id) = binding.get_schema_variant() {
                // check the schema for this variant
                let schema_id = SchemaVariant::schema_id(ctx, schema_variant_id).await?;
                let maybe_variant = schema_variant_map.get(&schema_id);
                match maybe_variant {
                    Some(_) => {
                        // if there's a thing in here, it might be the default one. replace if this one's unlocked
                        if !SchemaVariant::is_locked_by_id(ctx, schema_variant_id).await? {
                            schema_variant_map.insert(schema_id, (schema_variant_id, binding));
                        }
                    }
                    None => {
                        if !SchemaVariant::is_locked_by_id(ctx, schema_variant_id).await?
                            || SchemaVariant::get_by_id(ctx, schema_variant_id)
                                .await?
                                .is_default(ctx)
                                .await?
                        {
                            schema_variant_map.insert(schema_id, (schema_variant_id, binding));
                        }
                    }
                }
            }
        }

        Ok(schema_variant_map.into_values().collect_vec())
    }

    /// Prunes the list of bindings for a [`FuncId`] to only include the bindings for the "latest" [`SchemaVariant`]s
    pub async fn get_bindings_for_default_schema_variants(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut pruned_bindings = vec![];
        let mut schema_default_map: HashMap<SchemaId, SchemaVariantId> = HashMap::new();
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            if let Some(schema_variant_id) = binding.get_schema_variant() {
                // check the schema for this variant
                let schema = SchemaVariant::schema_id(ctx, schema_variant_id).await?;
                // check the map
                let maybe_default_for_variant = schema_default_map.get_key_value(&schema);
                match maybe_default_for_variant {
                    Some((_schema_id, default_schema_variant_id)) => {
                        if schema_variant_id == *default_schema_variant_id {
                            pruned_bindings.push(binding);
                        }
                    }
                    None => {
                        let default_for_schema =
                            SchemaVariant::default_id_for_schema(ctx, schema).await?;

                        schema_default_map.insert(schema, default_for_schema);

                        if default_for_schema == schema_variant_id {
                            pruned_bindings.push(binding);
                        }
                    }
                }
            }
        }
        Ok(pruned_bindings)
    }

    /// Prunes the list of bindings for a [`FuncId`] to only include the latest, unlocked [`SchemaVariant`]s
    pub async fn get_bindings_for_unlocked_schema_variants(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut pruned_bindings = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            if let Some(schema_variant_id) = binding.get_schema_variant() {
                if !SchemaVariant::is_locked_by_id(ctx, schema_variant_id).await? {
                    pruned_bindings.push(binding);
                }
            }
        }
        Ok(pruned_bindings)
    }

    /// Returns all bindings for a [`FuncId`] for a given [`SchemaVariantId`]
    pub async fn get_bindings_for_schema_variant_id(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut schema_variant_bindings = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            if let Some(current_schema_variant_id) = binding.get_schema_variant() {
                if current_schema_variant_id == schema_variant_id {
                    schema_variant_bindings.push(binding);
                }
            }
        }
        Ok(schema_variant_bindings)
    }

    /// Removes all existing bindings for a given [`FuncId`]
    /// Returns an error if any of the bindings are for a locked [`SchemaVariant`]
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.delete_all_bindings_for_func_id"
    )]
    pub async fn delete_all_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<()> {
        let func_bindings = FuncBinding::for_func_id(ctx, func_id).await?;
        for binding in func_bindings {
            if let Some(sv_id) = binding.get_schema_variant() {
                let sv = SchemaVariant::get_by_id(ctx, sv_id).await?;

                // If the variant is locked, not default and was created on this changeset, it needs to be garbage collected
                if sv.is_locked()
                    && !sv.is_default(ctx).await?
                    && sv.was_created_on_this_changeset(ctx).await?
                {
                    SchemaVariant::cleanup_variant(ctx, sv).await?;
                    // Don't delete the binding directly, this will be dealt with by the cleanup at the end
                    continue;
                }
            }

            match binding {
                FuncBinding::Action(action) => {
                    ActionBinding::delete_action_binding(ctx, action.action_prototype_id).await?;
                }
                FuncBinding::Attribute(attribute) => {
                    AttributeBinding::reset_attribute_binding(
                        ctx,
                        attribute.attribute_prototype_id,
                    )
                    .await?;
                }
                FuncBinding::Authentication(auth) => {
                    AuthBinding::delete_auth_binding(ctx, auth.func_id, auth.schema_variant_id)
                        .await?;
                }
                FuncBinding::CodeGeneration(binding) | FuncBinding::Qualification(binding) => {
                    match binding.leaf_binding_prototype {
                        LeafBindingPrototype::Attribute(attribute_prototype_id) => {
                            LeafBinding::delete_leaf_func_binding(ctx, attribute_prototype_id)
                                .await?;
                        }
                        LeafBindingPrototype::Overlay(leaf_prototype_id) => {
                            LeafBinding::delete_leaf_overlay_func_binding(ctx, leaf_prototype_id)
                                .await?;
                        }
                    }
                }
                FuncBinding::Management(mgmt) => {
                    ManagementBinding::delete_management_binding(ctx, mgmt.management_prototype_id)
                        .await?;
                }
            };
        }

        // Remove the bindings from the garbage collected schema variants
        ctx.workspace_snapshot()?.cleanup().await?;

        Ok(())
    }

    /// Compile all the types for all of the bindings to return to the front end for type checking
    pub async fn compile_types(ctx: &DalContext, func_id: FuncId) -> FuncBindingResult<String> {
        let func = Func::get_by_id(ctx, func_id).await?;
        let types: String = match func.kind {
            FuncKind::Action => ActionBinding::compile_action_types(ctx, func_id).await?,
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                LeafBinding::compile_leaf_func_types(ctx, func_id).await?
            }
            FuncKind::Attribute => AttributeBinding::compile_attribute_types(ctx, func_id).await?,
            FuncKind::Management => {
                ManagementBinding::compile_management_types(ctx, func_id).await?
            }
            FuncKind::Authentication
            | FuncKind::Intrinsic
            | FuncKind::SchemaVariantDefinition
            | FuncKind::Unknown => String::new(),
        };
        Ok(types)
    }

    /// For a given [`FuncId`], if it's an Attribute Func, returns the [`AttributeBinding`]s
    pub async fn get_attribute_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<AttributeBinding>> {
        let mut attributes = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            match binding {
                FuncBinding::Attribute(attribute) => {
                    attributes.push(attribute);
                }
                other_binding => {
                    return Err(FuncBindingError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Authentication,
                    ));
                }
            }
        }
        Ok(attributes)
    }

    /// For a given [`FuncId`], if it's a CodeGen Func, returns the [`LeafBinding`]s
    pub async fn get_code_gen_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<LeafBinding>> {
        let mut leaves = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            match binding {
                FuncBinding::CodeGeneration(leaf_binding) => {
                    leaves.push(leaf_binding);
                }
                other_binding => {
                    return Err(FuncBindingError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Authentication,
                    ));
                }
            }
        }
        Ok(leaves)
    }

    /// For a given [`FuncId`], if it's a Qualification Func, returns the [`LeafBinding`]s
    pub async fn get_qualification_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<LeafBinding>> {
        let mut leaves = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            match binding {
                FuncBinding::Qualification(leaf_binding) => {
                    leaves.push(leaf_binding);
                }
                other_binding => {
                    return Err(FuncBindingError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Authentication,
                    ));
                }
            }
        }
        Ok(leaves)
    }

    /// For a given [`FuncId`], if it's a Action Func, returns the [`ActionBinding`]s
    pub async fn get_action_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<ActionBinding>> {
        let mut actions = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            match binding {
                FuncBinding::Action(action_binding) => {
                    actions.push(action_binding);
                }
                other_binding => {
                    return Err(FuncBindingError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Authentication,
                    ));
                }
            }
        }
        Ok(actions)
    }
    /// For a given [`FuncId`], if it's a Auth Func, returns the [`AuthBinding`]s
    pub async fn get_auth_bindings_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<AuthBinding>> {
        let mut auth_bindings = vec![];
        let bindings = Self::for_func_id(ctx, func_id).await?;
        for binding in bindings {
            match binding {
                FuncBinding::Authentication(auth_binding) => {
                    auth_bindings.push(auth_binding);
                }
                other_binding => {
                    return Err(FuncBindingError::UnexpectedFuncBindingVariant(
                        other_binding.into(),
                        FuncBindingDiscriminants::Authentication,
                    ));
                }
            }
        }
        Ok(auth_bindings)
    }
}

impl From<ManagementPrototypeError> for FuncBindingError {
    fn from(value: ManagementPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for FuncBindingError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for FuncBindingError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for FuncBindingError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for FuncBindingError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

impl From<ActionPrototypeError> for FuncBindingError {
    fn from(value: ActionPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for FuncBindingError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for FuncBindingError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for FuncBindingError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncArgumentError> for FuncBindingError {
    fn from(value: FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for FuncBindingError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for FuncBindingError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for FuncBindingError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaError> for FuncBindingError {
    fn from(value: SchemaError) -> Self {
        Box::new(value).into()
    }
}
