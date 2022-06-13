use axum::extract::{Json, Query};
use dal::{
    socket::{SocketEdgeKind, SocketId, SocketKind},
    ExternalProviderId, InternalProviderId, SchemaVariant, SchemaVariantId, SchematicKind,
    StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

use super::{SchematicError, SchematicResult};
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
    color: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSocketView {
    id: SocketId,
    name: String,
    schematic_kind: SchematicKind,
    provider: OutputProviderView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputProviderView {
    id: InternalProviderId,
    ty: ProviderMetadata,
    color: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSocketView {
    id: SocketId,
    name: String,
    schematic_kind: SchematicKind,
    provider: InputProviderView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantView {
    id: SchemaVariantId,
    name: String,
    schema_name: String,
    color: i64,
    input_sockets: Vec<InputSocketView>,
    output_sockets: Vec<OutputSocketView>,
}
pub type ListSchemaVariantsResponse = Vec<SchemaVariantView>;

pub async fn list_schema_variants(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSchemaVariantsRequest>,
) -> SchematicResult<Json<ListSchemaVariantsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let variants = SchemaVariant::list(&ctx).await?;

    let mut variants_view = Vec::with_capacity(variants.len());
    for variant in variants {
        let mut input_sockets = Vec::new();
        let mut output_sockets = Vec::new();

        let sockets = variant.sockets(&ctx).await?;
        for socket in sockets {
            match socket.kind() {
                SocketKind::Provider => match socket.edge_kind() {
                    SocketEdgeKind::Output => {
                        let provider = socket.external_provider(&ctx).await?.ok_or_else(|| {
                            SchematicError::ExternalProviderNotFoundForSocket(*socket.id())
                        })?;
                        output_sockets.push(OutputSocketView {
                            id: *socket.id(),
                            name: socket.name().to_owned(),
                            schematic_kind: *socket.schematic_kind(),
                            provider: OutputProviderView {
                                id: *provider.id(),
                                ty: socket.name().to_owned(),
                                color: socket.color().map_or(0x00b0bc, |c| *c),
                            },
                        })
                    }
                    SocketEdgeKind::Configures => {
                        let provider = socket.internal_provider(&ctx).await?.ok_or_else(|| {
                            SchematicError::InternalProviderNotFoundForSocket(*socket.id())
                        })?;
                        input_sockets.push(InputSocketView {
                            id: *socket.id(),
                            name: socket.name().to_owned(),
                            schematic_kind: *socket.schematic_kind(),
                            provider: InputProviderView {
                                id: *provider.id(),
                                ty: socket.name().to_owned(),
                                color: socket.color().map_or(0x00b0bc, |c| *c),
                            },
                        })
                    }
                    _ => continue,
                },
            }
        }

        variants_view.push(SchemaVariantView {
            id: *variant.id(),
            name: variant.name().to_owned(),
            schema_name: variant
                .schema(&ctx)
                .await?
                .ok_or(SchematicError::SchemaNotFound)?
                .name()
                .to_owned(),
            color: variant.color().map_or(0x00b0bc, |c| *c),
            input_sockets,
            output_sockets,
        });
    }
    Ok(Json(variants_view))
}
