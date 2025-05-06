use std::collections::HashSet;

use dal::{
    Component,
    DalContext,
    Func,
    action::{
        Action,
        prototype::ActionPrototype,
    },
    qualification::QualificationSummary,
};
use dal_test::{
    Result,
    helpers::create_component_for_default_schema_name_in_default_view,
    prelude::OptionExt,
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_events::{
    ActionKind,
    ActionState,
};
use si_frontend_types::{
    action::ActionView,
    newhotness::component::{
        Component as ComponentMv,
        ComponentList,
    },
    reference::ReferenceKind,
};

#[test]
async fn actions(ctx: &DalContext) -> Result<()> {
    let schema_name = "swifty";
    let component_name = "tes vi";
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component.id()).await?;

    // Gather what we need for the assertions after the component has been created.
    let create_action_prototype = ActionPrototype::for_variant(ctx, schema_variant_id)
        .await?
        .into_iter()
        .find(|ap| ap.kind == dal::action::prototype::ActionKind::Create)
        .ok_or_eyre("could not find action prototype")?;
    let func_id = ActionPrototype::func_id(ctx, create_action_prototype.id()).await?;
    let func = Func::get_by_id(ctx, func_id).await?;
    let create_action_id =
        Action::find_equivalent(ctx, create_action_prototype.id(), Some(component.id()))
            .await?
            .ok_or_eyre("action not found")?;
    let create_action = Action::get_by_id(ctx, create_action_id).await?;

    // Check the frontend payload for actions.
    let mut mv = dal_materialized_views::action_view_list::assemble(ctx.clone()).await?;
    let action_view = mv.actions.pop().ok_or_eyre("empty actions")?;
    assert!(mv.actions.is_empty(), "only one action should exist");
    assert_eq!(
        ActionView {
            id: create_action.id(),
            prototype_id: create_action_prototype.id,
            component_id: Some(component.id()),
            component_schema_name: Some(schema_name.to_owned()),
            component_name: Some(component_name.to_owned()),
            name: create_action_prototype.name.to_owned(),
            description: func.display_name,
            kind: ActionKind::Create,
            state: ActionState::Queued,
            originating_change_set_id: create_action.originating_changeset_id(),
            func_run_id: None,
            my_dependencies: Vec::new(),
            dependent_on: Vec::new(),
            hold_status_influenced_by: Vec::new(),
        }, // expected
        action_view // actual
    );

    // Check the frontend payload for action prototypes.
    let mv = dal_materialized_views::action_prototype_view_list::assemble(
        ctx.clone(),
        schema_variant_id,
    )
    .await?;
    assert_eq!(
        schema_variant_id, // expected
        mv.id              // actual
    );
    assert_eq!(
        4,                          // expected
        mv.action_prototypes.len()  // actual
    );
    let mut kinds = HashSet::new();
    for action_prototype_view in mv.action_prototypes {
        kinds.insert(action_prototype_view.kind);
    }
    assert_eq!(
        HashSet::from_iter([
            ActionKind::Create,
            ActionKind::Destroy,
            ActionKind::Refresh,
            ActionKind::Update
        ]), // expected
        kinds // actual
    );

    Ok(())
}

#[test]
async fn component(ctx: &DalContext) -> Result<()> {
    let components = dal_materialized_views::component_list::assemble(ctx.clone()).await?;
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

    let components = dal_materialized_views::component_list::assemble(ctx.clone()).await?;
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

    let component =
        dal_materialized_views::component::assemble(ctx.clone(), created_component.id()).await?;
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
            schema_variant_description: schema_variant.description().to_owned(),
            schema_variant_doc_link: schema_variant.link().to_owned(),
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
