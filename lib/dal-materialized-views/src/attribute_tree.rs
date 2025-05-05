use std::collections::HashSet;

use dal::{
    AttributePrototype,
    AttributeValue,
    AttributeValueId,
    Component,
    DalContext,
    Func,
    InputSocketId,
    Prop,
    SchemaVariant,
    Secret,
    component::ControllingFuncData,
    prop::PropPath,
    property_editor::schema::WidgetKind,
    validation::ValidationOutputNode,
};
use si_frontend_types::newhotness::attribute_tree::{
    AttributeTree as AttributeTreeMv,
    AttributeValue as AttributeValueMv,
    Prop as PropMv,
    PropWidgetKind,
    ValidationOutput,
    WidgetOption,
    WidgetOptions,
};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.attribute_tree",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, id: AttributeValueId) -> super::Result<AttributeTreeMv> {
    let ctx = &ctx;
    // FIXME(nick): the controlling func data for this is wrong. We will need to adjust. Why not do it
    // at the time this PR is written? We are not sure what we need and don't need in the new attribute
    // panel in the newhotness.
    //
    // What is the context for this? This function is a port of the property editor methods... all of
    // them. They relied on work queues and cached information. There is a LOT of room for improvement
    // here, but this is meant to be a starting point.
    let av_id = id;
    let maybe_parent_av_id = AttributeValue::parent_attribute_value_id(ctx, av_id).await?;

    // NOTE(nick): every AV has to reference its own component. This is might be a good place to
    // split the macro component-relevant information into other MV(s).
    let component_id = AttributeValue::component_id(ctx, id).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let key = AttributeValue::key_for_id(ctx, av_id).await?;

    let sockets_on_component: HashSet<InputSocketId> =
        Component::incoming_connections_for_id(ctx, component_id)
            .await?
            .iter()
            .map(|c| c.to_input_socket_id)
            .collect();

    // FIXME(nick): we need to investigate all scenarios where this can be "None" and figure out
    // what to do. Maybe elements of arrays and maps should still have a populated prop field
    // within the MV, even if they don't have a direct edge to the prop node.
    let maybe_prop = AttributeValue::prop_opt(ctx, av_id).await?;

    let sockets_for_av = AttributeValue::list_input_socket_sources_for_id(ctx, av_id).await?;
    let can_be_set_by_socket = !sockets_for_av.is_empty();
    let is_from_external_source = sockets_for_av
        .iter()
        .any(|s| sockets_on_component.contains(s));

    let prototype_id = AttributeValue::prototype_id(ctx, av_id).await?;
    let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;

    // FIXME(nick): this is likely incorrect.
    let controlling_func = ControllingFuncData {
        func_id,
        av_id,
        is_dynamic_func: func.is_dynamic(),
    };

    // NOTE(nick): I ported Victor's comment.
    //
    // Note (victor): An attribute value is overridden if there is an attribute
    // prototype for this specific AV, which means it's set for the component,
    // not the schema variant. If the av is controlled, this check should be
    // made for its controlling AV.
    // This could be standalone func for AV, but we'd have to implement a
    // controlling_ancestors_for_av_id for av, instead of for the whole component.
    // Not a complicated task, but the PR that adds this has enough code as it is.
    let overridden = AttributeValue::component_prototype_id(ctx, controlling_func.av_id)
        .await?
        .is_some();

    let validation = ValidationOutputNode::find_for_attribute_value_id(ctx, av_id)
        .await?
        .map(|node| ValidationOutput {
            status: match node.validation.status {
                dal::validation::ValidationStatus::Pending => {
                    si_frontend_types::newhotness::attribute_tree::ValidationStatus::Pending
                }
                dal::validation::ValidationStatus::Error => {
                    si_frontend_types::newhotness::attribute_tree::ValidationStatus::Error
                }
                dal::validation::ValidationStatus::Failure => {
                    si_frontend_types::newhotness::attribute_tree::ValidationStatus::Failure
                }
                dal::validation::ValidationStatus::Success => {
                    si_frontend_types::newhotness::attribute_tree::ValidationStatus::Success
                }
            },
            message: node.validation.message,
        });

    // Get the value
    let mut value = match AttributeValue::get_by_id(ctx, av_id)
        .await?
        .value(ctx)
        .await?
    {
        Some(value) => value,
        None => match maybe_prop.clone() {
            Some(prop) => Prop::default_value(ctx, prop.id())
                .await?
                .unwrap_or(serde_json::Value::Null),
            None => serde_json::Value::Null,
        },
    };
    let secret_ids_by_key = Secret::list_ids_by_key(ctx).await?;
    let secrets_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "secrets"]).await?;

    // NOTE(nick): I ported this comment.
    //
    // If this is a secret, the JSON value has the secret key, not the secret id.
    // The editor needs the secret id, so we look in our mapto find which Secret in
    // the current graph has that key.
    if let Some(parent_av_id) = maybe_parent_av_id {
        if parent_av_id == secrets_av_id && value != serde_json::Value::Null {
            let secret_key = Secret::key_from_value_in_attribute_value(value)?;
            value = match secret_ids_by_key.get(&secret_key) {
                Some(secret_id) => serde_json::to_value(secret_id)?,
                None => {
                    // NOTE(nick): I ported this comment.
                    //
                    // If none of the secrets in the workspace have this key, we assume
                    // that dependent values haven't updated yet and will be fixed
                    // shortly. Thus we treat the property as missing for now and
                    // return null.
                    //
                    // This is an expected issue, so we don't warn--but it could trigger
                    // if something more serious is going on that is making the lookup
                    // fail more persistently, so we may want to measure how often it
                    // happens and figure out how to alert in that case.
                    warn!(
                        name: "Secret key does not match",
                        av_id = %av_id,
                        "Secret key in dependent value does not match any secret key; assuming that dependent values are not up to date and treating the property temporarily as missing",
                    );
                    serde_json::Value::Null
                }
            }
        }
    }

    let default_can_be_set_by_socket = if let Some(prop) = maybe_prop.clone() {
        !prop.input_socket_sources(ctx).await?.is_empty()
    } else {
        true
    };

    let mut is_create_only = false;
    let filtered_widget_options = if let Some(prop) = maybe_prop.clone() {
        prop.widget_options.map(|options| {
            options
                .into_iter()
                .filter(|option| {
                    if option.label() == "si_create_only_prop" {
                        is_create_only = true;
                        false
                    } else {
                        true
                    }
                })
                .map(|option| WidgetOption {
                    label: option.label,
                    value: option.value,
                })
                .collect::<WidgetOptions>()
        })
    } else {
        None
    };

    let origin_secret_prop_id = if SchemaVariant::is_secret_defining(ctx, schema_variant_id).await?
    {
        let output_socket =
            SchemaVariant::find_output_socket_for_secret_defining_id(ctx, schema_variant_id)
                .await?;
        Some(
            Prop::find_prop_id_by_path(
                ctx,
                schema_variant_id,
                &PropPath::new(["root", "secrets", output_socket.name()]),
            )
            .await?,
        )
    } else {
        None
    };

    let maybe_resource = Component::resource_by_id(ctx, component_id).await?;
    let has_resource = maybe_resource.is_some();

    let maybe_prop_view = if let Some(prop) = maybe_prop.clone() {
        Some(PropMv {
            id: prop.id,
            path: prop.path(ctx).await?.as_str().to_string(),
            name: prop.name,
            kind: prop.kind.into(),
            widget_kind: match prop.widget_kind {
                WidgetKind::Array => PropWidgetKind::Array,
                WidgetKind::Checkbox => PropWidgetKind::Checkbox,
                WidgetKind::CodeEditor => PropWidgetKind::CodeEditor,
                WidgetKind::Header => PropWidgetKind::Header,
                WidgetKind::Map => PropWidgetKind::Map,
                WidgetKind::Password => PropWidgetKind::Password,
                WidgetKind::Select => PropWidgetKind::Select {
                    options: filtered_widget_options,
                },
                WidgetKind::Color => PropWidgetKind::Color,
                WidgetKind::Secret => PropWidgetKind::Secret {
                    options: filtered_widget_options,
                },
                WidgetKind::Text => PropWidgetKind::Text,
                WidgetKind::TextArea => PropWidgetKind::TextArea,
                WidgetKind::ComboBox => PropWidgetKind::ComboBox {
                    options: filtered_widget_options,
                },
            },
            doc_link: prop.doc_link,
            documentation: prop.documentation,
            validation_format: prop.validation_format,
            default_can_be_set_by_socket,
            is_origin_secret: match origin_secret_prop_id {
                Some(prop_id) => prop_id == prop.id,
                None => false,
            },
            create_only: is_create_only && has_resource,
        })
    } else {
        None
    };

    let attribute_value = AttributeValueMv {
        id,
        key,
        value,
        can_be_set_by_socket,
        is_from_external_source,
        is_controlled_by_ancestor: controlling_func.av_id != av_id,
        is_controlled_by_dynamic_func: controlling_func.is_dynamic_func,
        overridden,
    };

    Ok(AttributeTreeMv {
        id,
        // FIXME(nick): we've gotten into scenarios where this can fail due to a missing edge to a
        // prop. We need to handle this error with more grace.
        #[allow(clippy::manual_unwrap_or_default)]
        children: match AttributeValue::get_child_av_ids_in_order(ctx, id).await {
            Ok(res) => res,
            Err(_) => Vec::new(),
        },
        parent: maybe_parent_av_id,
        prop: maybe_prop_view,
        attribute_value,
        validation,
    })
}
