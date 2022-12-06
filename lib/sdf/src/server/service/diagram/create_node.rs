use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::diagram::DiagramError::ExternalProviderNotFoundForSocket;
use crate::service::diagram::{DiagramError, DiagramResult};
use crate::service::schema::SchemaError;
use axum::Json;
use dal::job::definition::DependentValuesUpdate;
use dal::node::NodeId;
use dal::socket::SocketEdgeKind;
use dal::{
    generate_name, node_position::NodePositionView, Component, Connection, DiagramKind,
    NodePosition, NodeTemplate, NodeView, Schema, SchemaId, StandardModel, Visibility, WorkspaceId,
};
use dal::{AttributeReadContext, AttributeValue, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub schema_id: SchemaId,
    pub parent_id: Option<NodeId>,
    pub x: String,
    pub y: String,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeResponse {
    pub node: NodeView,
}

pub async fn create_node(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateNodeRequest>,
) -> DiagramResult<Json<CreateNodeResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let name = generate_name();
    let schema = Schema::get_by_id(&ctx, &request.schema_id)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    if diagram_kind != DiagramKind::Configuration {
        return Err(DiagramError::InvalidDiagramKind(diagram_kind));
    }

    let (component, node) =
        Component::new_for_schema_variant_with_node(&ctx, &name, schema_variant_id).await?;

    let component_id = *component.id();

    let node_template = NodeTemplate::new_from_schema_id(&ctx, request.schema_id).await?;

    let (width, height) = {
        let sockets = component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?
            .sockets(&ctx)
            .await?;

        let mut size = (None, None);

        for s in sockets {
            if s.name() == "Frame" && *s.edge_kind() == SocketEdgeKind::ConfigurationInput {
                size = (Some("500".to_string()), Some("500".to_string()));
                break;
            }
        }

        size
    };

    let position = NodePosition::new(
        &ctx,
        *node.id(),
        diagram_kind,
        request.x.clone(),
        request.y.clone(),
        width,
        height,
    )
    .await?;
    let positions = vec![NodePositionView::from(position)];
    let node_view = NodeView::new(name, &node, component_id, positions, node_template);

    if let Some(parent_id) = request.parent_id {
        let frame = Component::find_for_node(&ctx, parent_id).await?;

        if let Some(f) = frame {
            let component_socket = {
                let sockets = component
                    .schema_variant(&ctx)
                    .await?
                    .ok_or(DiagramError::SchemaVariantNotFound)?
                    .sockets(&ctx)
                    .await?;

                let mut socket = None;

                for s in sockets {
                    if s.name() == "Frame" && *s.edge_kind() == SocketEdgeKind::ConfigurationOutput
                    {
                        socket = Some(s);
                        break;
                    }
                }

                match socket {
                    None => {
                        return Err(DiagramError::FrameSocketNotFound(*schema_variant_id));
                    }
                    Some(socket) => socket,
                }
            };

            let frame_socket = {
                let frame_schema_variant = f
                    .schema_variant(&ctx)
                    .await?
                    .ok_or(DiagramError::SchemaVariantNotFound)?;

                let sockets = frame_schema_variant.sockets(&ctx).await?;

                let mut socket = None;

                for s in sockets {
                    if s.name() == "Frame" && *s.edge_kind() == SocketEdgeKind::ConfigurationInput {
                        socket = Some(s);
                        break;
                    }
                }

                match socket {
                    None => {
                        return Err(DiagramError::FrameSocketNotFound(
                            *frame_schema_variant.id(),
                        ));
                    }
                    Some(socket) => socket,
                }
            };

            let _connection = Connection::new(
                &ctx,
                *node.id(),
                *component_socket.id(),
                parent_id,
                *frame_socket.id(),
            )
            .await?;

            {
                let frame_schema_variant = f
                    .schema_variant(&ctx)
                    .await?
                    .ok_or(DiagramError::SchemaVariantNotFound)?;

                let parent_sockets = frame_schema_variant.sockets(&ctx).await?;

                let component_sockets = component
                    .schema_variant(&ctx)
                    .await?
                    .ok_or(DiagramError::SchemaVariantNotFound)?
                    .sockets(&ctx)
                    .await?;

                for parent_socket in parent_sockets {
                    if parent_socket.name() != "Frame"
                        && *parent_socket.edge_kind() == SocketEdgeKind::ConfigurationOutput
                    {
                        let parent_provider = parent_socket
                            .external_provider(&ctx)
                            .await?
                            .ok_or_else(|| {
                                ExternalProviderNotFoundForSocket(*parent_socket.id())
                            })?;

                        for component_socket in &component_sockets {
                            if component_socket.name() == parent_socket.name()
                                && *component_socket.edge_kind()
                                    == SocketEdgeKind::ConfigurationInput
                            {
                                Connection::new(
                                    &ctx,
                                    parent_id,
                                    *parent_socket.id(),
                                    *node.id(),
                                    *component_socket.id(),
                                )
                                .await?;

                                let attribute_value_context = AttributeReadContext {
                                    component_id: Some(*f.id()),
                                    schema_variant_id: Some(*frame_schema_variant.id()),
                                    schema_id: Some(
                                        *frame_schema_variant
                                            .schema(&ctx)
                                            .await?
                                            .expect("Err")
                                            .id(),
                                    ),
                                    external_provider_id: Some(*parent_provider.id()),
                                    ..Default::default()
                                };

                                let attribute_value =
                                    AttributeValue::find_for_context(&ctx, attribute_value_context)
                                        .await?
                                        .ok_or(DiagramError::AttributeValueNotFoundForContext(
                                            attribute_value_context,
                                        ))?;

                                ctx.enqueue_job(DependentValuesUpdate::new(
                                    &ctx,
                                    *attribute_value.id(),
                                ))
                                .await;
                            }
                        }
                    }
                }
            }
        }
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateNodeResponse { node: node_view }))
}
