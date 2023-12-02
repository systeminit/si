use axum::extract::{Json, Query};
use dal::{
    ExternalProviderId, InternalProviderId, Schema, SchemaId, SchemaVariant, SchemaVariantId,
    Visibility, Workspace, WorkspaceSnapshot,
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

// TODO(nick): collapse this into the socket view.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputProviderView {
    id: ExternalProviderId,
    ty: ProviderMetadata,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSocketView {
    id: ExternalProviderId,
    name: String,
    provider: OutputProviderView,
}

// TODO(nick): collapse this into the socket view.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputProviderView {
    id: InternalProviderId,
    ty: ProviderMetadata,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSocketView {
    id: InternalProviderId,
    name: String,
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

            let (external_providers, explicit_internal_providers) =
                SchemaVariant::list_external_providers_and_explicit_internal_providers(
                    &ctx,
                    schema_variant.id(),
                )
                .await?;

            for explicit_internal_provider in explicit_internal_providers {
                input_sockets.push(InputSocketView {
                    id: explicit_internal_provider.id(),
                    name: explicit_internal_provider.name().to_owned(),
                    provider: InputProviderView {
                        id: explicit_internal_provider.id(),
                        ty: explicit_internal_provider.name().to_owned(),
                    },
                })
            }

            for external_provider in external_providers {
                output_sockets.push(OutputSocketView {
                    id: external_provider.id(),
                    name: external_provider.name().to_owned(),
                    provider: OutputProviderView {
                        id: external_provider.id(),
                        ty: external_provider.name().to_owned(),
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
                color: schema_variant
                    .get_color(&ctx)
                    .await?
                    .unwrap_or("#0F0F0F".into()),
                category: schema_variant.category().to_owned(),
                input_sockets,
                output_sockets,
            });
        }
    }

    Ok(Json(schema_variants_views))
}
