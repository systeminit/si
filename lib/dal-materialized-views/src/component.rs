use dal::{
    Component,
    ComponentId,
    DalContext,
    SchemaVariant,
    qualification::QualificationSummary,
};
use si_frontend_mv_types::component::{
    Component as ComponentMv,
    ComponentDiff,
};
use telemetry::prelude::*;

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

    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, schema_variant.id).await?;
    let attribute_tree = attribute_tree::assemble(ctx.to_owned(), component_id).await?;
    let input_count = attribute_tree
        .attribute_values
        .values()
        .filter(|value| value.is_from_external_source)
        .count();
    Ok(ComponentMv {
        id: component_id,
        name: Component::name_by_id(ctx, component_id).await?,
        color,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id: schema_variant.id.into(),
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
        is_secret_defining,
    })
}
