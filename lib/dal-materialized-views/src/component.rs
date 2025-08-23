use dal::{
    Component,
    ComponentId,
    DalContext,
    Prop,
    Schema,
    SchemaId,
    SchemaVariant,
    component::diff::DiffStatus,
    prop::PropPath,
    qualification::QualificationSummary,
    workspace_snapshot::traits::component::ComponentExt,
};
use si_frontend_mv_types::component::{
    Component as ComponentMv,
    ComponentDiffStatus,
    ComponentInList as ComponentInListMv,
    ComponentTextDiff,
    SchemaMembers,
};
use telemetry::prelude::*;

pub mod attribute_tree;
pub mod component_diff;
pub mod erased_components;

#[instrument(
    name = "dal_materialized_views.component_in_list",
    level = "debug",
    skip_all,
    fields()
)]
pub async fn assemble_in_list(
    ctx: DalContext,
    component_id: ComponentId,
) -> crate::Result<ComponentInListMv> {
    let ctx = &ctx;

    let name = Component::name_by_id(ctx, component_id).await?;
    let color = Component::color_by_id(ctx, component_id).await?;

    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id).await?;
    let has_resource = Component::resource_by_id(ctx, component_id)
        .await?
        .is_some();
    let qualification_totals = QualificationSummary::individual_stats(ctx, component_id)
        .await?
        .into();
    let input_count = ctx
        .workspace_snapshot()?
        .external_source_count(component_id)
        .await?;
    let has_socket_connections = ctx
        .workspace_snapshot()?
        .has_socket_connections(component_id)
        .await?;

    let diff_status = map_diff_status(Component::has_diff_from_head(ctx, component_id).await?);
    let to_delete = Component::is_set_to_delete(ctx, component_id)
        .await?
        .unwrap_or(false);

    let prop_path_raw = ["root", "si", "resourceId"];
    let mut resource_id = None;
    if has_resource {
        resource_id = if let Some(prop_id) =
            Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &PropPath::new(prop_path_raw))
                .await?
        {
            let av_id_for_prop_id =
                Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
            dal::AttributeValue::view(ctx, av_id_for_prop_id).await?
        } else {
            None
        };
    }

    Ok(ComponentInListMv {
        id: component_id,
        name,
        color,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id,
        schema_variant_name: schema_variant.display_name().to_owned(),
        schema_category: schema_variant.category().to_owned(),
        has_resource,
        qualification_totals,
        input_count,
        diff_status,
        to_delete,
        resource_id,
        has_socket_connections,
    })
}

pub fn map_diff_status(status: dal::component::diff::DiffStatus) -> ComponentDiffStatus {
    match status {
        DiffStatus::Added => ComponentDiffStatus::Added,
        DiffStatus::None => ComponentDiffStatus::None,
        DiffStatus::Modified => ComponentDiffStatus::Modified,
    }
}

#[instrument(
    name = "dal_materialized_views.component",
    level = "debug",
    skip_all,
    fields()
)]
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> crate::Result<ComponentMv> {
    let ctx = &ctx;

    let name = Component::name_by_id(ctx, component_id).await?;
    let color = Component::color_by_id(ctx, component_id).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id).await?;
    let has_resource = Component::resource_by_id(ctx, component_id)
        .await?
        .is_some();
    let qualification_totals = QualificationSummary::individual_stats(ctx, component_id)
        .await?
        .into();
    let input_count = ctx
        .workspace_snapshot()?
        .external_source_count(component_id)
        .await?;
    let resource_diff = {
        let dal_component_diff = Component::get_diff(ctx, component_id).await?;
        let diff = match dal_component_diff.diff {
            Some(code_view) => code_view.code,
            None => None,
        };
        ComponentTextDiff {
            current: dal_component_diff.current.code,
            diff,
        }
    };
    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, schema_variant_id).await?;
    let to_delete = Component::is_set_to_delete(ctx, component_id)
        .await?
        .unwrap_or(false);

    Ok(ComponentMv {
        id: component_id,
        name,
        color,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id: schema_variant_id.into(),
        schema_variant_name: schema_variant.display_name().to_owned(),
        schema_category: schema_variant.category().to_owned(),
        schema_variant_description: schema_variant.description().to_owned(),
        schema_variant_doc_link: schema_variant.link().to_owned(),
        has_resource,
        qualification_totals,
        schema_members: schema.id().into(),
        input_count,
        resource_diff,
        is_secret_defining,
        to_delete,
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
