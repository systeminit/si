//! An [`AttributePrototype`] represents, for a specific attribute:
//!
//!   * Which context the following applies to ([`AttributeContext`](crate::AttributeContext))
//!   * The function that should be run to find its value.
//!   * In the case that the [`Prop`](crate::Prop) is the child of an
//!     [`Array`](crate::prop::PropKind::Array): Which index in the `Array` the value
//!     is for.
//!   * In the case that the [`Prop`](crate::Prop) is the child of a
//!     [`Map`](crate::prop::PropKind::Map): Which key of the `Map` the value is
//!     for.

use std::sync::Arc;

use argument::AttributePrototypeArgumentError;
use content_node_weight::ContentNodeWeight;
use petgraph::Direction;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    Timestamp,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributeValue,
    AttributeValueId,
    ComponentId,
    DalContext,
    Func,
    FuncError,
    FuncId,
    HelperError,
    InputSocketId,
    OutputSocketId,
    PropId,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    TransactionsError,
    attribute::{
        prototype::argument::{
            AttributePrototypeArgument,
            AttributePrototypeArgumentId,
            value_source::ValueSource,
        },
        value::AttributeValueError,
    },
    change_set::ChangeSetError,
    func::{
        argument::FuncArgument,
        intrinsics::IntrinsicFunc,
    },
    implement_add_edge_to,
    layer_db_types::{
        AttributePrototypeContent,
        AttributePrototypeContentV1,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        node_weight::{
            NodeWeight,
            NodeWeightDiscriminants,
            NodeWeightError,
            content_node_weight,
            traits::SiNodeWeight,
        },
        traits::attribute_prototype::AttributePrototypeExt as _,
    },
};

pub mod argument;
pub mod debug;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func error: {0}")]
    FuncArgument(#[from] Box<crate::func::argument::FuncArgumentError>),
    #[error("helper error: {0}")]
    Helper(#[from] Box<HelperError>),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("attribute prototype {0} is missing a function edge")]
    MissingFunction(AttributePrototypeId),
    #[error("Attribute prototype {0} has multiple functions")]
    MultipleFunctionsFound(AttributePrototypeId),
    #[error("no arguments to identity func at {0}")]
    NoArgumentsToIdentityFunction(AttributePrototypeId),
    #[error("No attribute values for: {0}")]
    NoAttributeValues(AttributePrototypeId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("{0} is not identity func")]
    NonIdentityFunc(FuncId),
    #[error("Attribute Prototype not found: {0}")]
    NotFound(AttributePrototypeId),
    #[error("attribute prototype has been orphaned: {0}")]
    Orphaned(AttributePrototypeId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("unexpected node ({0}) using attribute prototype ({1})")]
    UnexpectedNodeUsingAttributePrototype(Ulid, AttributePrototypeId),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributePrototypeResult<T> = Result<T, AttributePrototypeError>;

impl From<AttributePrototypeArgumentError> for AttributePrototypeError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for AttributePrototypeError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for AttributePrototypeError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::func::argument::FuncArgumentError> for AttributePrototypeError {
    fn from(value: crate::func::argument::FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<HelperError> for AttributePrototypeError {
    fn from(value: HelperError) -> Self {
        Box::new(value).into()
    }
}

/// Indicates the _one and only one_ eventual parent of a corresponding [`AttributePrototype`].
///
/// - If an [`AttributePrototype`] is used by an [`AttributeValue`], its eventual parent is a
///   [`Component`](crate::Component).
/// - If an [`AttributePrototype`] is used by a [`Prop`](crate::Prop), an
///   [`InputSocket`](crate::InputSocket), or an [`OutputSocket`](crate::OutputSocket), its eventual
///   parent is a [`SchemaVariant`].
#[remain::sorted]
#[derive(Debug, Clone, Copy, EnumDiscriminants)]
pub enum AttributePrototypeEventualParent {
    Component(ComponentId, AttributeValueId),
    SchemaVariantFromInputSocket(SchemaVariantId, InputSocketId),
    SchemaVariantFromOutputSocket(SchemaVariantId, OutputSocketId),
    SchemaVariantFromProp(SchemaVariantId, PropId),
}

pub use si_id::AttributePrototypeId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototype {
    pub id: AttributePrototypeId,
    pub timestamp: Timestamp,
}

impl AttributePrototype {
    pub fn assemble(id: AttributePrototypeId, inner: &AttributePrototypeContentV1) -> Self {
        let inner: AttributePrototypeContentV1 = inner.to_owned();
        Self {
            id,
            timestamp: inner.timestamp,
        }
    }

    implement_add_edge_to!(
        source_id: AttributePrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: AttributePrototypeResult,
    );
    implement_add_edge_to!(
        source_id: AttributePrototypeId,
        destination_id: AttributePrototypeArgumentId,
        add_fn: add_edge_to_argument,
        discriminant: EdgeWeightKindDiscriminants::PrototypeArgument,
        result: AttributePrototypeResult,
    );

    pub fn id(&self) -> AttributePrototypeId {
        self.id
    }

    pub async fn new(ctx: &DalContext, func_id: FuncId) -> AttributePrototypeResult<Self> {
        let timestamp = Timestamp::now();

        let content = AttributePrototypeContentV1 { timestamp };
        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(AttributePrototypeContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight =
            NodeWeight::new_content(id, lineage_id, ContentAddress::AttributePrototype(hash));
        workspace_snapshot.add_or_replace_node(node_weight).await?;

        let prototype = AttributePrototype::assemble(id.into(), &content);

        Self::add_edge_to_func(ctx, prototype.id, func_id, EdgeWeightKind::new_use()).await?;

        Ok(prototype)
    }

    pub async fn func_id(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<FuncId> {
        ctx.workspace_snapshot()?
            .attribute_prototype_func_id(prototype_id)
            .await
    }

    pub async fn func(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Func> {
        Ok(Func::get_by_id(ctx, Self::func_id(ctx, prototype_id).await?).await?)
    }

    pub async fn is_dynamic(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<bool> {
        let func_id = Self::func_id(ctx, prototype_id).await?;
        Ok(Func::is_dynamic(ctx, func_id).await?)
    }

    pub async fn find_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
        key: &Option<String>,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(prop_id, Direction::Outgoing)
            .await?
            .into_iter()
            .find(|(edge_weight, _, _)| {
                if let EdgeWeightKind::Prototype(maybe_key) = edge_weight.kind() {
                    maybe_key == key
                } else {
                    false
                }
            })
            .map(|(_, _, target_idx)| target_idx)
        {
            let node_weight = workspace_snapshot.get_node_weight(prototype_idx).await?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn find_for_output_socket(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(output_socket_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| {
                EdgeWeightKindDiscriminants::Prototype == edge_weight.kind().into()
            })
            .map(|(_, _, target_idx)| target_idx)
        {
            let node_weight = workspace_snapshot.get_node_weight(*prototype_idx).await?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn find_for_input_socket(
        ctx: &DalContext,
        input_socket_id: InputSocketId,
    ) -> AttributePrototypeResult<Option<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(prototype_idx) = workspace_snapshot
            .edges_directed(input_socket_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| {
                EdgeWeightKindDiscriminants::Prototype == edge_weight.kind().into()
            })
            .map(|(_, _, target_idx)| target_idx)
        {
            let node_weight = workspace_snapshot.get_node_weight(*prototype_idx).await?;

            if matches!(
                node_weight.content_address_discriminants(),
                Some(ContentAddressDiscriminants::AttributePrototype)
            ) {
                return Ok(Some(node_weight.id().into()));
            }
        }

        Ok(None)
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Self> {
        let (_node_weight, content) = Self::get_node_weight_and_content(ctx, prototype_id).await?;
        Ok(Self::assemble(prototype_id, &content))
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<(ContentNodeWeight, AttributePrototypeContentV1)> {
        let content_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(prototype_id)
            .await?;
        let prototype_node_weight = content_weight
            .get_content_node_weight_of_kind(ContentAddressDiscriminants::AttributePrototype)?;

        let content: AttributePrototypeContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&prototype_node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                prototype_id.into(),
            ))?;

        // Do "upgrading" of the storage format from old versions to the latest here.
        let AttributePrototypeContent::V1(inner) = content;

        Ok((prototype_node_weight, inner))
    }

    pub async fn update_func_by_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_id: FuncId,
    ) -> AttributePrototypeResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let current_func_node_id = workspace_snapshot
            .edges_directed(attribute_prototype_id, Direction::Outgoing)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| edge_weight.kind() == &EdgeWeightKind::new_use())
            .map(|(_, _, target_idx)| *target_idx)
            .ok_or(AttributePrototypeError::MissingFunction(
                attribute_prototype_id,
            ))?;

        workspace_snapshot
            .remove_edge(
                attribute_prototype_id,
                current_func_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        Self::add_edge_to_func(
            ctx,
            attribute_prototype_id,
            func_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(())
    }

    pub async fn attribute_value_ids(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributeValueId>> {
        if let Some(attribute_value_id) =
            Self::attribute_value_id(ctx, attribute_prototype_id).await?
        {
            return Ok(vec![attribute_value_id]);
        }

        // Remaining edges
        // prototype <-- Prototype -- (Prop | Socket) <-- Prop|Socket -- Attribute Values
        // (multiple avs possible)

        let mut attribute_value_ids = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        for prototype_edge_source in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?
        {
            let (target_id, edge_weight_discrim) = match workspace_snapshot
                .get_node_weight(prototype_edge_source)
                .await?
            {
                NodeWeight::Prop(prop_inner) => {
                    (prop_inner.id(), EdgeWeightKindDiscriminants::Prop)
                }
                NodeWeight::Content(content_inner) => match content_inner.content_address() {
                    ContentAddress::OutputSocket(_) | ContentAddress::InputSocket(_) => {
                        (content_inner.id(), EdgeWeightKindDiscriminants::Socket)
                    }
                    _ => {
                        return Err(WorkspaceSnapshotError::UnexpectedEdgeSource(
                            content_inner.id(),
                            attribute_prototype_id.into(),
                            EdgeWeightKindDiscriminants::Prototype,
                        )
                        .into());
                    }
                },
                NodeWeight::InputSocket(input_socket) => {
                    (input_socket.id(), EdgeWeightKindDiscriminants::Socket)
                }
                other => {
                    return Err(WorkspaceSnapshotError::UnexpectedEdgeSource(
                        other.id(),
                        attribute_prototype_id.into(),
                        EdgeWeightKindDiscriminants::Prototype,
                    )
                    .into());
                }
            };

            for attribute_value_target in workspace_snapshot
                .incoming_sources_for_edge_weight_kind(target_id, edge_weight_discrim)
                .await?
            {
                if let NodeWeight::AttributeValue(av_node_weight) = workspace_snapshot
                    .get_node_weight(attribute_value_target)
                    .await?
                {
                    attribute_value_ids.push(av_node_weight.id().into())
                }
            }
        }

        Ok(attribute_value_ids)
    }

    /// Adds a new APA with the given source, to the single function argument (used for intrinsics)
    pub async fn add_arg_to_intrinsic(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        value_source: impl Into<ValueSource>,
    ) -> AttributePrototypeResult<AttributePrototypeArgumentId> {
        let func_id = Self::func_id(ctx, attribute_prototype_id).await?;
        let arg_id = FuncArgument::single_arg_for_func(ctx, func_id).await?;
        Ok(
            AttributePrototypeArgument::new(ctx, attribute_prototype_id, arg_id, value_source)
                .await?
                .id(),
        )
    }

    // If this is a prototype for a prop, returns the PropId. Otherwise, returns None.
    pub async fn prop_id(
        ctx: &DalContext,
        apa_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Option<PropId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        if let Some(for_id) = workspace_snapshot
            .source_opt(apa_id, EdgeWeightKindDiscriminants::Prototype)
            .await?
        {
            if let NodeWeight::Prop(node) = workspace_snapshot.get_node_weight(for_id).await? {
                return Ok(Some(node.id().into()));
            }
        }

        Ok(None)
    }

    /// If this prototype is defined at the component level, it will have an incoming edge from the
    /// AttributeValue for which it is the prototype. Otherwise this will return None, indicating a
    /// prototype defined at the schema variant level (which has no attribute value)
    pub async fn attribute_value_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Option<AttributeValueId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let maybe_value_idxs = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                attribute_prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?;

        if maybe_value_idxs.len() > 1 {
            return Err(WorkspaceSnapshotError::UnexpectedNumberOfIncomingEdges(
                EdgeWeightKindDiscriminants::Prototype,
                NodeWeightDiscriminants::Content,
                attribute_prototype_id.into(),
            )
            .into());
        }

        Ok(match maybe_value_idxs.first().copied() {
            Some(value_idx) => {
                if let NodeWeight::AttributeValue(av_node_weight) =
                    workspace_snapshot.get_node_weight(value_idx).await?
                {
                    Some(av_node_weight.id().into())
                } else {
                    None
                }
            }
            None => None,
        })
    }

    pub async fn remove(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<()> {
        // Also remove all apas that use this prototype, to prevent hanging APAs
        // during subscription creation
        let apa_ids = AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        for apa_id in apa_ids {
            AttributePrototypeArgument::remove(ctx, apa_id).await?;
        }

        ctx.workspace_snapshot()?
            .remove_node_by_id(prototype_id)
            .await?;

        Ok(())
    }

    pub async fn list_input_socket_sources_for_id(
        ctx: &DalContext,
        ap_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<InputSocketId>> {
        let apa_ids = AttributePrototypeArgument::list_ids_for_prototype(ctx, ap_id).await?;

        let mut input_socket_ids = Vec::<InputSocketId>::with_capacity(apa_ids.len());
        for apa_id in apa_ids {
            let maybe_value_source =
                AttributePrototypeArgument::value_source_opt(ctx, apa_id).await?;
            if let Some(ValueSource::InputSocket(socket_id)) = maybe_value_source {
                input_socket_ids.push(socket_id);
            }
        }

        Ok(input_socket_ids)
    }

    pub async fn list_ids_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let prototype_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?;
        let mut attribute_prototype_argument_ids = Vec::with_capacity(prototype_sources.len());
        for node_index in prototype_sources {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let node_weight_id = node_weight.id();
            if let Some(ContentAddressDiscriminants::AttributePrototype) =
                node_weight.content_address_discriminants()
            {
                attribute_prototype_argument_ids.push(node_weight_id.into());
            }
        }

        Ok(attribute_prototype_argument_ids)
    }

    /// Returns the [eventual parent](AttributePrototypeEventualParent) of the
    /// [`AttributePrototype`], which will either be a [`Component`](crate::Component) or a
    /// [`SchemaVariant`].
    pub async fn eventual_parent(
        ctx: &DalContext,
        id: AttributePrototypeId,
    ) -> AttributePrototypeResult<AttributePrototypeEventualParent> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let source = *workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Prototype)
            .await?
            .first()
            .ok_or(AttributePrototypeError::Orphaned(id))?;

        let node_weight = workspace_snapshot.get_node_weight(source).await?;
        let node_weight_id = node_weight.id();

        let eventual_parent = match node_weight {
            NodeWeight::AttributeValue(attribute_value_id) => {
                AttributePrototypeEventualParent::Component(
                    AttributeValue::component_id(ctx, node_weight_id.into())
                        .await
                        .map_err(Box::new)?,
                    attribute_value_id.id().into(),
                )
            }
            NodeWeight::Prop(_) => AttributePrototypeEventualParent::SchemaVariantFromProp(
                SchemaVariant::find_for_prop_id(ctx, node_weight_id.into())
                    .await
                    .map_err(Box::new)?,
                node_weight_id.into(),
            ),
            NodeWeight::InputSocket(_) => {
                AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                    SchemaVariant::find_for_input_socket_id(ctx, node_weight_id.into())
                        .await
                        .map_err(Box::new)?,
                    node_weight_id.into(),
                )
            }
            NodeWeight::Content(inner) => match inner.content_address().into() {
                ContentAddressDiscriminants::InputSocket => {
                    AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                        SchemaVariant::find_for_input_socket_id(ctx, node_weight_id.into())
                            .await
                            .map_err(Box::new)?,
                        node_weight_id.into(),
                    )
                }
                ContentAddressDiscriminants::OutputSocket => {
                    AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                        SchemaVariant::find_for_output_socket_id(ctx, node_weight_id.into())
                            .await
                            .map_err(Box::new)?,
                        node_weight_id.into(),
                    )
                }
                _ => {
                    return Err(
                        AttributePrototypeError::UnexpectedNodeUsingAttributePrototype(
                            node_weight_id,
                            id,
                        ),
                    );
                }
            },
            _ => {
                return Err(
                    AttributePrototypeError::UnexpectedNodeUsingAttributePrototype(
                        node_weight_id,
                        id,
                    ),
                );
            }
        };

        Ok(eventual_parent)
    }

    pub async fn list_arguments(
        ctx: &DalContext,
        ap_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Vec<AttributePrototypeArgumentId>> {
        ctx.workspace_snapshot()?
            .attribute_prototype_arguments(ap_id)
            .await
    }

    // Returns the argument to this identity (or unset) function call.
    // Errors if it is neither identity nor unset.
    pub async fn identity_or_unset_argument(
        ctx: &DalContext,
        ap_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<Option<AttributePrototypeArgumentId>> {
        let func = Self::func(ctx, ap_id).await?;
        if func.name != IntrinsicFunc::Identity.name() {
            if func.name == IntrinsicFunc::Unset.name() {
                return Ok(None);
            }
            return Err(AttributePrototypeError::NonIdentityFunc(func.id));
        }
        let args = AttributePrototype::list_arguments(ctx, ap_id).await?;
        let arg_id = args
            .first()
            .ok_or(AttributePrototypeError::NoArgumentsToIdentityFunction(
                ap_id,
            ))?;
        Ok(Some(*arg_id))
    }

    /// Get a short, human-readable title suitable for debugging/display.
    /// Pass component_id if this is a prototype on the schema and you want to print only
    /// values for the given component.
    pub async fn fmt_title(ctx: &DalContext, ap_id: AttributePrototypeId) -> String {
        Self::fmt_title_fallible(ctx, ap_id)
            .await
            .unwrap_or_else(|e| e.to_string())
    }

    pub async fn fmt_title_fallible(
        ctx: &DalContext,
        ap_id: AttributePrototypeId,
    ) -> AttributePrototypeResult<String> {
        let func_id = Self::func_id(ctx, ap_id).await?;
        let args = AttributePrototype::list_arguments(ctx, ap_id).await?;

        // Special case some very common intrinsics
        if args.len() == 1 {
            if let Some(&apa_id) = args.first() {
                let intrinsic = Func::intrinsic_kind(ctx, func_id).await?;
                let value_source =
                    AttributePrototypeArgument::value_source_opt(ctx, apa_id).await?;
                let omit_function = match (intrinsic, value_source) {
                    // si:setObject({}), si:setString(<string>), si:setNumber(<number>), etc.
                    // just print the value
                    (Some(intrinsic), Some(ValueSource::StaticArgumentValue(..))) => {
                        intrinsic.set_func().is_some()
                    }
                    // si:identity(<subscription>) is just printed as <subscription>
                    (Some(IntrinsicFunc::Identity), Some(ValueSource::ValueSubscription(..))) => {
                        true
                    }
                    _ => false,
                };
                if omit_function {
                    return Ok(AttributePrototypeArgument::fmt_title(ctx, apa_id).await);
                }
            }
        }

        // Print <function>(<args>)
        let mut title = Func::fmt_title(ctx, func_id).await;
        title.push('(');
        let mut is_first = true;
        for apa_id in args {
            // Print commas between arguments.
            if !is_first {
                title.push_str(", ");
            }
            is_first = false;
            title.push_str(&AttributePrototypeArgument::fmt_title(ctx, apa_id).await);
        }
        title.push(')');
        Ok(title)
    }
}
