use std::collections::{
    HashMap,
    VecDeque,
};

use dal::{
    DalContext,
    Prop,
    PropId,
    SchemaVariant,
    SchemaVariantId,
    property_editor::schema::WidgetKind,
};
use si_frontend_mv_types::schema_variant::prop_tree::{
    Prop as PropMv,
    PropKind,
    PropTree,
    PropTreeInfo,
    PropWidgetKind,
    WidgetOption,
    WidgetOptions,
};
use telemetry::prelude::*;

/// Generates a [`PropTree`] MV.
#[instrument(
    name = "dal_materialized_views.assemble_prop_tree"
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    schema_variant_id: SchemaVariantId,
) -> crate::Result<PropTree> {
    let ctx = &ctx;

    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;

    let mut props = HashMap::new();
    let mut tree_info = HashMap::new();

    let mut work_queue = VecDeque::from([root_prop_id]);

    while let Some(prop_id) = work_queue.pop_front() {
        let maybe_parent_prop_id = Prop::parent_prop_id_by_id(ctx, prop_id).await?;
        let child_prop_ids: Vec<PropId> = Prop::direct_child_prop_ids_ordered(ctx, prop_id).await?;

        work_queue.extend(&child_prop_ids);
        tree_info.insert(
            prop_id,
            PropTreeInfo {
                parent: maybe_parent_prop_id,
                children: child_prop_ids,
            },
        );

        let prop_mv = assemble_prop(ctx.clone(), prop_id, schema_variant_id).await?;

        props.insert(prop_id, prop_mv);
    }

    Ok(PropTree { props, tree_info })
}

#[instrument(
    name = "dal_materialized_views.assemble_prop"
    level = "debug",
    skip_all
)]
pub async fn assemble_prop(
    ctx: DalContext,
    prop_id: PropId,
    schema_variant_id: SchemaVariantId,
) -> crate::Result<PropMv> {
    let ctx = &ctx;

    let prop = Prop::get_by_id(ctx, prop_id).await?;

    let mut is_create_only = false;
    let filtered_widget_options = prop.widget_options.clone().map(|options| {
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
    });
    let default_can_be_set_by_socket = !prop.input_socket_sources(ctx).await?.is_empty();
    let secret_definition = crate::secret::find_definition(ctx, schema_variant_id, prop_id).await?;

    let path = prop.path(ctx).await?.with_replaced_sep("/");
    let prop_mv = PropMv {
        id: prop_id,
        path: path.to_owned(),
        name: prop.name,
        kind: match prop.kind {
            dal::PropKind::Array => PropKind::Array,
            dal::PropKind::Boolean => PropKind::Boolean,
            dal::PropKind::Integer => PropKind::Integer,
            dal::PropKind::Json => PropKind::Json,
            dal::PropKind::Map => PropKind::Map,
            dal::PropKind::Object => PropKind::Object,
            dal::PropKind::String => PropKind::String,
            dal::PropKind::Float => PropKind::Float,
        },
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
        is_origin_secret: secret_definition.is_some(),
        secret_definition,
        create_only: is_create_only,
        hidden: prop.hidden,
        eligible_for_connection: {
            // props can receive data if they're on a certain part of the prop tree
            path == ("root/si/name")
                || path.starts_with("root/domain")
                || path.starts_with("root/secrets")
                || path.starts_with("root/resource_value")
        },
    };
    Ok(prop_mv)
}
