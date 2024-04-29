use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prop::{PropError, PropPath};
use crate::property_editor::schema::PropertyEditorPropWidgetKind;
use crate::schema::variant::root_prop::RootPropChild;
use crate::{DalContext, Prop, PropId, SchemaVariant, SchemaVariantError, SchemaVariantId};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretDefinitionViewError {
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

type SecretDefinitionViewResult<T> = Result<T, SecretDefinitionViewError>;

/// A view of the definition of a [`Secret`](crate::Secret).
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SecretDefinitionView {
    /// The name of the [`Prop`] that corresponds to the secret definition.
    pub secret_definition: String,
    form_data: Vec<SecretFormDataView>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
struct SecretFormDataView {
    name: String,
    kind: String,
    widget_kind: PropertyEditorPropWidgetKind,
}

impl SecretDefinitionView {
    /// Assembles [`views`](SecretDefinitionView) for all secret definitions in the
    /// [`snapshot`](crate::WorkspaceSnapshot).
    pub async fn list(ctx: &DalContext) -> SecretDefinitionViewResult<Vec<Self>> {
        let schema_variant_ids = SchemaVariant::list_ids(ctx).await?;

        let secret_definition_path = PropPath::new(["root", "secret_definition"]);
        let mut views = Vec::new();

        for schema_variant_id in schema_variant_ids {
            let maybe_secret_definition_prop_id =
                Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &secret_definition_path)
                    .await?;

            // We have found a schema variant with a secret definition!
            if let Some(secret_definition_prop_id) = maybe_secret_definition_prop_id {
                let view =
                    Self::assemble(ctx, schema_variant_id, secret_definition_prop_id).await?;
                views.push(view);
            }
        }

        Ok(views)
    }

    async fn assemble(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        secret_definition_prop_id: PropId,
    ) -> SecretDefinitionViewResult<Self> {
        // Now, find all the fields of the definition.
        let field_prop_ids = Prop::direct_child_prop_ids(ctx, secret_definition_prop_id).await?;

        // Assemble the form data views.
        let mut form_data_views = Vec::new();
        for field_prop_id in field_prop_ids {
            let field_prop = Prop::get_by_id_or_error(ctx, field_prop_id).await?;
            form_data_views.push(SecretFormDataView {
                name: field_prop.name,
                kind: field_prop.kind.to_string(),
                widget_kind: PropertyEditorPropWidgetKind::new(
                    field_prop.widget_kind,
                    field_prop.widget_options,
                ),
            });
        }

        // Get the name from the (hopefully) only child of secrets prop.
        let secrets_prop_id =
            SchemaVariant::find_root_child_prop_id(ctx, schema_variant_id, RootPropChild::Secrets)
                .await?;

        let entry_prop_id = Prop::direct_single_child_prop_id(ctx, secrets_prop_id).await?;
        let entry_prop = Prop::get_by_id_or_error(ctx, entry_prop_id).await?;

        Ok(Self {
            secret_definition: entry_prop.name,
            form_data: form_data_views,
        })
    }
}
