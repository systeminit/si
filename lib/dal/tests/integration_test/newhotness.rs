use std::collections::VecDeque;

use dal::{
    Component,
    DalContext,
    qualification::QualificationSummary,
};
use dal_materialized_views::{
    attribute_tree,
    component,
};
use dal_test::{
    helpers::create_component_for_default_schema_name_in_default_view,
    prelude::*,
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_frontend_types::{
    newhotness::component::{
        Component as ComponentMv,
        ComponentList,
    },
    reference::ReferenceKind,
};

#[test]
async fn attribute_tree(ctx: &DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "swifty").await?;
    let root_attribute_value_id = Component::root_attribute_value_id(ctx, component.id()).await?;

    // NOTE(nick): right now, this test basically just makes sure this does not regress and
    // provides a psuedo-benchmark for generating MVs for a new component.
    let mut work_queue = VecDeque::from([root_attribute_value_id]);
    while let Some(attribute_value_id) = work_queue.pop_front() {
        let tree = attribute_tree::as_frontend_type(ctx, attribute_value_id).await?;
        work_queue.extend(tree.children);
    }
    Ok(())
}

#[test]
async fn component(ctx: &DalContext) -> Result<()> {
    let components = component::as_frontend_list_type(ctx).await?;
    assert_eq!(
        ComponentList {
            id: ctx.change_set_id(),
            components: Vec::new()
        }, // expected
        components // actual
    );

    let schema_name = "starfield";
    let component_name = schema_name;
    let created_component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await?;

    let components = component::as_frontend_list_type(ctx).await?;
    let reference = components
        .components
        .first()
        .ok_or_eyre("no components found")?;
    assert_eq!(
        ReferenceKind::Component, // expected
        reference.kind,           // actual
    );
    assert_eq!(
        created_component.id(), // expected
        reference.id.0          // actual
    );

    let component = component::as_frontend_type(ctx, created_component.id()).await?;
    let schema = created_component.schema(ctx).await?;
    let schema_variant = created_component.schema_variant(ctx).await?;
    let stats = QualificationSummary::individual_stats(ctx, created_component.id()).await?;

    let root_attribute_value_id =
        Component::root_attribute_value_id(ctx, created_component.id()).await?;
    let domain_attribute_value_id =
        Component::attribute_value_for_prop(ctx, created_component.id(), &["root", "domain"])
            .await?;
    let secrets_attribute_value_id =
        Component::attribute_value_for_prop(ctx, created_component.id(), &["root", "secrets"])
            .await?;
    let si_attribute_value_id =
        Component::attribute_value_for_prop(ctx, created_component.id(), &["root", "si"]).await?;
    let resource_value_attribute_value_id = Component::attribute_value_for_prop(
        ctx,
        created_component.id(),
        &["root", "resource_value"],
    )
    .await?;

    assert_eq!(
        ComponentMv {
            id: created_component.id(),
            name: component_name.to_owned(),
            schema_name: schema_name.to_owned(),
            schema_id: schema.id(),
            schema_variant_id: schema_variant.id(),
            schema_variant_name: schema_variant.display_name().to_owned(),
            schema_category: schema_variant.category().to_owned(),
            has_resource: false,
            qualification_totals: stats,
            input_count: 0,
            output_count: 0,
            diff_count: 0,
            root_attribute_value_id,
            domain_attribute_value_id,
            secrets_attribute_value_id,
            si_attribute_value_id,
            resource_value_attribute_value_id,
        }, // expected
        component // actual
    );

    Ok(())
}
