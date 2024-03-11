//! An [`AttributePrototypeArgument`] joins a prototype to a function argument
//! and to either the input socket that supplies its value or to a constant
//! value. It defines source of the value for the function argument in the
//! context of the prototype.

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    change_set_pointer::ChangeSetPointerError,
    func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId},
    pk,
    socket::input::InputSocketId,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
        node_weight::{
            AttributePrototypeArgumentNodeWeight, NodeWeight, NodeWeightDiscriminants,
            NodeWeightError,
        },
        WorkspaceSnapshotError,
    },
    AttributePrototype, AttributePrototypeId, ComponentId, DalContext, OutputSocketId, PropId,
    Timestamp, TransactionsError,
};

use self::{
    static_value::{StaticArgumentValue, StaticArgumentValueId},
    value_source::ValueSource,
};

pub use crate::workspace_snapshot::node_weight::attribute_prototype_argument_node_weight::ArgumentTargets;

use super::AttributePrototypeError;

pub mod static_value;
pub mod value_source;

pk!(AttributePrototypeArgumentId);

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("Destination prototype {0} has no function arguments")]
    InterComponentDestinationPrototypeHasNoFuncArgs(AttributePrototypeId),
    #[error("Destination prototype {0} has more than one function argument")]
    InterComponentDestinationPrototypeHasTooManyFuncArgs(AttributePrototypeId),
    #[error("attribute prototype argument {0} has no func argument")]
    MissingFuncArgument(AttributePrototypeArgumentId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
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
            let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

            for node_idx in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                apa_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )? {
                match workspace_snapshot.get_node_weight(node_idx)? {
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

    pub async fn get_by_id(
        ctx: &DalContext,
        id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;

        Ok(node_weight
            .get_attribute_prototype_argument_node_weight()?
            .into())
    }

    pub async fn new(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        arg_id: FuncArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_attribute_prototype_argument(change_set, id, None)?;

        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;

        workspace_snapshot.add_node(node_weight.clone())?;

        workspace_snapshot.add_edge(
            prototype_id,
            EdgeWeight::new(change_set, EdgeWeightKind::PrototypeArgument)?,
            id,
        )?;

        workspace_snapshot.add_edge(
            id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            arg_id,
        )?;

        Ok(node_weight
            .get_attribute_prototype_argument_node_weight()?
            .into())
    }

    pub async fn new_inter_component(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let change_set = ctx.change_set_pointer()?;
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
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;

            workspace_snapshot.add_node(node_weight.clone())?;

            workspace_snapshot.add_edge(
                destination_attribute_prototype_id,
                EdgeWeight::new(change_set, EdgeWeightKind::PrototypeArgument)?,
                id,
            )?;

            let prototype_arg: Self = node_weight
                .get_attribute_prototype_argument_node_weight()?
                .into();

            workspace_snapshot.add_edge(
                prototype_arg.id(),
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                func_arg_id,
            )?;

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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        for target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(apa_id, EdgeWeightKindDiscriminants::Use)?
        {
            match workspace_snapshot
                .get_node_weight(target)?
                .get_func_argument_node_weight()
            {
                Ok(content_node_weight) => {
                    return Ok(content_node_weight.id().into());
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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        if let Some(target) = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                apa_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )?
            .into_iter()
            .next()
        {
            match workspace_snapshot.get_node_weight(target)? {
                NodeWeight::Prop(inner) => {
                    return Ok(Some(ValueSource::Prop(inner.id().into())));
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
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        let change_set = ctx.change_set_pointer()?;

        for existing_value_source in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            self.id,
            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        )? {
            let self_node_index = workspace_snapshot.get_node_index_by_id(self.id)?;
            workspace_snapshot.remove_edge(
                change_set,
                self_node_index,
                existing_value_source,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )?;
        }

        workspace_snapshot.add_edge(
            self.id,
            EdgeWeight::new(change_set, EdgeWeightKind::PrototypeArgumentValue)?,
            value_id,
        )?;

        Ok(self)
    }

    pub async fn prototype_id_for_argument_id(
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<AttributePrototypeId> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let prototype_idxs = workspace_snapshot.incoming_sources_for_edge_weight_kind(
            attribute_prototype_argument_id,
            EdgeWeightKindDiscriminants::PrototypeArgument,
        )?;

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

        let prototype_node_weight = workspace_snapshot.get_node_weight(prototype_idx)?;

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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let apa_node_idxs = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            prototype_id,
            EdgeWeightKindDiscriminants::PrototypeArgument,
        )?;

        for idx in apa_node_idxs {
            let node_weight = workspace_snapshot.get_node_weight(idx)?;
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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let apa_node_idxs = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            prototype_id,
            EdgeWeightKindDiscriminants::PrototypeArgument,
        )?;

        for idx in apa_node_idxs {
            let node_weight = workspace_snapshot.get_node_weight(idx)?;
            if let NodeWeight::AttributePrototypeArgument(apa_weight) = node_weight {
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

    pub async fn remove(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
        let change_set = ctx.change_set_pointer()?;
        workspace_snapshot.remove_node_by_id(change_set, apa_id)?;

        Ok(())
    }
}
