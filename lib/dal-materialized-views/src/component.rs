use dal::{
    Component,
    ComponentId,
    DalContext,
    Schema,
    SchemaId,
    SchemaVariant,
    qualification::QualificationSummary,
    workspace_snapshot::traits::component::ComponentExt,
};
use si_frontend_mv_types::component::{
    Component as ComponentMv,
    ComponentDiff,
    ComponentInList as ComponentInListMv,
    SchemaMembers,
};
use telemetry::prelude::*;

pub mod attribute_tree;

// TODO(fnichol): demote back to `debug`
#[instrument(
    name = "dal_materialized_views.component_in_list",
    level = "debug",
    skip_all
)]
pub async fn assemble_in_list(
    ctx: DalContext,
    component_id: ComponentId,
) -> crate::Result<ComponentInListMv> {
    let ctx = &ctx;
    // TODO(fnichol): remove `.instrument()` calls
    let schema_variant_id = Component::schema_variant_id(ctx, component_id)
        // .instrument(info_span!("Component::schema_variant_id"))
        .await?;
    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id)
        // .instrument(info_span!("SchemaVariant::get_by_id"))
        .await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id)
        // .instrument(info_span!("SchemaVariant::schema_for_schema_variant_id"))
        .await?;
    let has_resource = Component::resource_by_id(ctx, component_id)
        // .instrument(info_span!("Component::resource_by_id"))
        .await?
        .is_some();
    let stats = QualificationSummary::individual_stats(ctx, component_id)
        // .instrument(info_span!("QualificationSummary::individual_stats"))
        .await?
        .into();

    let diff_count = Component::get_diff_count(ctx, component_id)
        // .instrument(info_span!("Component::get_diff_count"))
        .await?;
    let color = Component::color_by_id(ctx, component_id)
        // .instrument(info_span!("Component::color_by_id"))
        .await?;

    let input_count = ctx
        .workspace_snapshot()?
        .external_source_count(component_id)
        // .instrument(info_span!("WorkspaceSnapshot::external_source_count"))
        .await?;

    Ok(ComponentInListMv {
        id: component_id,
        name: Component::name_by_id(ctx, component_id)
            // .instrument(info_span!("Component::name_by_id"))
            .await?,
        color,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id,
        schema_variant_name: schema_variant.display_name().to_owned(),
        schema_category: schema_variant.category().to_owned(),
        has_resource,
        qualification_totals: stats,
        input_count,
        diff_count,
        to_delete: Component::is_set_to_delete(ctx, component_id)
            // .instrument(info_span!("Component::is_set_to_delete"))
            .await?
            .unwrap_or(false),
    })
}

// TODO(fnichol): demote back to `debug`
#[instrument(name = "dal_materialized_views.component", level = "debug", skip_all)]
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> crate::Result<ComponentMv> {
    let ctx = &ctx;
    // TODO(fnichol): remove `.instrument()` calls
    let schema_variant_id = Component::schema_variant_id(ctx, component_id)
        // .instrument(info_span!("Component::schema_variant_id"))
        .await?;
    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id)
        // .instrument(info_span!("SchemaVariant::get_by_id"))
        .await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id)
        // .instrument(info_span!("SchemaVariant::schema_for_schema_variant_id"))
        .await?;
    let has_resource = Component::resource_by_id(ctx, component_id)
        // .instrument(info_span!("Component::resource_by_id"))
        .await?
        .is_some();
    let stats = QualificationSummary::individual_stats(ctx, component_id)
        // .instrument(info_span!("QualificationSummary::individual_stats"))
        .await?
        .into();

    let diff_count = Component::get_diff_count(ctx, component_id)
        // .instrument(info_span!("Component::get_diff_count"))
        .await?;
    let color = Component::color_by_id(ctx, component_id)
        // .instrument(info_span!("Component::color_by_id"))
        .await?;

    let dal_component_diff = Component::get_diff(ctx, component_id)
        // .instrument(info_span!("Component::get_diff"))
        .await?;
    let diff = match dal_component_diff.diff {
        Some(code_view) => code_view.code,
        None => None,
    };
    let resource_diff = ComponentDiff {
        current: dal_component_diff.current.code,
        diff,
    };

    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, schema_variant_id)
        // .instrument(info_span!("SchemaVariant::is_secret_defining"))
        .await?;

    let input_count = ctx
        .workspace_snapshot()?
        .external_source_count(component_id)
        // .instrument(info_span!("WorkspaceSnapshot::external_source_count"))
        .await?;

    Ok(ComponentMv {
        id: component_id,
        name: Component::name_by_id(ctx, component_id)
            // .instrument(info_span!("Component::name_by_id"))
            .await?,
        color,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id: schema_variant_id.into(),
        schema_variant_name: schema_variant.display_name().to_owned(),
        schema_category: schema_variant.category().to_owned(),
        schema_variant_description: schema_variant.description().to_owned(),
        schema_variant_doc_link: schema_variant.link().to_owned(),
        has_resource,
        qualification_totals: stats,
        schema_members: schema.id().into(),
        input_count,
        diff_count,
        resource_diff,
        is_secret_defining,
        to_delete: Component::is_set_to_delete(ctx, component_id)
            // .instrument(info_span!("Component::is_set_to_delete"))
            .await?
            .unwrap_or(false),
    })
}

#[instrument(
    name = "dal_materialized_views.schema_members",
    level = "debug",
    skip_all
)]
pub async fn assemble_schema_members(
    ctx: DalContext,
    schema_id: SchemaId,
) -> crate::Result<SchemaMembers> {
    let ctx = &ctx;

    let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;
    let unlocked_variant_id = SchemaVariant::get_unlocked_for_schema(ctx, schema_id)
        .await?
        .map(|variant| variant.id());

    Ok(SchemaMembers {
        id: schema_id,
        default_variant_id,
        editing_variant_id: unlocked_variant_id,
    })
}
