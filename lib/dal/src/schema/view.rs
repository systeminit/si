//! This module contains [`SchemaView`], which is used by the frontend to know and organize all non-hidden
//! [`Schemas`](Schema) and [`SchemaVariants`](Schema) in the current [`snapshot`](crate::WorkspaceSnapshot).

use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::Timestamp;
use thiserror::Error;

use crate::{
    DalContext,
    InputSocketId,
    OutputSocketId,
    Schema,
    SchemaError,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    schema::variant::{
        SchemaVariantError,
        root_prop::component_type::ComponentType,
    },
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaViewError {
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

pub type SchemaViewResult<T> = Result<T, SchemaViewError>;

pub type SchemaViews = HashMap<SchemaId, SchemaView>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaView {
    id: SchemaId,
    builtin: bool,
    name: String,

    variants: Vec<SchemaVariantView>,
}

impl SchemaView {
    pub async fn list(ctx: &DalContext) -> SchemaViewResult<Vec<Self>> {
        let mut schema_views = Vec::new();
        let schemas = Schema::list(ctx).await?;

        for schema in schemas {
            if schema.ui_hidden {
                continue;
            }

            let mut schema_variant_views = Vec::new();
            let default_variant_id = Schema::default_variant_id(ctx, schema.id).await?;
            let schema_variants = SchemaVariant::list_for_schema(ctx, schema.id).await?;
            for schema_variant in schema_variants {
                if schema_variant.ui_hidden() {
                    continue;
                }

                let (output_sockets, input_sockets) =
                    SchemaVariant::list_all_sockets(ctx, schema_variant.id()).await?;

                schema_variant_views.push(SchemaVariantView {
                    id: schema_variant.id(),
                    // FIXME(nick): use the real value here
                    builtin: true,
                    is_default: schema_variant.id() == default_variant_id,
                    name: schema_variant.version().to_owned(),
                    color: schema_variant.get_color(ctx).await?,
                    category: schema_variant.category().to_owned(),
                    component_type: schema_variant.component_type().to_owned(),
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
                    timestamp: schema_variant.timestamp(),
                    description: schema_variant.description(),
                    display_name: Some(schema_variant.display_name().to_string()),
                    is_locked: schema_variant.is_locked(),
                });
            }

            schema_views.push(Self {
                id: schema.id,
                // FIXME(nick): use the real value here
                builtin: true,
                name: schema.name,
                variants: schema_variant_views,
            });
        }

        Ok(schema_views)
    }

    pub fn id(&self) -> SchemaId {
        self.id
    }

    pub fn variants(&self) -> &Vec<SchemaVariantView> {
        &self.variants
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantView {
    id: SchemaVariantId,
    builtin: bool,
    name: String,
    is_default: bool,
    component_type: ComponentType,

    color: String,
    category: String,
    input_sockets: Vec<InputSocketView>,
    output_sockets: Vec<OutputSocketView>,
    #[serde(flatten)]
    timestamp: Timestamp,
    description: Option<String>,
    display_name: Option<String>,
    is_locked: bool,
}

impl SchemaVariantView {
    pub fn id(&self) -> SchemaVariantId {
        self.id
    }
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
