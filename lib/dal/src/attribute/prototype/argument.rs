//! An [`AttributePrototypeArgument`] joins a prototype to a function argument
//! and to either the internal provider that supplies its value or to a constant
//! value. It defines source of the value for the function argument in the
//! context of the prototype.

use content_store::Store;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    change_set_pointer::ChangeSetPointerError,
    func::argument::FuncArgumentId,
    pk,
    provider::internal::InternalProviderId,
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
        node_weight::{NodeWeight, NodeWeightError},
        WorkspaceSnapshotError,
    },
    AttributePrototypeId, DalContext, Timestamp, TransactionsError,
};

use self::static_value::{StaticArgumentValue, StaticArgumentValueId};

pub mod static_value;

pk!(AttributePrototypeArgumentId);

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
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
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributePrototypeArgumentResult<T> = Result<T, AttributePrototypeArgumentError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototypeArgument {
    id: AttributePrototypeArgumentId,
    timestamp: Timestamp,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum AttributePrototypeArgumentContent {
    V1(AttributePrototypeArgumentContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributePrototypeArgumentContentV1 {
    pub timestamp: Timestamp,
}

impl AttributePrototypeArgument {
    pub fn assemble(
        id: AttributePrototypeArgumentId,
        inner: &AttributePrototypeArgumentContentV1,
    ) -> Self {
        Self {
            id,
            timestamp: inner.timestamp.to_owned(),
        }
    }

    pub fn id(&self) -> AttributePrototypeArgumentId {
        self.id
    }

    pub async fn static_value_by_id(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<Option<StaticArgumentValue>> {
        let mut static_value_id: Option<StaticArgumentValueId> = None;
        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

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

    pub fn new(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
        arg_id: FuncArgumentId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let timestamp = Timestamp::now();
        let content = AttributePrototypeArgumentContentV1 { timestamp };

        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&AttributePrototypeArgumentContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(
            change_set,
            id,
            ContentAddress::AttributePrototypeArgument(hash),
        )?;

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        workspace_snapshot.add_node(node_weight)?;

        workspace_snapshot.add_edge(
            prototype_id.into(),
            EdgeWeight::new(change_set, EdgeWeightKind::PrototypeArgument)?,
            id,
        )?;
        workspace_snapshot.add_edge(
            id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            arg_id.into(),
        )?;

        Ok(AttributePrototypeArgument::assemble(id.into(), &content))
    }

    pub fn func_argument_id_by_id(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<FuncArgumentId> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        for target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(apa_id, EdgeWeightKindDiscriminants::Use)?
        {
            match workspace_snapshot
                .get_node_weight(target)?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::FuncArg)
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

    fn set_value_source(
        self,
        ctx: &DalContext,
        value_id: Ulid,
    ) -> AttributePrototypeArgumentResult<Self> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
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
            self.id.into(),
            EdgeWeight::new(change_set, EdgeWeightKind::PrototypeArgumentValue)?,
            value_id,
        )?;

        Ok(self)
    }

    pub fn set_value_from_internal_provider_id(
        self,
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, internal_provider_id.into())
    }

    pub fn set_value_from_static_value_id(
        self,
        ctx: &DalContext,
        value_id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<Self> {
        self.set_value_source(ctx, value_id.into())
    }

    pub fn set_value_from_static_value(
        self,
        ctx: &DalContext,
        value: serde_json::Value,
    ) -> AttributePrototypeArgumentResult<Self> {
        let static_value = StaticArgumentValue::new(ctx, value)?;

        self.set_value_from_static_value_id(ctx, static_value.id())
    }

    pub fn list_ids_for_prototype(
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Vec<AttributePrototypeArgumentId>> {
        let mut apas = vec![];
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

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

    pub fn remove(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> AttributePrototypeArgumentResult<()> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        workspace_snapshot.remove_incoming_edges_of_kind(
            ctx.change_set_pointer()?,
            apa_id,
            EdgeWeightKindDiscriminants::PrototypeArgument,
        )?;
        workspace_snapshot.remove_node_by_id(apa_id)?;

        Ok(())
    }
}

// use si_data_pg::PgError;
// use telemetry::prelude::*;
// use thiserror::Error;

// const LIST_FOR_ATTRIBUTE_PROTOTYPE: &str =
//     include_str!("../../queries/attribute_prototype_argument/list_for_attribute_prototype.sql");
// const LIST_FOR_FUNC_ARGUMENT_ID: &str =
//     include_str!("../../queries/attribute_prototype_argument/list_for_func_argument.sql");
// const FIND_FOR_PROVIDERS_AND_COMPONENTS: &str = include_str!(
//     "../../queries/attribute_prototype_argument/find_for_providers_and_components.sql"
// );

// #[remain::sorted]
// #[derive(Error, Debug)]
// pub enum AttributePrototypeArgumentError {
//     #[error("cannot update set field to become unset: {0}")]
//     CannotFlipSetFieldToUnset(&'static str),
//     #[error("cannot update unset field to become set: {0}")]
//     CannotFlipUnsetFieldToSet(&'static str),
//     #[error("history event error: {0}")]
//     HistoryEvent(#[from] HistoryEventError),
//     #[error("pg error: {0}")]
//     Pg(#[from] PgError),
//     #[error("required value fields must be set, found at least one unset required value field")]
//     RequiredValueFieldsUnset,
//     #[error("serde json error: {0}")]
//     SerdeJson(#[from] serde_json::Error),
//     #[error("standard model error: {0}")]
//     StandardModel(#[from] StandardModelError),
//     #[error("transactions error: {0}")]
//     Transactions(#[from] TransactionsError),
// }

// pub type AttributePrototypeArgumentResult<T> = Result<T, AttributePrototypeArgumentError>;

// /// Contains a "key" and fields to derive a "value" that dynamically used as an argument for a
// /// [`AttributePrototypes`](crate::AttributePrototype) function execution.
// #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
// pub struct AttributePrototypeArgument {
//     pk: AttributePrototypeArgumentPk,
//     id: AttributePrototypeArgumentId,
//     #[serde(flatten)]
//     tenancy: Tenancy,
//     #[serde(flatten)]
//     visibility: Visibility,
//     #[serde(flatten)]
//     timestamp: Timestamp,

//     /// Indicates the [`AttributePrototype`](crate::AttributePrototype) that [`Self`] is used as
//     /// an argument for.
//     attribute_prototype_id: AttributePrototypeId,
//     /// Where to find the name and type of the "key" for a given argument.
//     func_argument_id: FuncArgumentId,
//     /// Where to find the value for a given argument for _intra_ [`Component`](crate::Component)
//     /// connections.
//     internal_provider_id: InternalProviderId,
//     /// Where to find the value for a given argument for _inter_ [`Component`](crate::Component)
//     /// connections.
//     external_provider_id: ExternalProviderId,
//     /// For _inter_ [`Component`](crate::Component) connections, this field provides additional
//     /// information to determine the _source_ of the value.
//     tail_component_id: ComponentId,
//     /// For _inter_ [`Component`](crate::Component) connections, this field provides additional
//     /// information to determine the _destination_ of the value.
//     head_component_id: ComponentId,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct AttributePrototypeArgumentGroup {
//     pub name: String,
//     pub arguments: Vec<AttributePrototypeArgument>,
// }

// impl_standard_model! {
//     model: AttributePrototypeArgument,
//     pk: AttributePrototypeArgumentPk,
//     id: AttributePrototypeArgumentId,
//     table_name: "attribute_prototype_arguments",
//     history_event_label_base: "attribute_prototype_argument",
//     history_event_message_name: "Attribute Prototype Argument"
// }

// impl AttributePrototypeArgument {
//     #[instrument(skip_all)]
//     /// Create a new [`AttributePrototypeArgument`] for _intra_ [`Component`](crate::Component) use.
//     pub async fn new_for_intra_component(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         func_argument_id: FuncArgumentId,
//         internal_provider_id: InternalProviderId,
//     ) -> AttributePrototypeArgumentResult<Self> {
//         // Ensure the value fields are what we expect.
//         let external_provider_id = ExternalProviderId::NONE;
//         let tail_component_id = ComponentId::NONE;
//         let head_component_id = ComponentId::NONE;
//         if internal_provider_id == InternalProviderId::NONE {
//             return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
//         }

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &func_argument_id,
//                     &internal_provider_id,
//                     &external_provider_id,
//                     &tail_component_id,
//                     &head_component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::finish_create_from_row(ctx, row).await?)
//     }

//     /// Create a new [`AttributePrototypeArgument`] for _inter_ [`Component`](crate::Component) use.
//     #[instrument(skip_all)]
//     pub async fn new_for_inter_component(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         func_argument_id: FuncArgumentId,
//         head_component_id: ComponentId,
//         tail_component_id: ComponentId,
//         external_provider_id: ExternalProviderId,
//     ) -> AttributePrototypeArgumentResult<Self> {
//         // Ensure the value fields are what we expect.
//         if external_provider_id == ExternalProviderId::NONE
//             || tail_component_id == ComponentId::NONE
//             || head_component_id == ComponentId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
//         }

//         // For inter component connections, the internal provider id field must be unset.
//         let internal_provider_id = InternalProviderId::NONE;

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &func_argument_id,
//                     &internal_provider_id,
//                     &external_provider_id,
//                     &tail_component_id,
//                     &head_component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::finish_create_from_row(ctx, row).await?)
//     }

//     /// Create a new [`AttributePrototypeArgument`] for _inter_ [`Component`](crate::Component) use.
//     #[instrument(skip_all)]
//     pub async fn new_explicit_internal_to_explicit_internal_inter_component(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         func_argument_id: FuncArgumentId,
//         head_component_id: ComponentId,
//         tail_component_id: ComponentId,
//         internal_provider_id: InternalProviderId,
//     ) -> AttributePrototypeArgumentResult<Self> {
//         // Ensure the value fields are what we expect.
//         if internal_provider_id == InternalProviderId::NONE
//             || tail_component_id == ComponentId::NONE
//             || head_component_id == ComponentId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
//         }

//         // For inter component connections, the internal provider id field must be unset.
//         let external_provider_id = ExternalProviderId::NONE;

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &func_argument_id,
//                     &internal_provider_id,
//                     &external_provider_id,
//                     &tail_component_id,
//                     &head_component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::finish_create_from_row(ctx, row).await?)
//     }

//     /// Create a new [`AttributePrototypeArgument`] for _inter_ [`Component`](crate::Component) use.
//     #[instrument(skip_all)]
//     pub async fn new_external_to_external_inter_component(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//         func_argument_id: FuncArgumentId,
//         head_component_id: ComponentId,
//         tail_component_id: ComponentId,
//         external_provider_id: ExternalProviderId,
//     ) -> AttributePrototypeArgumentResult<Self> {
//         // Ensure the value fields are what we expect.
//         if external_provider_id == ExternalProviderId::NONE
//             || tail_component_id == ComponentId::NONE
//             || head_component_id == ComponentId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
//         }

//         // For inter component connections, the internal provider id field must be unset.
//         let internal_provider_id = InternalProviderId::NONE;

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_prototype_id,
//                     &func_argument_id,
//                     &internal_provider_id,
//                     &external_provider_id,
//                     &tail_component_id,
//                     &head_component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::finish_create_from_row(ctx, row).await?)
//     }

//     standard_model_accessor!(
//         attribute_prototype_id,
//         Pk(AttributePrototypeId),
//         AttributePrototypeArgumentResult
//     );
//     standard_model_accessor!(
//         func_argument_id,
//         Pk(FuncArgumentId),
//         AttributePrototypeArgumentResult
//     );
//     standard_model_accessor!(
//         internal_provider_id,
//         Pk(InternalProviderId),
//         AttributePrototypeArgumentResult
//     );
//     standard_model_accessor!(
//         external_provider_id,
//         Pk(ExternalProviderId),
//         AttributePrototypeArgumentResult
//     );
//     standard_model_accessor!(
//         tail_component_id,
//         Pk(ComponentId),
//         AttributePrototypeArgumentResult
//     );
//     standard_model_accessor!(
//         head_component_id,
//         Pk(ComponentId),
//         AttributePrototypeArgumentResult
//     );

//     /// Wraps the standard model accessor for "internal_provider_id" to ensure that a set value
//     /// cannot become unset and vice versa.
//     pub async fn set_internal_provider_id_safe(
//         &mut self,
//         ctx: &DalContext,
//         internal_provider_id: InternalProviderId,
//     ) -> AttributePrototypeArgumentResult<()> {
//         if self.internal_provider_id != InternalProviderId::NONE
//             && internal_provider_id == InternalProviderId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
//                 "InternalProviderId",
//             ));
//         };
//         if self.internal_provider_id == InternalProviderId::NONE
//             && internal_provider_id != InternalProviderId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
//                 "InternalProviderId",
//             ));
//         }
//         self.set_internal_provider_id(ctx, internal_provider_id)
//             .await?;
//         Ok(())
//     }

//     /// Wraps the standard model accessor for "external_provider_id" to ensure that a set value
//     /// cannot become unset and vice versa.
//     pub async fn set_external_provider_id_safe(
//         mut self,
//         ctx: &DalContext,
//         external_provider_id: ExternalProviderId,
//     ) -> AttributePrototypeArgumentResult<()> {
//         if self.external_provider_id != ExternalProviderId::NONE
//             && external_provider_id == ExternalProviderId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
//                 "ExternalProviderId",
//             ));
//         }
//         if self.external_provider_id == ExternalProviderId::NONE
//             && external_provider_id != ExternalProviderId::NONE
//         {
//             return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
//                 "ExternalProviderId",
//             ));
//         }
//         self.set_external_provider_id(ctx, external_provider_id)
//             .await?;
//         Ok(())
//     }

//     /// Wraps the standard model accessor for "tail_component_id" to ensure that a set value
//     /// cannot become unset and vice versa.
//     pub async fn set_tail_component_id_safe(
//         mut self,
//         ctx: &DalContext,
//         tail_component_id: ComponentId,
//     ) -> AttributePrototypeArgumentResult<()> {
//         if self.tail_component_id != ComponentId::NONE && tail_component_id == ComponentId::NONE {
//             return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
//                 "tail ComponentId",
//             ));
//         }
//         if self.tail_component_id == ComponentId::NONE && tail_component_id != ComponentId::NONE {
//             return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
//                 "tail ComponentId",
//             ));
//         }
//         self.set_tail_component_id(ctx, tail_component_id).await?;
//         Ok(())
//     }

//     /// Wraps the standard model accessor for "head_component_id" to ensure that a set value
//     /// cannot become unset and vice versa.
//     pub async fn set_head_component_id_safe(
//         mut self,
//         ctx: &DalContext,
//         head_component_id: ComponentId,
//     ) -> AttributePrototypeArgumentResult<()> {
//         if self.head_component_id != ComponentId::NONE && head_component_id == ComponentId::NONE {
//             return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
//                 "head ComponentId",
//             ));
//         }
//         if self.head_component_id == ComponentId::NONE && head_component_id != ComponentId::NONE {
//             return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
//                 "head ComponentId",
//             ));
//         }
//         self.set_head_component_id(ctx, head_component_id).await?;
//         Ok(())
//     }

//     /// Determines if the [`InternalProviderId`](crate::InternalProvider) is unset. This function
//     /// can be useful for determining how to build [`FuncBinding`](crate::FuncBinding) arguments.
//     pub fn is_internal_provider_unset(&self) -> bool {
//         self.internal_provider_id == InternalProviderId::NONE
//     }

//     /// List all [`AttributePrototypeArguments`](Self) for a given
//     /// [`AttributePrototype`](crate::AttributePrototype).
//     #[tracing::instrument(skip(ctx))]
//     pub async fn list_for_attribute_prototype(
//         ctx: &DalContext,
//         attribute_prototype_id: AttributePrototypeId,
//     ) -> AttributePrototypeArgumentResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_ATTRIBUTE_PROTOTYPE,
//                 &[ctx.tenancy(), ctx.visibility(), &attribute_prototype_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// List all [`AttributePrototypeArguments`](Self) for a given [`FuncArgument`](crate::func::argument::FuncArgument).
//     pub async fn list_by_func_argument_id(
//         ctx: &DalContext,
//         func_argument_id: FuncArgumentId,
//     ) -> AttributePrototypeArgumentResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_FUNC_ARGUMENT_ID,
//                 &[ctx.tenancy(), ctx.visibility(), &func_argument_id],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_for_providers_and_components(
//         ctx: &DalContext,
//         external_provider_id: &ExternalProviderId,
//         internal_provider_id: &InternalProviderId,
//         tail_component: &ComponentId,
//         head_component: &ComponentId,
//     ) -> AttributePrototypeArgumentResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_FOR_PROVIDERS_AND_COMPONENTS,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     external_provider_id,
//                     internal_provider_id,
//                     tail_component,
//                     head_component,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::object_option_from_row_option(row)?)
//     }
// }
