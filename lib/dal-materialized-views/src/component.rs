use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    InputSocket,
    attribute::prototype::argument::AttributePrototypeArgument,
    qualification::QualificationSummary,
};
use si_frontend_mv_types::component::{
    Component as ComponentMv,
    ComponentDiff,
};
use telemetry::prelude::*;

use crate::schema_variant;

pub mod attribute_tree;

#[instrument(name = "dal_materialized_views.component", level = "debug", skip_all)]
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> crate::Result<ComponentMv> {
    let ctx = &ctx;
    let schema_variant = Component::schema_variant_for_component_id(ctx, component_id).await?;
    let schema = schema_variant.schema(ctx).await?;
    let has_resource = Component::resource_by_id(ctx, component_id)
        .await?
        .is_some();
    let stats = QualificationSummary::individual_stats(ctx, component_id)
        .await?
        .into();

    // TODO(Wendy) - There is probably a better way to do this
    let input_socket_ids =
        InputSocket::list_ids_for_schema_variant(ctx, schema_variant.id()).await?;
    let mut input_count = 0;
    for input_socket_id in input_socket_ids {
        let attribute_value_id =
            InputSocket::component_attribute_value_id(ctx, input_socket_id, component_id).await?;
        let attribute_prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let attribute_prototype_argument_ids =
            AttributePrototype::list_arguments(ctx, attribute_prototype_id).await?;
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            let attribute_prototype_argument =
                AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id).await?;
            if let Some(targets) = attribute_prototype_argument.targets() {
                if targets.destination_component_id == component_id {
                    input_count += 1;
                }
            }
        }
    }

    let diff_count = Component::get_diff_count(ctx, component_id).await?;
    let color = Component::color_by_id(ctx, component_id).await?;

    let dal_component_diff = Component::get_diff(ctx, component_id).await?;
    let diff = match dal_component_diff.diff {
        Some(code_view) => code_view.code,
        None => None,
    };
    let resource_diff = ComponentDiff {
        current: dal_component_diff.current.code,
        diff,
    };

    let sv = schema_variant::assemble(ctx.to_owned(), schema_variant.id).await?;

    let attribute_tree = attribute_tree::assemble(ctx.to_owned(), component_id).await?;

    Ok(ComponentMv {
        id: component_id,
        name: Component::name_by_id(ctx, component_id).await?,
        color,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id: (&sv).into(),
        schema_variant_name: schema_variant.display_name().to_owned(),
        schema_category: schema_variant.category().to_owned(),
        schema_variant_description: schema_variant.description().to_owned(),
        schema_variant_doc_link: schema_variant.link().to_owned(),
        has_resource,
        qualification_totals: stats,
        input_count,
        diff_count,
        attribute_tree,
        resource_diff,
    })
}
