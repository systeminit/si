//! An [`AttributePrototypeArgument`] joins a prototype to a function argument
//! and to either the input socket that supplies its value or to a constant
//! value. It defines source of the value for the function argument in the
//! context of the prototype.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::{
    change_set::ChangeSetError,
    func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId},
    implement_add_edge_to, pk,
    socket::input::InputSocketId,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::{EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
        node_weight::{
            AttributePrototypeArgumentNodeWeight, NodeWeight, NodeWeightDiscriminants,
            NodeWeightError,
        },
        WorkspaceSnapshotError,
    },
    AttributePrototype, AttributePrototypeId, AttributeValue, ComponentId, DalContext, HelperError,
    OutputSocketId, PropId, SecretId, Timestamp, TransactionsError,
};

use self::{
    static_value::{StaticArgumentValue, StaticArgumentValueId},
    value_source::ValueSource,
};

pub use crate::workspace_snapshot::node_weight::attribute_prototype_argument_node_weight::ArgumentTargets;

use super::AttributePrototypeError;

pub mod static_value;
pub mod value_source;

// TODO(nick): switch to the "id!" macro once the frontend doesn't use the old nil id to indicate
// that the argument is a new one.
pk!(AttributePrototypeArgumentId);

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
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
    MissingSource(AttributePrototypeArgumentId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("no targets for prototype argument: {0}")]
    NoTargets(AttributePrototypeArgumentId),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error(
    "PrototypeArgument {0} ArgumentValue edge pointing to unexpected content node weight kind: {1:?}"
    )]
    UnexpectedValueSourceContent(AttributePrototypeArgumentId, ContentAddressDiscriminants),
    #[error(
        "PrototypeArgument {0} ArgumentValue edge pointing to unexpected node weight kind: {1:?}"
    )]
    UnexpectedValueSourceNode(AttributePrototypeArgumentId, NodeWeightDiscriminants),
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

    implement_add_edge_to!(
        source_id: AttributePrototypeArgumentId,
        destination_id: Ulid,
        add_fn: add_edge_to_value,
        discriminant: EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        result: AttributePrototypeArgumentResult,
    );

    pub async fn get_by_id(
        ctx: &DalContext,
        id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;

        Ok(node_weight
            .get_attribute_prototype_argument_node_weight()?
            .into())
    }

    pub async fn new(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        arg_id: FuncArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_attribute_prototype_argument(change_set, id, None)?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot.add_node(node_weight.clone()).await?;

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
    #[instrument(level = "info", skip(ctx))]
    pub async fn new_inter_component(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_attribute_prototype_argument(
            change_set,
            id,
            Some(ArgumentTargets {
                source_component_id,
                destination_component_id,
            }),
        )?;

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

            workspace_snapshot.add_node(node_weight.clone()).await?;

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

        prototype_arg
            .set_value_from_output_socket_id(ctx, source_output_socket_id)
            .await
    }

    pub async fn func_argument_id_by_id(
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

    pub async fn value_source(
        &self,
        ctx: &DalContext,
    ) -> AttributePrototypeArgumentResult<Option<ValueSource>> {
        Self::value_source_by_id(ctx, self.id).await
    }

    pub async fn value_source_by_id(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Option<ValueSource>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(target) = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                apa_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?
            .into_iter()
            .next()
        {
            match workspace_snapshot.get_node_weight(target).await? {
                NodeWeight::Prop(inner) => {
                    return Ok(Some(ValueSource::Prop(inner.id().into())));
                }
                NodeWeight::Secret(inner) => {
                    return Ok(Some(ValueSource::Secret(inner.id().into())));
                }
                NodeWeight::Content(inner) => {
                    let discrim: ContentAddressDiscriminants = inner.content_address().into();
                    return Ok(Some(match discrim {
                        ContentAddressDiscriminants::InputSocket => {
                            ValueSource::InputSocket(inner.id().into())
                        }
                        ContentAddressDiscriminants::OutputSocket => {
                            ValueSource::OutputSocket(inner.id().into())
                        }
                        ContentAddressDiscriminants::StaticArgumentValue => {
                            ValueSource::StaticArgumentValue(inner.id().into())
                        }
                        other => {
                            return Err(
                                AttributePrototypeArgumentError::UnexpectedValueSourceContent(
                                    apa_id, other,
                                ),
                            );
                        }
                    }));
                }
                other => {
                    return Err(AttributePrototypeArgumentError::UnexpectedValueSourceNode(
                        apa_id,
                        other.into(),
                    ));
                }
            }
        }

        Ok(None)
    }

    async fn set_value_source(
        self,
        ctx: &DalContext,
        value_id: Ulid,
    ) -> AttributePrototypeArgumentResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let change_set = ctx.change_set()?;

        for existing_value_source in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                self.id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?
        {
            let self_node_index = workspace_snapshot.get_node_index_by_id(self.id).await?;
            workspace_snapshot
                .remove_edge(
                    change_set,
                    self_node_index,
                    existing_value_source,
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                )
                .await?;
        }

        Self::add_edge_to_value(
            ctx,
            self.id,
            value_id,
            EdgeWeightKind::PrototypeArgumentValue,
        )
        .await?;

        Ok(self)
    }

    pub async fn prototype_id_for_argument_id(
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

    pub async fn prototype_id(
        &self,
        ctx: &DalContext,
    ) -> AttributePrototypeArgumentResult<AttributePrototypeId> {
        Self::prototype_id_for_argument_id(ctx, self.id).await
    }

    pub async fn set_value_from_input_socket_id(
        self,
        ctx: &DalContext,
        input_socket_id: InputSocketId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, input_socket_id.into()).await
    }

    pub async fn set_value_from_output_socket_id(
        self,
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, output_socket_id.into()).await
    }

    pub async fn set_value_from_prop_id(
        self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, prop_id.into()).await
    }

    pub async fn set_value_from_secret_id(
        self,
        ctx: &DalContext,
        secret_id: SecretId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, secret_id.into()).await
    }

    pub async fn set_value_from_static_value_id(
        self,
        ctx: &DalContext,
        value_id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, value_id.into()).await
    }

    pub async fn set_value_from_static_value(
        self,
        ctx: &DalContext,
        value: serde_json::Value,
    ) -> AttributePrototypeArgumentResult<Self> {
        let static_value = StaticArgumentValue::new(ctx, value).await?;

        self.set_value_from_static_value_id(ctx, static_value.id())
            .await
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
        let prototype_id = self.prototype_id(ctx).await?;
        // Find all of the "destination" attribute values.
        let mut avs_to_update = AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;
        // If the argument has targets, then we only care about AVs that are for the same
        // destination component.
        if let Some(targets) = self.targets() {
            let mut av_ids_to_keep = HashSet::new();
            for av_id in &avs_to_update {
                let component_id = AttributeValue::component_id(ctx, *av_id)
                    .await
                    .map_err(|e| AttributePrototypeArgumentError::AttributeValue(e.to_string()))?;
                if component_id == targets.destination_component_id {
                    av_ids_to_keep.insert(*av_id);
                }
            }
            avs_to_update.retain(|av_id| av_ids_to_keep.contains(av_id));
        }

        // Remove the argument
        ctx.workspace_snapshot()?
            .remove_node_by_id(ctx.change_set()?, self.id)
            .await?;

        // Enqueue a dependent values update with the destination attribute values
        ctx.add_dependent_values_and_enqueue(avs_to_update).await?;

        Ok(())
    }
}
