//! This module contains [`Edge`], the mathematical "edge" between two [`Nodes`](crate::Node) in a
//! graph.

use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::node::NodeId;
use crate::{
    impl_standard_model, pk, socket::SocketId, standard_model, standard_model_accessor,
    AttributeReadContext, AttributeValue, ComponentError, ComponentId, DiagramKind,
    ExternalProviderError, Func, HistoryEventError, InternalProviderError, ReadTenancyError,
    SchemaId, StandardModel, StandardModelError, SystemId, Timestamp, Visibility, WriteTenancy,
};
use crate::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, Component, DalContext,
    ExternalProvider, ExternalProviderId, InternalProvider, InternalProviderId, NodeError,
};

const LIST_PARENTS_FOR_COMPONENT: &str =
    include_str!("queries/edge_list_parents_for_component.sql");

#[derive(Error, Debug)]
pub enum EdgeError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),

    #[error("component error: {0}")]
    Component(String),
    #[error("external provider not found for id: {0}")]
    ExternalProviderNotFound(ExternalProviderId),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("implicit internal provider cannot be used for inter component connection: {0}")]
    FoundImplicitInternalProvider(InternalProviderId),
    #[error("internal provider not found for id: {0}")]
    InternalProviderNotFound(InternalProviderId),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("cannot find identity func")]
    IdentityFuncNotFound,
    #[error("cannot find identity func argument")]
    IdentityFuncArgNotFound,
    #[error("error during omega hack: {0}")]
    OmegaHack(String),
}

pub type EdgeResult<T> = Result<T, EdgeError>;

/// Used to dictate what [`EdgeKinds`](EdgeKind) can be for the head and tail of an [`Edges`](Edge).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum VertexObjectKind {
    /// Used for [`Nodes`](crate::Node) of [`NodeKind::Configuration`](crate::NodeKind::Configuration).
    Configuration,
    /// Used for [`Nodes`](crate::Node) of [`NodeKind::System`](crate::NodeKind::System).
    System,
}

/// The kind of an [`Edge`](Edge). This provides the ability to categorize [`Edges`](Edge)
/// and create [`EdgeKind`](Self)-specific graphs.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum EdgeKind {
    /// Used to connect a configuration to another configuration.
    Configuration,
    /// Used to connect a configuration to a system.
    System,
}

pk!(EdgeId);
pk!(EdgePk);

/// A mathematical edge between a head and a tail [`Node`](crate::Node).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pk: EdgePk,
    id: EdgeId,
    kind: EdgeKind,
    // NOTE: Would love to flatten this, but serde doesn't allow flatten and rename.
    head_node_id: NodeId,
    head_object_kind: VertexObjectKind,
    head_object_id: i64,
    head_socket_id: SocketId,
    tail_node_id: NodeId,
    tail_object_kind: VertexObjectKind,
    tail_object_id: i64,
    tail_socket_id: SocketId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Edge,
    pk: EdgePk,
    id: EdgeId,
    table_name: "edges",
    history_event_label_base: "edge",
    history_event_message_name: "Edge"
}

impl Edge {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        kind: EdgeKind,
        head_node_id: NodeId,
        head_object_kind: VertexObjectKind,
        head_object_id: i64,
        head_socket_id: SocketId,
        tail_node_id: NodeId,
        tail_object_kind: VertexObjectKind,
        tail_object_id: i64,
        tail_socket_id: SocketId,
    ) -> EdgeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM edge_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &kind.to_string(),
                    &head_node_id,
                    &head_object_kind.to_string(),
                    &head_object_id,
                    &head_socket_id,
                    &tail_node_id,
                    &tail_object_kind.to_string(),
                    &tail_object_id,
                    &tail_socket_id,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// Creates a new [`Edge`](Self) and "connects" the underlying [`providers`](crate::provider)
    /// for the [`Connection`](crate::Connection).
    ///
    /// Terminology:
    /// - The _from_ side of a [`Connection`](crate::Connection) is the _tail_ of an [`Edge`](Self)
    /// - The _to_ side of a [`Connection`](crate::Connection) is the _head_ of an [`Edge`](Self)
    ///
    /// Please note that the _head_ information comes before the _tail_ information in the
    /// function parameters.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_for_connection(
        ctx: &DalContext,
        head_node_id: NodeId,
        head_socket_id: SocketId,
        tail_node_id: NodeId,
        tail_socket_id: SocketId,
    ) -> EdgeResult<Self> {
        let head_component = Component::find_for_node(ctx, head_node_id)
            .await
            .map_err(|err| EdgeError::Component(err.to_string()))?
            .ok_or(NodeError::ComponentIsNone)?;
        let tail_component = Component::find_for_node(ctx, tail_node_id)
            .await
            .map_err(|err| EdgeError::Component(err.to_string()))?
            .ok_or(NodeError::ComponentIsNone)?;

        let head_explicit_internal_provider =
            InternalProvider::find_explicit_for_socket(ctx, head_socket_id)
                .await?
                .ok_or(EdgeError::InternalProviderNotFoundForSocket(head_socket_id))?;
        let tail_external_provider = ExternalProvider::find_for_socket(ctx, tail_socket_id)
            .await?
            .ok_or(EdgeError::ExternalProviderNotFoundForSocket(tail_socket_id))?;

        // TODO(nick): allow for more transformation functions.
        Self::connect_providers_for_components(
            ctx,
            *head_explicit_internal_provider.id(),
            *head_component.id(),
            *tail_external_provider.id(),
            *tail_component.id(),
        )
        .await?;

        // FIXME(nick): this is an omega hack. If a Butane component ("from"/"tail") is connected to
        // an EC2 component ("to"/"head"), we need to fake the "User Data" field's value.
        let head_schema = head_component
            .schema(ctx)
            .await
            .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?
            .ok_or(ComponentError::SchemaNotFound)
            .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?;
        let tail_schema = tail_component
            .schema(ctx)
            .await
            .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?
            .ok_or(ComponentError::SchemaNotFound)
            .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?;
        if head_schema.name() == "EC2 Instance" && tail_schema.name() == "Butane" {
            butane_to_ec2_user_data_omega_hack(ctx, *head_schema.id(), *head_component.id())
                .await?;
        }

        // NOTE(nick): a lot of hardcoded values here that'll likely need to be adjusted.
        let edge = Edge::new(
            ctx,
            EdgeKind::Configuration,
            head_node_id,
            VertexObjectKind::Configuration,
            (*head_component.id()).into(),
            head_socket_id,
            tail_node_id,
            VertexObjectKind::Configuration,
            (*tail_component.id()).into(),
            tail_socket_id,
        )
        .await?;
        Ok(edge)
    }

    standard_model_accessor!(kind, Enum(EdgeKind), EdgeResult);

    // Sockets
    standard_model_accessor!(head_node_id, Pk(NodeId), EdgeResult);
    standard_model_accessor!(head_object_kind, Enum(VertexObjectKind), EdgeResult);
    standard_model_accessor!(head_object_id, i64, EdgeResult);
    standard_model_accessor!(head_socket_id, Pk(SocketId), EdgeResult);
    standard_model_accessor!(tail_node_id, Pk(NodeId), EdgeResult);
    standard_model_accessor!(tail_object_kind, Enum(VertexObjectKind), EdgeResult);
    standard_model_accessor!(tail_object_id, i64, EdgeResult);
    standard_model_accessor!(tail_socket_id, Pk(SocketId), EdgeResult);

    pub async fn list_parents_for_component(
        ctx: &DalContext,
        head_component_id: ComponentId,
    ) -> EdgeResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_PARENTS_FOR_COMPONENT,
                &[ctx.read_tenancy(), ctx.visibility(), &head_component_id],
            )
            .await?;
        let objects = rows
            .into_iter()
            .map(|row| row.get("tail_object_id"))
            .collect();
        Ok(objects)
    }

    pub async fn include_component_in_system(
        ctx: &DalContext,
        component_id: ComponentId,
        diagram_kind: DiagramKind,
        system_id: SystemId,
    ) -> EdgeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM edge_include_component_in_system_v1($1, $2, $3, $4, $5)",
                &[
                    &ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                    &(diagram_kind.to_string()),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// This function should be only called by [`Self::new_for_connection()`] and integration tests.
    /// The latter is why this function is public.
    ///
    /// When in the context of [`Connections`](crate::Connection), the following terminology is
    /// relevant:
    /// - _"head":_ where the connection is going to
    /// - _"tail":_ where the connection is coming from
    ///
    /// Currently this func only supports connecting via the "si:identity" func, refactoring
    /// is necessary to support other transformation functions for edge connections.
    pub async fn connect_providers_for_components(
        ctx: &DalContext,
        head_explicit_internal_provider_id: InternalProviderId,
        head_component_id: ComponentId,
        tail_external_provider_id: ExternalProviderId,
        tail_component_id: ComponentId,
    ) -> EdgeResult<()> {
        let head_explicit_internal_provider: InternalProvider =
            InternalProvider::get_by_id(ctx, &head_explicit_internal_provider_id)
                .await?
                .ok_or(EdgeError::InternalProviderNotFound(
                    head_explicit_internal_provider_id,
                ))?;
        let tail_external_provider: ExternalProvider =
            ExternalProvider::get_by_id(ctx, &tail_external_provider_id)
                .await?
                .ok_or(EdgeError::ExternalProviderNotFound(
                    tail_external_provider_id,
                ))?;

        // Check that the explicit internal provider is actually explicit and find its attribute
        // prototype id.
        if head_explicit_internal_provider.is_internal_consumer() {
            return Err(EdgeError::FoundImplicitInternalProvider(
                *head_explicit_internal_provider.id(),
            ));
        }
        let head_explicit_internal_provider_attribute_prototype = head_explicit_internal_provider
            .attribute_prototype_id()
            .ok_or(InternalProviderError::EmptyAttributePrototype)?;

        let identity_func = Func::find_by_attr(ctx, "name", &"si:identity")
            .await?
            .pop()
            .ok_or(EdgeError::IdentityFuncNotFound)?;
        let identity_func_arg =
            FuncArgument::find_by_name_for_func(ctx, "identity", *identity_func.id())
                .await?
                .ok_or(EdgeError::IdentityFuncArgNotFound)?;

        // Now, we can create the inter component attribute prototype argument.
        AttributePrototypeArgument::new_for_inter_component(
            ctx,
            *head_explicit_internal_provider_attribute_prototype,
            *identity_func_arg.id(),
            head_component_id,
            tail_component_id,
            *tail_external_provider.id(),
        )
        .await?;
        Ok(())
    }
}

// Reference: ( cd research/coreos/butane; make base64 )
const DUMMY_IGNITION_USER_DATA: &str = "\
ewogICJpZ25pdGlvbiI6IHsKICAgICJ2ZXJzaW9uIjogIjMuMy4wIgogIH0sCiAgInN5c3RlbWQi\
OiB7CiAgICAidW5pdHMiOiBbCiAgICAgIHsKICAgICAgICAiY29udGVudHMiOiAiW1VuaXRdXG5E\
ZXNjcmlwdGlvbj1XaGlza2Vyc1xuQWZ0ZXI9bmV0d29yay1vbmxpbmUudGFyZ2V0XG5XYW50cz1u\
ZXR3b3JrLW9ubGluZS50YXJnZXRcblxuW1NlcnZpY2VdXG5UaW1lb3V0U3RhcnRTZWM9MFxuRXhl\
Y1N0YXJ0UHJlPS0vYmluL3BvZG1hbiBraWxsIHdoaXNrZXJzMVxuRXhlY1N0YXJ0UHJlPS0vYmlu\
L3BvZG1hbiBybSB3aGlza2VyczFcbkV4ZWNTdGFydFByZT0vYmluL3BvZG1hbiBwdWxsIGRvY2tl\
ci5pby9zeXN0ZW1pbml0L3doaXNrZXJzXG5FeGVjU3RhcnQ9L2Jpbi9wb2RtYW4gcnVuIC0tbmFt\
ZSB3aGlza2VyczEgLS1wdWJsaXNoIDgwOjgwIGRvY2tlci5pby9zeXN0ZW1pbml0L3doaXNrZXJz\
XG5cbltJbnN0YWxsXVxuV2FudGVkQnk9bXVsdGktdXNlci50YXJnZXRcbiIsCiAgICAgICAgImVu\
YWJsZWQiOiB0cnVlLAogICAgICAgICJuYW1lIjogIndoaXNrZXJzLnNlcnZpY2UiCiAgICAgIH0K\
ICAgIF0KICB9Cn0K";

// NOTE(nick): please kill this, someone.
async fn butane_to_ec2_user_data_omega_hack(
    ctx: &DalContext,
    schema_id: SchemaId,
    component_id: ComponentId,
) -> EdgeResult<()> {
    let head_component = Component::get_by_id(ctx, &component_id)
        .await?
        .ok_or(ComponentError::NotFound(component_id))
        .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?;
    let head_schema_variant = head_component
        .schema_variant(ctx)
        .await
        .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?
        .ok_or(ComponentError::SchemaVariantNotFound)
        .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?;
    let mut maybe_user_data_prop = None;
    for prop in head_schema_variant
        .all_props(ctx)
        .await
        .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?
    {
        if prop.name() == "UserData" {
            maybe_user_data_prop = Some(prop);
            break;
        }
    }
    if let Some(user_data_prop) = maybe_user_data_prop {
        let read_context = AttributeReadContext {
            prop_id: Some(*user_data_prop.id()),
            schema_id: Some(schema_id),
            schema_variant_id: Some(*head_schema_variant.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };
        if let Some(attribute_value) = AttributeValue::find_for_context(ctx, read_context)
            .await
            .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?
        {
            if let Some(parent) = attribute_value
                .parent_attribute_value(ctx)
                .await
                .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?
            {
                let context = AttributeContextBuilder::from(read_context)
                    .to_context()
                    .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?;
                AttributeValue::update_for_context(
                    ctx,
                    *attribute_value.id(),
                    Some(*parent.id()),
                    context,
                    Some(serde_json::json![DUMMY_IGNITION_USER_DATA]),
                    None,
                )
                .await
                .map_err(|e| EdgeError::OmegaHack(format!("{e}")))?;
            }
        }
    }
    Ok(())
}
