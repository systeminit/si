use std::collections::{HashMap, VecDeque};

use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use hyper::http::Uri;
use serde::{Deserialize, Serialize};

use dal::edge::EdgeKind;
use dal::job::definition::DependentValuesUpdate;
use dal::socket::{SocketEdgeKind, SocketKind};
use dal::{
    AttributeReadContext, AttributeValue, ChangeSet, Component, ComponentError, ComponentId,
    Connection, DalContext, Edge, EdgeError, ExternalProvider, InternalProvider, SocketId,
    StandardModel, Visibility,
};
use dal::{ComponentType, Socket};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

use super::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionRequest {
    pub child_component_id: ComponentId,
    pub parent_component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionResponse {
    pub connection: Connection,
}

// Internal struct to track work-to-process in `connect_component_sockets_to_frame_inner`.
struct Work {
    parent_id: ComponentId,
    child_id: ComponentId,
}

// Create all valid connections between parent and child sockets
pub async fn connect_component_sockets_to_frame(
    ctx: &DalContext,
    parent_id: ComponentId,
    child_id: ComponentId,
    original_uri: &Uri,
    posthog_client: &crate::server::state::PosthogClient,
) -> DiagramResult<()> {
    // We stored connected sockets to ensure we connect children's sockets only to the nearest valid ancestor socket
    let mut connected_sockets_for_component_id: HashMap<ComponentId, Vec<SocketId>> =
        HashMap::new();

    connected_sockets_for_component_id
        .entry(child_id)
        .or_default();
    connected_sockets_for_component_id
        .entry(parent_id)
        .or_default();

    Connection::new_to_parent(ctx, child_id, parent_id).await?;

    connect_component_sockets_to_frame_inner(
        ctx,
        parent_id,
        child_id,
        original_uri,
        posthog_client,
        &mut connected_sockets_for_component_id,
    )
    .await?;

    // Now we have to propagate the children this child
    let mut sorted_children: VecDeque<(ComponentId, ComponentId)> =
        Edge::list_children_for_component(ctx, child_id)
            .await?
            .into_iter()
            .map(|grandchild_component_id| (child_id, grandchild_component_id))
            .collect();

    // Goes down new child's children list, trying to connect them to their new ancestors' Configuration Sockets
    while let Some((this_parent_id, this_child_id)) = sorted_children.pop_front() {
        connect_component_sockets_to_frame_inner(
            ctx,
            this_parent_id,
            this_child_id,
            original_uri,
            posthog_client,
            &mut HashMap::new(),
        )
        .await?;

        sorted_children.extend(
            Edge::list_children_for_component(ctx, this_child_id)
                .await?
                .into_iter()
                .map(|grandchild_id| (this_child_id, grandchild_id)),
        );
    }

    Ok(())
}

async fn connect_component_sockets_to_frame_inner(
    ctx: &DalContext,
    parent_id: ComponentId,
    child_id: ComponentId,
    original_uri: &Uri,
    posthog_client: &crate::server::state::PosthogClient,
    connected_sockets_for_component_id: &mut HashMap<ComponentId, Vec<SocketId>>,
) -> DiagramResult<()> {
    let mut work_stack = Vec::new();

    work_stack.push(Work {
        parent_id,
        child_id,
    });

    while let Some(Work {
        parent_id,
        child_id,
    }) = work_stack.pop()
    {
        connect_component_sockets_to_frame_inner_work(
            ctx,
            parent_id,
            child_id,
            original_uri,
            posthog_client,
            connected_sockets_for_component_id,
            &mut work_stack,
        )
        .await?;
    }

    Ok(())
}

async fn connect_component_sockets_to_frame_inner_work(
    ctx: &DalContext,
    parent_component_id: ComponentId,
    child_component_id: ComponentId,
    original_uri: &Uri,
    posthog_client: &crate::server::state::PosthogClient,
    connected_sockets_for_component_id: &mut HashMap<ComponentId, Vec<SocketId>>,
    work_stack: &mut Vec<Work>,
) -> Result<(), DiagramError> {
    let parent_component = Component::get_by_id(ctx, &parent_component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let parent_sockets = Socket::list_for_component(ctx, *parent_component.id()).await?;

    let child_component = Component::get_by_id(ctx, &child_component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    for parent_socket in &parent_sockets {
        if parent_socket.kind() == &SocketKind::Frame {
            continue;
        }

        match parent_component.get_type(ctx).await? {
            component_type @ ComponentType::Component => {
                return Err(DiagramError::InvalidComponentTypeForFrame(component_type));
            }
            ComponentType::AggregationFrame => {
                match *parent_socket.edge_kind() {
                    SocketEdgeKind::ConfigurationInput => {
                        let provider =
                            InternalProvider::find_explicit_for_socket(ctx, *parent_socket.id())
                                .await?
                                .ok_or(EdgeError::InternalProviderNotFoundForSocket(
                                    *parent_socket.id(),
                                ))?;

                        // We don't want to connect the provider when we are not using configuration edge kind
                        Edge::connect_internal_providers_for_components(
                            ctx,
                            *provider.id(),
                            *child_component.id(),
                            *parent_component.id(),
                        )
                        .await?;

                        Edge::new(
                            ctx,
                            EdgeKind::Configuration,
                            *child_component.id(),
                            *parent_socket.id(),
                            *parent_component.id(),
                            *parent_socket.id(),
                        )
                        .await?;

                        let attribute_value_context = AttributeReadContext {
                            component_id: Some(*parent_component.id()),
                            internal_provider_id: Some(*provider.id()),
                            ..Default::default()
                        };

                        let attribute_value =
                            AttributeValue::find_for_context(ctx, attribute_value_context)
                                .await?
                                .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                    attribute_value_context,
                                ))?;

                        ctx.enqueue_job(DependentValuesUpdate::new(
                            ctx.access_builder(),
                            *ctx.visibility(),
                            vec![*attribute_value.id()],
                        ))
                        .await?;
                    }
                    SocketEdgeKind::ConfigurationOutput => {
                        let provider = ExternalProvider::find_for_socket(ctx, *parent_socket.id())
                            .await?
                            .ok_or(EdgeError::ExternalProviderNotFoundForSocket(
                                *parent_socket.id(),
                            ))?;

                        Edge::connect_external_providers_for_components(
                            ctx,
                            *provider.id(),
                            *parent_component.id(),
                            *child_component.id(),
                        )
                        .await?;

                        Edge::new(
                            ctx,
                            EdgeKind::Configuration,
                            parent_component_id,
                            *parent_socket.id(),
                            child_component_id,
                            *parent_socket.id(),
                        )
                        .await?;

                        let attribute_value_context = AttributeReadContext {
                            component_id: Some(*child_component.id()),
                            external_provider_id: Some(*provider.id()),
                            ..Default::default()
                        };

                        let attribute_value =
                            AttributeValue::find_for_context(ctx, attribute_value_context)
                                .await?
                                .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                    attribute_value_context,
                                ))?;

                        ctx.enqueue_job(DependentValuesUpdate::new(
                            ctx.access_builder(),
                            *ctx.visibility(),
                            vec![*attribute_value.id()],
                        ))
                        .await?;
                    }
                }
            }
            component_type @ (ComponentType::ConfigurationFrameDown
            | ComponentType::ConfigurationFrameUp) => {
                let child_sockets = Socket::list_for_component(ctx, *child_component.id()).await?;

                for child_socket in &child_sockets {
                    // Skip child sockets corresponding to frames.
                    if child_socket.kind() == &SocketKind::Frame {
                        continue;
                    }

                    // Configuration frames down and up behave similarly, only connecting either child to parent vs
                    // parent to child sockets. So we assign destination and source entities here
                    let (
                        source_component_id,
                        source_socket,
                        dest_component_id,
                        destination_component_id,
                        dest_socket,
                    ) = if component_type == ComponentType::ConfigurationFrameDown {
                        let used_socket = connected_sockets_for_component_id
                            .get(&child_component_id)
                            .into_iter()
                            .flatten()
                            .any(|socket_id| child_socket.id() == socket_id);
                        if used_socket {
                            continue;
                        }

                        (
                            parent_component_id,
                            parent_socket,
                            child_component_id,
                            *child_component.id(),
                            child_socket,
                        )
                    } else {
                        (
                            child_component_id,
                            child_socket,
                            parent_component_id,
                            *parent_component.id(),
                            parent_socket,
                        )
                    };

                    if let (Some(source_provider), Some(dest_provider)) = (
                        source_socket.external_provider(ctx).await?,
                        dest_socket.internal_provider(ctx).await?,
                    ) {
                        // TODO(victor): Refactor to match on connection annotations.
                        if source_provider.name() == dest_provider.name() {
                            connected_sockets_for_component_id
                                .entry(dest_component_id)
                                .or_default()
                                .push(*dest_socket.id());

                            Connection::new(
                                ctx,
                                source_component_id,
                                *source_socket.id(),
                                dest_component_id,
                                *dest_socket.id(),
                                EdgeKind::Configuration,
                            )
                            .await?;

                            let dest_socket_internal_provider =
                                InternalProvider::find_explicit_for_socket(ctx, *dest_socket.id())
                                    .await?
                                    .ok_or(DiagramError::InternalProviderNotFoundForSocket(
                                        *dest_socket.id(),
                                    ))?;

                            let dest_attribute_value_context = AttributeReadContext {
                                internal_provider_id: Some(*dest_socket_internal_provider.id()),
                                component_id: Some(destination_component_id),
                                ..Default::default()
                            };
                            let mut dest_attribute_value =
                                AttributeValue::find_for_context(ctx, dest_attribute_value_context)
                                    .await?
                                    .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                        dest_attribute_value_context,
                                    ))?;

                            dest_attribute_value
                                .update_from_prototype_function(ctx)
                                .await?;

                            ctx.enqueue_job(DependentValuesUpdate::new(
                                ctx.access_builder(),
                                *ctx.visibility(),
                                vec![*dest_attribute_value.id()],
                            ))
                            .await?;
                        }
                    }
                }
            }
        };
    }

    if let Some(grandparent_id) =
        Edge::get_parent_for_component(ctx, *parent_component.id()).await?
    {
        let grandparent = Component::get_by_id(ctx, &grandparent_id)
            .await?
            .ok_or(ComponentError::NotFound(grandparent_id))?;
        let ty = grandparent.get_type(ctx).await?;

        match ty {
            ComponentType::Component => {}
            ComponentType::ConfigurationFrameDown | ComponentType::ConfigurationFrameUp => {
                work_stack.push(Work {
                    parent_id: *grandparent.id(),
                    child_id: child_component_id,
                });
            }
            ComponentType::AggregationFrame => unimplemented!(),
        }
    }

    {
        let child_schema = child_component
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        let parent_schema = parent_component
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        let from_socket = Socket::find_frame_socket_for_component(
            ctx,
            child_component_id,
            SocketEdgeKind::ConfigurationOutput,
        )
        .await?;

        let to_socket = Socket::find_frame_socket_for_component(
            ctx,
            parent_component_id,
            SocketEdgeKind::ConfigurationInput,
        )
        .await?;

        track(
            posthog_client,
            ctx,
            original_uri,
            "component_connected_to_frame",
            serde_json::json!({
                        "parent_component_id": parent_component.id(),
                        "parent_component_schema_name": parent_schema.name(),
                        "parent_socket_id": to_socket.id(),
                        "parent_socket_name": to_socket.name(),
                        "child_component_id": child_component.id(),
                        "child_component_schema_name": child_schema.name(),
                        "child_socket_id": from_socket.id(),
                        "child_socket_name": from_socket.name(),
            }),
        );
    }
    Ok(())
}

/// Create a [`Connection`] with a _to_ [`Socket`] and
/// [`Component`] and a _from_ [`Socket`] and [`Component`].
/// Creating a change set if on head.
pub async fn connect_component_to_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateFrameConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    // Detach from previous parent
    {
        let child_component = Component::get_by_id(&ctx, &request.child_component_id)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let maybe_parent = Edge::get_parent_for_component(&ctx, *child_component.id()).await?;

        if maybe_parent.is_some() {
            Edge::detach_component_from_parent(&ctx, *child_component.id()).await?;
        }
    }

    // Connect children to parent through frame edge
    connect_component_sockets_to_frame(
        &ctx,
        request.parent_component_id,
        request.child_component_id,
        &original_uri,
        &posthog_client,
    )
    .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response
        .header("content-type", "application/json")
        .body("{}".to_owned())?)
}
