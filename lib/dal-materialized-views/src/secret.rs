use dal::{
    DalContext,
    Prop,
    PropId,
    SchemaVariant,
    SchemaVariantId,
    Secret,
    SecretId,
    prop::PropPath,
    property_editor::schema::WidgetKind,
};
use si_frontend_mv_types::{
    schema_variant::prop_tree::{
        PropWidgetKind,
        WidgetOption,
        WidgetOptions,
    },
    secret::{
        Secret as SecretMv,
        SecretDefinition,
        SecretFormDataView,
    },
};
use telemetry::prelude::*;

#[instrument(name = "dal_materialized_views.secret", level = "debug", skip_all)]
pub async fn assemble(ctx: &DalContext, secret_id: SecretId) -> super::Result<SecretMv> {
    let secret = Secret::get_by_id(ctx, secret_id).await?;

    Ok(SecretMv {
        id: secret_id,
        name: secret.name().to_owned(),
        label: secret.definition().to_owned(),
        description: secret.description().to_owned(),
        is_usable: secret.can_be_decrypted(ctx).await?,
    })
}

#[instrument(name = "dal_materialized_views.secret", level = "debug", skip_all)]
pub async fn find_definition(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
    prop_id: PropId,
) -> super::Result<Option<SecretDefinition>> {
    // First let's see if this schema variant is secret defining.
    // If not, we need to go find the secret defining variant for any/all secrets this prop uses
    if !SchemaVariant::is_secret_defining(ctx, schema_variant_id).await? {
        return Ok(None);
    } else {
        let output_socket =
            SchemaVariant::find_output_socket_for_secret_defining_id(ctx, schema_variant_id)
                .await?;
        let secrets_prop_id = Prop::find_prop_id_by_path(
            ctx,
            schema_variant_id,
            &PropPath::new(["root", "secrets", output_socket.name()]),
        )
        .await?;
        // this prop is for the secret that this variant is defining! Let's get the definition
        if secrets_prop_id != prop_id {
            return Ok(None);
        } else {
            let secret_definition_path = PropPath::new(["root", "secret_definition"]);
            let secret_definition_prop_id =
                Prop::find_prop_id_by_path(ctx, schema_variant_id, &secret_definition_path).await?;

            // Now, find all the fields of the definition.
            let field_props =
                Prop::direct_child_props_ordered(ctx, secret_definition_prop_id).await?;

            // Assemble the form data views.
            let mut form_data_views = Vec::with_capacity(field_props.len());
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
                let widget_kind = match field_prop.widget_kind {
                    WidgetKind::Array => PropWidgetKind::Array,
                    WidgetKind::Checkbox => PropWidgetKind::Checkbox,
                    WidgetKind::CodeEditor => PropWidgetKind::CodeEditor,
                    WidgetKind::Header => PropWidgetKind::Header,
                    WidgetKind::Map => PropWidgetKind::Map,
                    WidgetKind::Password => PropWidgetKind::Password,
                    WidgetKind::Select => PropWidgetKind::Select {
                        options: widget_options.clone(),
                    },
                    WidgetKind::Color => PropWidgetKind::Color,
                    WidgetKind::Secret => PropWidgetKind::Secret {
                        options: widget_options.clone(),
                    },
                    WidgetKind::Text => PropWidgetKind::Text,
                    WidgetKind::TextArea => PropWidgetKind::TextArea,
                    WidgetKind::ComboBox => PropWidgetKind::ComboBox {
                        options: widget_options.clone(),
                    },
                };
                form_data_views.push(SecretFormDataView {
                    name: field_prop.name,
                    kind: field_prop.kind.to_string(),
                    widget_kind,
                });
            }
            Ok(Some(SecretDefinition {
                label: output_socket.name().to_string(),
                form_data: form_data_views,
            }))
        }
    }
}
