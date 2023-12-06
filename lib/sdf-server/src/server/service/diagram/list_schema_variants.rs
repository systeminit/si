use axum::extract::{Json, Query};
use dal::{
    socket::{SocketEdgeKind, SocketId},
    DiagramKind, ExternalProvider, ExternalProviderId, InternalProvider, InternalProviderId,
    SchemaId, SchemaVariant, SchemaVariantId, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaVariantsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ProviderMetadata = String;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputProviderView {
    id: ExternalProviderId,
    ty: ProviderMetadata,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSocketView {
    id: SocketId,
    name: String,
    diagram_kind: DiagramKind,
    provider: OutputProviderView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputProviderView {
    id: InternalProviderId,
    ty: ProviderMetadata,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSocketView {
    id: SocketId,
    name: String,
    diagram_kind: DiagramKind,
    provider: InputProviderView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantView {
    id: SchemaVariantId,
    builtin: bool,
    name: String,
    schema_name: String,
    schema_id: SchemaId,
    color: String,
    input_sockets: Vec<InputSocketView>,
    output_sockets: Vec<OutputSocketView>,
}
pub type ListSchemaVariantsResponse = Vec<SchemaVariantView>;

pub async fn list_schema_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSchemaVariantsRequest>,
) -> DiagramResult<Json<ListSchemaVariantsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variants = SchemaVariant::list(&ctx).await?;
    let external_provider_by_socket = ExternalProvider::by_socket(&ctx).await?;
    let internal_provider_by_socket = InternalProvider::by_socket(&ctx).await?;

    let mut variants_view = Vec::with_capacity(variants.len());
    for variant in variants {
        if variant.ui_hidden() {
            continue;
        }

        let schema = variant
            .schema(&ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        if schema.ui_hidden() {
            continue;
        }
        let mut input_sockets = Vec::new();
        let mut output_sockets = Vec::new();

        let sockets = variant.sockets(&ctx).await?;

        for socket in sockets {
            match socket.edge_kind() {
                SocketEdgeKind::ConfigurationOutput => {
                    let provider =
                        external_provider_by_socket
                            .get(socket.id())
                            .ok_or_else(|| {
                                DiagramError::ExternalProviderNotFoundForSocket(*socket.id())
                            })?;
                    output_sockets.push(OutputSocketView {
                        id: *socket.id(),
                        name: socket.name().to_owned(),
                        diagram_kind: *socket.diagram_kind(),
                        provider: OutputProviderView {
                            id: *provider.id(),
                            ty: socket.name().to_owned(),
                        },
                    })
                }
                SocketEdgeKind::ConfigurationInput => {
                    let provider =
                        internal_provider_by_socket
                            .get(socket.id())
                            .ok_or_else(|| {
                                DiagramError::InternalProviderNotFoundForSocket(*socket.id())
                            })?;
                    input_sockets.push(InputSocketView {
                        id: *socket.id(),
                        name: socket.name().to_owned(),
                        diagram_kind: *socket.diagram_kind(),
                        provider: InputProviderView {
                            id: *provider.id(),
                            ty: socket.name().to_owned(),
                        },
                    })
                }
            }
        }

        variants_view.push(SchemaVariantView {
            id: *variant.id(),
            builtin: variant.is_builtin(&ctx).await?,
            name: variant.name().to_owned(),
            schema_id: *schema.id(),
            schema_name: schema.name().to_owned(),
            input_sockets,
            color: variant
                .color(&ctx)
                .await?
                .unwrap_or_else(|| "00b0bc".to_owned()),
            output_sockets,
        });
    }

    Ok(Json(variants_view))
}
