use axum::extract::{Json, Query};
use dal::{
    socket::{DiagramKind, SocketEdgeKind, SocketId},
    ExternalProviderId, InternalProviderId, Schema, SchemaId, SchemaVariant, SchemaVariantId,
    Socket, Visibility, Workspace, WorkspaceSnapshot,
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
    category: String,
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

    let mut schema_variants_views: Vec<SchemaVariantView> = Vec::new();
    let schemas = Schema::list(&ctx).await?;

    for schema in schemas {
        if schema.ui_hidden {
            continue;
        }

        let schema_variants = SchemaVariant::list_for_schema(&ctx, schema.id()).await?;
        for schema_variant in schema_variants {
            if schema_variant.ui_hidden() {
                continue;
            }

            let mut input_sockets = Vec::new();
            let mut output_sockets = Vec::new();

            let (input_sockets_with_providers, output_sockets_with_providers) =
                Socket::list_for_schema_variant(&ctx, schema_variant.id()).await?;

            for (input_socket, internal_provider) in input_sockets_with_providers {
                input_sockets.push(InputSocketView {
                    id: input_socket.id(),
                    name: input_socket.name().to_owned(),
                    diagram_kind: input_socket.diagram_kind(),
                    provider: InputProviderView {
                        id: internal_provider.id(),
                        ty: input_socket.name().to_owned(),
                    },
                })
            }

            for (output_socket, external_provider) in output_sockets_with_providers {
                output_sockets.push(OutputSocketView {
                    id: output_socket.id(),
                    name: output_socket.name().to_owned(),
                    diagram_kind: output_socket.diagram_kind(),
                    provider: OutputProviderView {
                        id: external_provider.id(),
                        ty: output_socket.name().to_owned(),
                    },
                })
            }

            schema_variants_views.push(SchemaVariantView {
                id: schema_variant.id(),
                // FIXME(nick): use the real value here
                builtin: true,
                // builtin: schema_variant.is_builtin(&ctx).await?,
                name: schema_variant.name().to_owned(),
                schema_id: schema.id(),
                schema_name: schema.name.to_owned(),
                color: schema_variant.color().to_owned(),
                category: schema_variant.category().to_owned(),
                // FIXME(nick): use the real value here
                // color: schema_variant
                //     .color(&ctx)
                //     .await?
                //     .unwrap_or_else(|| "00b0bc".to_owned()),
                input_sockets,
                output_sockets,
            });
        }
    }

    Ok(Json(schema_variants_views))
}
