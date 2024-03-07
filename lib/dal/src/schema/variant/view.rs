//! This module contains [`SchemaVariantView`], which is used by the frontend to know and organize the
//! [`SchemaVariants`](SchemaVariant) in the current [`snapshot`](crate::WorkspaceSnapshot).

use serde::{Deserialize, Serialize};

use crate::schema::variant::SchemaVariantResult;
use crate::{
    DalContext, InputSocketId, OutputSocketId, Schema, SchemaId, SchemaVariant, SchemaVariantId,
};

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSocketView {
    id: OutputSocketId,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSocketView {
    id: InputSocketId,
    name: String,
}

impl SchemaVariantView {
    pub async fn list(ctx: &DalContext) -> SchemaVariantResult<Vec<Self>> {
        let mut views = Vec::new();

        let schemas = Schema::list(ctx).await?;

        for schema in schemas {
            if schema.ui_hidden {
                continue;
            }

            let schema_variants = SchemaVariant::list_for_schema(ctx, schema.id()).await?;
            for schema_variant in schema_variants {
                if schema_variant.ui_hidden() {
                    continue;
                }

                let (output_sockets, input_sockets) =
                    SchemaVariant::list_all_sockets(ctx, schema_variant.id()).await?;

                views.push(SchemaVariantView {
                    id: schema_variant.id(),
                    // FIXME(nick): use the real value here
                    builtin: true,
                    // builtin: schema_variant.is_builtin(&ctx).await?,
                    name: schema_variant.name().to_owned(),
                    schema_id: schema.id(),
                    schema_name: schema.name.to_owned(),
                    // TODO(nick): we should probably centralize and standardize this color logic.
                    color: schema_variant
                        .get_color(ctx)
                        .await?
                        .unwrap_or("#0F0F0F".into()),
                    category: schema_variant.category().to_owned(),
                    input_sockets: input_sockets
                        .iter()
                        .map(|s| InputSocketView {
                            id: s.id(),
                            name: s.name().to_owned(),
                        })
                        .collect(),
                    output_sockets: output_sockets
                        .iter()
                        .map(|s| OutputSocketView {
                            id: s.id(),
                            name: s.name().to_owned(),
                        })
                        .collect(),
                });
            }
        }

        Ok(views)
    }
}
