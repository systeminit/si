//! An [`AttributePrototypeArgument`] joins a prototype to a function argument
//! and to either the input socket that supplies its value or to a constant
//! value. It defines source of the value for the function argument in the
//! context of the prototype.

use std::collections::HashSet;

use petgraph::Direction;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    Timestamp,
    ulid::Ulid,
};
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
    ComponentId,
    FuncArgumentId,
    OutputSocketId,
    StaticArgumentValueId,
};
use static_value::StaticArgumentValue;
use telemetry::prelude::*;
use thiserror::Error;
use value_source::ValueSource;

use super::AttributePrototypeError;
use crate::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    HelperError,
    TransactionsError,
    attribute::value::subscription::ValueSubscription,
    change_set::ChangeSetError,
    func::argument::{
        FuncArgument,
        FuncArgumentError,
    },
    implement_add_edge_to,
    workspace_snapshot::{
        WorkspaceSnapshotError,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        dependent_value_root::DependentValueRootError,
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::WorkspaceSnapshotGraphError,
        node_weight::{
            ArgumentTargets,
            AttributePrototypeArgumentNodeWeight,
            NodeWeight,
            NodeWeightDiscriminants,
            NodeWeightError,
            reason_node_weight::Reason,
            traits::SiNodeWeight as _,
        },
    },
};

pub mod static_value;
pub mod value_source;

pub use si_id::AttributePrototypeArgumentId;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] crate::attribute::value::AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] crate::ComponentError),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("Destination prototype {0} has no function arguments")]
    InterComponentDestinationPrototypeHasNoFuncArgs(AttributePrototypeId),
    #[error("Destination prototype {0} has more than one function argument")]
    InterComponentDestinationPrototypeHasTooManyFuncArgs(AttributePrototypeId),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("attribute prototype argument {0} has no func argument")]
    MissingFuncArgument(AttributePrototypeArgumentId),
    #[error("attribute prototype argument {0} has no value source")]
    MissingValueSource(AttributePrototypeArgumentId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("no targets for prototype argument: {0}")]
    NoTargets(AttributePrototypeArgumentId),
    #[error("prototype argument not found for attribute prototype {0} and func arg {1}")]
    NotFoundForApAndFuncArg(AttributePrototypeId, FuncArgumentId),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error(
        "PrototypeArgument {0} value edge pointing to unexpected content node weight kind: {1:?}"
    )]
    UnexpectedValueSourceContent(AttributePrototypeArgumentId, ContentAddressDiscriminants),
    #[error("PrototypeArgument {0} value edge pointing to EdgeWeightKindDiscriminants kind: {1:?}")]
    UnexpectedValueSourceNode(AttributePrototypeArgumentId, EdgeWeightKindDiscriminants),
    #[error("value source error: {0}")]
    ValueSource(#[from] value_source::ValueSourceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributePrototypeArgumentResult<T> = Result<T, AttributePrototypeArgumentError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototypeArgument {
    id: AttributePrototypeArgumentId,
    targets: Option<ArgumentTargets>,
    timestamp: Timestamp,
}

impl From<AttributePrototypeArgumentNodeWeight> for AttributePrototypeArgument {
    fn from(value: AttributePrototypeArgumentNodeWeight) -> Self {
        Self {
            timestamp: value.timestamp().to_owned(),
            id: value.id().into(),
            targets: value.targets(),
        }
    }
}

impl AttributePrototypeArgument {
    pub fn id(&self) -> AttributePrototypeArgumentId {
        self.id
    }

    pub fn targets(&self) -> Option<ArgumentTargets> {
        self.targets
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub async fn static_value_by_id(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Option<StaticArgumentValue>> {
        let mut static_value_id: Option<StaticArgumentValueId> = None;
        {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            for node_idx in workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    apa_id,
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                )
                .await?
            {
                match workspace_snapshot.get_node_weight(node_idx).await? {
                    NodeWeight::Content(inner) => {
                        let inner_addr_discrim: ContentAddressDiscriminants =
                            inner.content_address().into();

                        if inner_addr_discrim == ContentAddressDiscriminants::StaticArgumentValue {
                            static_value_id = Some(inner.id().into());
                            break;
                        }
                    }
                    _ => continue,
                }
            }
        }

        Ok(match static_value_id {
            Some(static_value_id) => {
                Some(StaticArgumentValue::get_by_id(ctx, static_value_id).await?)
            }
            None => None,
        })
    }

    implement_add_edge_to!(
        source_id: AttributePrototypeArgumentId,
        destination_id: FuncArgumentId,
        add_fn: add_edge_to_func_argument,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: AttributePrototypeArgumentResult,
    );

    // This can be used for InputSocketId, OutputSocketId, PropId, SecretId, or StaticArgumentValueId
    implement_add_edge_to!(
        source_id: AttributePrototypeArgumentId,
        destination_id: Ulid,
        add_fn: add_edge_to_value,
        discriminant: EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        result: AttributePrototypeArgumentResult,
    );

    implement_add_edge_to!(
        source_id: AttributePrototypeArgumentId,
        destination_id: AttributeValueId,
        add_fn: add_value_subscription_edge,
        discriminant: EdgeWeightKindDiscriminants::ValueSubscription,
        result: AttributePrototypeArgumentResult,
    );

    implement_add_edge_to!(
        source_id: AttributePrototypeArgumentId,
        destination_id: Ulid,
        add_fn: add_reason_edge,
        discriminant: EdgeWeightKindDiscriminants::Reason,
        result: AttributePrototypeArgumentResult,
    );

    pub async fn get_by_id(
        ctx: &DalContext,
        id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_weight = workspace_snapshot.get_node_weight(id).await?;

        Ok(node_weight
            .get_attribute_prototype_argument_node_weight()?
            .into())
    }

    pub async fn add_reason(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
        reason: Reason,
    ) -> AttributePrototypeArgumentResult<()> {
        let workspace = ctx.workspace_snapshot()?;

        let reason_node = Reason::new_reason_node(reason);
        let reason_id = reason_node.id();
        workspace.add_or_replace_node(reason_node).await?;
        Self::add_reason_edge(ctx, apa_id, reason_id, EdgeWeightKind::Reason).await?;

        Ok(())
    }

    pub async fn get_reasons(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Vec<Reason>> {
        let workspace = ctx.workspace_snapshot()?;

        Ok(workspace
            .all_outgoing_targets(apa_id)
            .await?
            .into_iter()
            .filter_map(|node| match node {
                NodeWeight::Reason(reason_node_weight) => Some(reason_node_weight.reason()),
                _ => None,
            })
            .collect())
    }

    pub async fn new(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        arg_id: FuncArgumentId,
        value_source: impl Into<ValueSource>,
    ) -> AttributePrototypeArgumentResult<Self> {
        let argument = Self::new_without_source(ctx, prototype_id, arg_id).await?;
        Self::attach_value_source(ctx, argument.id, value_source).await?;
        Ok(argument)
    }

    pub async fn new_without_source(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        arg_id: FuncArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;

        let node_weight = NodeWeight::new_attribute_prototype_argument(id, lineage_id, None);

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot
            .add_or_replace_node(node_weight.clone())
            .await?;

        AttributePrototype::add_edge_to_argument(
            ctx,
            prototype_id,
            id.into(),
            EdgeWeightKind::PrototypeArgument,
        )
        .await?;

        let argument: Self = node_weight
            .get_attribute_prototype_argument_node_weight()?
            .into();
        Self::add_edge_to_func_argument(ctx, argument.id, arg_id, EdgeWeightKind::new_use())
            .await?;

        Ok(argument)
    }

    pub async fn new_static_value(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        arg_id: FuncArgumentId,
        value: serde_json::Value,
    ) -> AttributePrototypeArgumentResult<Self> {
        let static_value = StaticArgumentValue::new(ctx, value).await?;
        Self::new(ctx, prototype_id, arg_id, static_value.id()).await
    }

    #[instrument(level = "info", skip(ctx))]
    pub async fn new_inter_component(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let node_weight = NodeWeight::new_attribute_prototype_argument(
            id,
            lineage_id,
            Some(ArgumentTargets {
                source_component_id,
                destination_component_id,
            }),
        );

        let prototype_func_id =
            AttributePrototype::func_id(ctx, destination_attribute_prototype_id).await?;
        let func_arg_ids = FuncArgument::list_ids_for_func(ctx, prototype_func_id).await?;

        if func_arg_ids.len() > 1 {
            return Err(AttributePrototypeArgumentError::InterComponentDestinationPrototypeHasTooManyFuncArgs(destination_attribute_prototype_id));
        }

        let func_arg_id = func_arg_ids.first().ok_or(
            AttributePrototypeArgumentError::InterComponentDestinationPrototypeHasNoFuncArgs(
                destination_attribute_prototype_id,
            ),
        )?;

        let prototype_arg: Self = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            workspace_snapshot
                .add_or_replace_node(node_weight.clone())
                .await?;

            AttributePrototype::add_edge_to_argument(
                ctx,
                destination_attribute_prototype_id,
                id.into(),
                EdgeWeightKind::PrototypeArgument,
            )
            .await?;

            let prototype_arg: Self = node_weight
                .get_attribute_prototype_argument_node_weight()?
                .into();

            Self::add_edge_to_func_argument(
                ctx,
                prototype_arg.id,
                *func_arg_id,
                EdgeWeightKind::new_use(),
            )
            .await?;

            prototype_arg
        };

        Self::attach_value_source(ctx, prototype_arg.id, source_output_socket_id).await?;

        Ok(prototype_arg)
    }

    pub async fn func_argument(
        &self,
        ctx: &DalContext,
    ) -> AttributePrototypeArgumentResult<FuncArgument> {
        let func_arg_id = Self::func_argument_id(ctx, self.id).await?;
        Ok(FuncArgument::get_by_id(ctx, func_arg_id).await?)
    }

    pub async fn func_argument_id(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<FuncArgumentId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(apa_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            match workspace_snapshot
                .get_node_weight(target)
                .await?
                .get_func_argument_node_weight()
            {
                Ok(func_argument_node_weight) => {
                    return Ok(func_argument_node_weight.id().into());
                }
                Err(NodeWeightError::UnexpectedNodeWeightVariant(_, _)) => continue,
                Err(e) => Err(e)?,
            }
        }

        Err(AttributePrototypeArgumentError::MissingFuncArgument(apa_id))
    }

    pub async fn find_by_func_argument_id_and_attribute_prototype_id(
        ctx: &DalContext,
        func_argument_id: FuncArgumentId,
        ap_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Option<AttributePrototypeArgumentId>> {
        // AP --> APA --> Func Arg

        for apa_id in AttributePrototype::list_arguments(ctx, ap_id).await? {
            let this_func_arg_id = Self::func_argument_id(ctx, apa_id).await?;

            if this_func_arg_id == func_argument_id {
                return Ok(Some(apa_id));
            }
        }

        Ok(None)
    }

    pub async fn value_source(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<ValueSource> {
        Self::value_source_opt(ctx, apa_id)
            .await?
            .ok_or(AttributePrototypeArgumentError::MissingValueSource(apa_id))
    }

    pub async fn value_source_opt(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Option<ValueSource>> {
        let snap = ctx.workspace_snapshot()?;
        for (edge, _, target) in snap.edges_directed(apa_id, Direction::Outgoing).await? {
            match edge.kind() {
                // Handle APA -- PrototypeArgumentValue -> Prop/Secret/InputSocket/OutputSocket/StaticArgumentValue
                EdgeWeightKind::PrototypeArgumentValue => {
                    return Ok(Some(match snap.get_node_weight(target).await? {
                        NodeWeight::Prop(node) => ValueSource::Prop(node.id().into()),
                        NodeWeight::Secret(node) => ValueSource::Secret(node.id().into()),
                        NodeWeight::Content(node) => match node.content_address() {
                            ContentAddress::InputSocket(..) => {
                                ValueSource::InputSocket(node.id().into())
                            }
                            ContentAddress::OutputSocket(..) => {
                                ValueSource::OutputSocket(node.id().into())
                            }
                            ContentAddress::StaticArgumentValue(..) => {
                                ValueSource::StaticArgumentValue(node.id().into())
                            }
                            other => {
                                return Err(
                                    AttributePrototypeArgumentError::UnexpectedValueSourceContent(
                                        apa_id,
                                        other.into(),
                                    ),
                                );
                            }
                        },
                        NodeWeight::InputSocket(node) => ValueSource::InputSocket(node.id().into()),
                        _ => {
                            return Err(
                                AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                                    apa_id,
                                    edge.kind().into(),
                                ),
                            );
                        }
                    }));
                }

                // Handle APA -- ValueSubscription(path) -> AttributeValue
                EdgeWeightKind::ValueSubscription(path) => {
                    return Ok(Some(match snap.get_node_weight(target).await? {
                        NodeWeight::AttributeValue(node) => {
                            ValueSource::ValueSubscription(ValueSubscription {
                                attribute_value_id: node.id().into(),
                                path: path.clone(),
                            })
                        }
                        _ => {
                            return Err(
                                AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                                    apa_id,
                                    edge.kind().into(),
                                ),
                            );
                        }
                    }));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// This sets the value source on an APA.
    /// If the APA already has a value source, it will be replaced.
    pub async fn set_value_source(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
        value_source: impl Into<ValueSource>,
    ) -> AttributePrototypeArgumentResult<()> {
        // First remove any existing sources.
        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot
            .remove_outgoing_edges_of_kind(
                apa_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?;
        workspace_snapshot
            .remove_outgoing_edges_of_kind(apa_id, EdgeWeightKindDiscriminants::ValueSubscription)
            .await?;

        // Then attach the new one.
        Self::attach_value_source(ctx, apa_id, value_source).await
    }

    /// This will attach a value source to an APA.
    ///
    /// The APA *must not* already have a value source.
    ///
    /// This should only be used in conjunction with new_without_source().
    async fn attach_value_source(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
        value_source: impl Into<ValueSource>,
    ) -> AttributePrototypeArgumentResult<()> {
        let value_source = value_source.into();
        match value_source {
            ValueSource::InputSocket(_)
            | ValueSource::OutputSocket(_)
            | ValueSource::Prop(_)
            | ValueSource::Secret(_)
            | ValueSource::StaticArgumentValue(_) => {
                Self::add_edge_to_value(
                    ctx,
                    apa_id,
                    value_source.into_inner_id(),
                    EdgeWeightKind::PrototypeArgumentValue,
                )
                .await
            }
            ValueSource::ValueSubscription(ValueSubscription {
                attribute_value_id,
                path,
            }) => {
                Self::add_value_subscription_edge(
                    ctx,
                    apa_id,
                    attribute_value_id,
                    EdgeWeightKind::ValueSubscription(path),
                )
                .await
            }
        }
    }

    pub async fn prototype_id(
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<AttributePrototypeId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let prototype_idxs = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                attribute_prototype_argument_id,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?;

        if prototype_idxs.len() != 1 {
            return Err(WorkspaceSnapshotError::UnexpectedNumberOfIncomingEdges(
                EdgeWeightKindDiscriminants::PrototypeArgument,
                NodeWeightDiscriminants::AttributePrototypeArgument,
                attribute_prototype_argument_id.into(),
            )
            .into());
        }

        let prototype_idx = prototype_idxs
            .first()
            .copied()
            .expect("checked length above");

        let prototype_node_weight = workspace_snapshot.get_node_weight(prototype_idx).await?;

        Ok(prototype_node_weight.id().into())
    }

    pub async fn set_static_value_source(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
        value: serde_json::Value,
    ) -> AttributePrototypeArgumentResult<StaticArgumentValue> {
        let static_value = StaticArgumentValue::new(ctx, value).await?;
        Self::attach_value_source(ctx, apa_id, static_value.id()).await?;
        Ok(static_value)
    }

    pub async fn list_ids_for_prototype(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Vec<AttributePrototypeArgumentId>> {
        let mut apas = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let apa_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                prototype_id,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?;

        for idx in apa_node_idxs {
            let node_weight = workspace_snapshot.get_node_weight(idx).await?;
            apas.push(node_weight.id().into())
        }

        Ok(apas)
    }

    pub async fn list_ids_for_prototype_and_destination(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        destination_id: ComponentId,
    ) -> AttributePrototypeArgumentResult<Vec<AttributePrototypeArgumentId>> {
        let mut apas = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let apa_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                prototype_id,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?;

        for idx in apa_node_idxs {
            let node_weight = workspace_snapshot.get_node_weight(idx).await?;
            if let NodeWeight::AttributePrototypeArgument(apa_weight) = &node_weight {
                if let Some(ArgumentTargets {
                    destination_component_id,
                    ..
                }) = apa_weight.targets()
                {
                    if destination_component_id == destination_id {
                        apas.push(node_weight.id().into())
                    }
                }
            }
        }

        Ok(apas)
    }

    pub async fn list_ids_for_output_socket_and_source(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
        source_id: ComponentId,
    ) -> AttributePrototypeArgumentResult<Vec<AttributePrototypeArgumentId>> {
        let mut apas = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let apa_node_idxs = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                output_socket_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?;

        for idx in apa_node_idxs {
            let node_weight = workspace_snapshot.get_node_weight(idx).await?;
            if let NodeWeight::AttributePrototypeArgument(apa_weight) = &node_weight {
                if let Some(ArgumentTargets {
                    source_component_id,
                    ..
                }) = apa_weight.targets()
                {
                    if source_component_id == source_id {
                        apas.push(node_weight.id().into())
                    }
                }
            }
        }

        Ok(apas)
    }

    /// Removes the [`AttributePrototypeArgument`] corresponding to the provided ID.
    pub async fn remove(
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<()> {
        let attribute_prototype_argument =
            Self::get_by_id(ctx, attribute_prototype_argument_id).await?;
        attribute_prototype_argument.remove_inner(ctx).await?;
        Ok(())
    }

    /// Removes the [`AttributePrototypeArgument`] corresponding to the provided ID, but is a
    /// "no-op" if it cannot be found before removal.
    pub async fn remove_or_no_op(
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<()> {
        let attribute_prototype_argument =
            match Self::get_by_id(ctx, attribute_prototype_argument_id).await {
                Ok(found_attribute_prototype_argument) => found_attribute_prototype_argument,
                Err(AttributePrototypeArgumentError::WorkspaceSnapshot(
                    WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                        WorkspaceSnapshotGraphError::NodeWithIdNotFound(raw_id),
                    ),
                )) if raw_id == attribute_prototype_argument_id.into() => return Ok(()),
                Err(err) => return Err(err),
            };

        attribute_prototype_argument.remove_inner(ctx).await?;
        Ok(())
    }

    /// A _private_ method that consumes self and removes the corresponding
    /// [`AttributePrototypeArgument`].
    async fn remove_inner(self, ctx: &DalContext) -> AttributePrototypeArgumentResult<()> {
        let prototype_id = Self::prototype_id(ctx, self.id).await?;
        // Find all of the "destination" attribute values.
        let mut avs_to_update = AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;
        // If the argument has targets, then we only care about AVs that are for the same
        // destination component.
        if let Some(targets) = self.targets() {
            let mut av_ids_to_keep = HashSet::new();
            for av_id in &avs_to_update {
                let component_id = AttributeValue::component_id(ctx, *av_id).await?;
                if component_id == targets.destination_component_id {
                    av_ids_to_keep.insert(*av_id);
                }
            }
            avs_to_update.retain(|av_id| av_ids_to_keep.contains(av_id));
        }

        // Remove the argument
        ctx.workspace_snapshot()?.remove_node_by_id(self.id).await?;

        // Enqueue a dependent values update with the destination attribute values
        ctx.add_dependent_values_and_enqueue(avs_to_update).await?;

        Ok(())
    }

    /// Get the value, formatted for debugging/display.
    /// Pass component_id to get a more concise title if this APA is for a socket connection
    /// (i.e. is on the prototype in the schema, but is for a specific component).
    pub async fn fmt_title(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
        component_id: Option<ComponentId>,
    ) -> String {
        Self::fmt_title_fallible(ctx, apa_id, component_id)
            .await
            .unwrap_or_else(|e| e.to_string())
    }

    async fn fmt_title_fallible(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
        component_id: Option<ComponentId>,
    ) -> AttributePrototypeArgumentResult<String> {
        let mut title = match Self::value_source_opt(ctx, apa_id).await? {
            Some(value_source) => value_source.fmt_title(ctx).await,
            None => "<no value source>".to_string(),
        };
        let apa = Self::get_by_id(ctx, apa_id).await?;
        if let Some(ArgumentTargets {
            source_component_id,
            destination_component_id,
        }) = apa.targets()
        {
            if Some(source_component_id) != component_id {
                title.push_str(" on ");
                title.push_str(&Component::fmt_title(ctx, source_component_id).await);
            }
            if Some(destination_component_id) != component_id {
                title.push_str(" (only for ");
                title.push_str(&Component::fmt_title(ctx, destination_component_id).await);
            }
        }
        Ok(title)
    }
}
