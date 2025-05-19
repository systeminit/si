use dal::{
    DalContext,
    Prop,
    PropId,
    SchemaVariant,
    SecretError,
    prop::PropPath,
};
use si_frontend_mv_types::{
    schema_variant::prop_tree::{
        PropWidgetKind,
        WidgetOption,
        WidgetOptions,
    },
    secret::{
        SecretDefinition as SecretDefinitionMv,
        SecretFormDataView,
    },
};
use telemetry::prelude::*;

use crate::Error;

#[instrument(
    name = "dal_materialized_views.secret_definition.assemble",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, prop_id: PropId) -> crate::Result<SecretDefinitionMv> {
    let ctx = &ctx;

    // first let's check if this prop's variant is a secret defining variant
    let schema_variant_id = match Prop::schema_variant_id(ctx, prop_id).await? {
        Some(sv) => sv,
        None => {
            return Err(Error::Secret(SecretError::Prop(
                dal::prop::PropError::PropIsOrphan(prop_id),
            )));
        }
    };

    // This is probably unnecessary given how we're short circuiting in Edda..
    if !SchemaVariant::is_secret_defining(ctx, schema_variant_id).await? {
        return Err(Error::Secret(SecretError::SchemaVariantNotSecretDefining(
            schema_variant_id,
        )));
    }
    let secret_definition_path = PropPath::new(["root", "secret_definition"]);
    let maybe_secret_definition_prop_id =
        Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &secret_definition_path).await?;
    // now let's see if this prop is a secret definining prop
    if let Some(secret_definition_prop_id) = maybe_secret_definition_prop_id {
        if secret_definition_prop_id != prop_id {
            // This is also probably unnecessary given how we're short circuiting in Edda..
            // better safe than sorry?
            return Err(Error::Secret(SecretError::PropIdNotForSecret(prop_id)));
        }
    }

    // Now, find all the fields of the definition.
    let field_props = Prop::direct_child_props_ordered(ctx, prop_id).await?;

    // Assemble the form data views.
    let mut form_data_views = Vec::new();
    for field_prop in field_props {
        let widget_options = field_prop.widget_options.clone().map(|options| {
            options
                .into_iter()
                .map(|option| WidgetOption {
                    label: option.label,
                    value: option.value,
                })
                .collect::<WidgetOptions>()
        });
        form_data_views.push(SecretFormDataView {
            name: field_prop.name,
            kind: field_prop.kind.to_string(),
            widget_kind: PropWidgetKind::Secret {
                options: widget_options,
            },
        });
    }

    // Get the secret output socket corresponding to the definition. There should only be one
    // output socket as secret defining schema variants are required to have one and only one.
    let secret_output_socket =
        SchemaVariant::find_output_socket_for_secret_defining_id(ctx, schema_variant_id).await?;

    Ok(SecretDefinitionMv {
        id: prop_id,
        label: secret_output_socket.name().to_owned(),
        form_data: form_data_views,
    })
}
