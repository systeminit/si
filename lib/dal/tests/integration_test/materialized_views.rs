use std::collections::HashSet;

use dal::{
    AttributeValue,
    Component,
    DalContext,
    Func,
    Prop,
    SchemaVariant,
    action::{
        Action,
        prototype::ActionPrototype,
    },
    prop::PropPath,
    qualification::QualificationSummary,
};
use dal_test::{
    Result,
    helpers::{
        attribute::value,
        create_component_for_default_schema_name_in_default_view,
    },
    prelude::{
        ChangeSetTestHelpers,
        OptionExt,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_events::{
    ActionKind,
    ActionState,
};
use si_frontend_mv_types::{
    action::ActionView,
    component::{
        Component as ComponentMv,
        ComponentDiff,
        ComponentList,
    },
    incoming_connections::Connection,
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
async fn attribute_tree(ctx: &DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "swifty").await?;

    // NOTE(nick): right now, this test basically just makes sure this does not regress and
    // provides a psuedo-benchmark for generating MVs for a new component.
    dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), component.id())
        .await?;

    Ok(())
}

#[test]
async fn schema_variant(ctx: &DalContext) -> Result<()> {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "swifty").await?;

    // NOTE(brit): right now, this test basically just makes sure this does not regress and
    // provides a psuedo-benchmark for generating MVs for a schema.
    dal_materialized_views::schema_variant::assemble(ctx.clone(), schema_variant_id).await?;

    Ok(())
}

#[test]
async fn component(ctx: &DalContext) -> Result<()> {
    let components = dal_materialized_views::component_list::assemble(ctx.clone()).await?;
    assert_eq!(
        ComponentList {
            id: ctx.workspace_pk()?,
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
    let stats = QualificationSummary::individual_stats(ctx, created_component.id())
        .await?
        .into();

    let resource_diff = ComponentDiff {
        current: Some(String::from(
            "{\n  \"si\": {\n    \"name\": \"starfield\",\n    \"type\": \"component\",\n    \"color\": \"#ffffff\"\n  }\n}",
        )),
        diff: Some(String::from(
            "+{\n+  \"si\": {\n+    \"name\": \"starfield\",\n+    \"type\": \"component\",\n+    \"color\": \"#ffffff\"\n+  }\n+}",
        )),
    };

    let sv_id = created_component.schema_variant(ctx).await?.id();

    let attribute_tree = dal_materialized_views::component::attribute_tree::assemble(
        ctx.clone(),
        created_component.id(),
    )
    .await?;
    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, sv_id).await?;
    let can_be_upgraded = Component::can_be_upgraded_by_id(ctx, created_component.id()).await?;

    assert_eq!(
        ComponentMv {
            id: created_component.id(),
            name: component_name.to_owned(),
            color: created_component.color(ctx).await?.to_owned(),
            schema_name: schema_name.to_owned(),
            schema_id: schema.id(),
            schema_variant_id: sv_id.into(),
            schema_variant_name: schema_variant.display_name().to_owned(),
            schema_category: schema_variant.category().to_owned(),
            schema_variant_description: schema_variant.description().to_owned(),
            schema_variant_doc_link: schema_variant.link().to_owned(),
            has_resource: false,
            qualification_totals: stats,
            input_count: 0,
            diff_count: 0,
            resource_diff,
            attribute_tree,
            is_secret_defining,
            can_be_upgraded,
        }, // expected
        component // actual
    );

    Ok(())
}

// FIXME(nick): this test used to handle socket-to-socket connections, but now that sockets are dead, it has
// become much simpler. We should do two things to make this test better: 1) make the original connections with
// the eventual replacement to sockets (alpha "two" to beta "two" and beta "one" to charlie "one") and 2) use
// management functions provided by the lego schemas.
#[test]
async fn incoming_connections(ctx: &mut DalContext) -> Result<()> {
    // Create all components.
    let alpha =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "alpha")
            .await?;
    let beta =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "beta")
            .await?;
    let charlie =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "charlie")
            .await?;

    // Cache everything with need for making subscriptions (as well as for assertions later).
    let alpha_schema_variant_id = Component::schema_variant_id(ctx, alpha.id()).await?;
    let beta_schema_variant_id = Component::schema_variant_id(ctx, beta.id()).await?;
    let charlie_schema_variant_id = alpha_schema_variant_id;

    let alpha_si_name_prop_path = PropPath::new(["root", "si", "name"]);
    let charlie_si_name_prop_path = alpha_si_name_prop_path.clone();
    let beta_domain_name_prop_path = PropPath::new(["root", "domain", "name"]);
    let charlie_domain_name_prop_path = beta_domain_name_prop_path.clone();

    let alpha_si_name_prop_id =
        Prop::find_prop_id_by_path(ctx, alpha_schema_variant_id, &alpha_si_name_prop_path).await?;
    let charlie_si_name_prop_id =
        Prop::find_prop_id_by_path(ctx, charlie_schema_variant_id, &charlie_si_name_prop_path)
            .await?;
    let beta_domain_name_prop_id =
        Prop::find_prop_id_by_path(ctx, beta_schema_variant_id, &beta_domain_name_prop_path)
            .await?;
    let charlie_domain_name_prop_id = Prop::find_prop_id_by_path(
        ctx,
        charlie_schema_variant_id,
        &charlie_domain_name_prop_path,
    )
    .await?;

    let alpha_si_name_attribute_value_id =
        Component::attribute_value_for_prop_id(ctx, alpha.id(), alpha_si_name_prop_id).await?;
    let charlie_si_name_attribute_value_id =
        Component::attribute_value_for_prop_id(ctx, charlie.id(), charlie_si_name_prop_id).await?;
    let beta_domain_name_attribute_value_id =
        Component::attribute_value_for_prop_id(ctx, beta.id(), beta_domain_name_prop_id).await?;
    let charlie_domain_name_attribute_value_id =
        Component::attribute_value_for_prop_id(ctx, charlie.id(), charlie_domain_name_prop_id)
            .await?;

    let (_, alpha_si_name_attribute_value_path) =
        AttributeValue::path_from_root(ctx, alpha_si_name_attribute_value_id).await?;
    let (_, charlie_si_name_attribute_value_path) =
        AttributeValue::path_from_root(ctx, charlie_si_name_attribute_value_id).await?;
    let (_, beta_domain_name_attribute_value_path) =
        AttributeValue::path_from_root(ctx, beta_domain_name_attribute_value_id).await?;
    let (_, charlie_domain_name_attribute_value_path) =
        AttributeValue::path_from_root(ctx, charlie_domain_name_attribute_value_id).await?;

    // Check that the attribue value paths look as we expect for the subscriptions.
    {
        assert_eq!(
            "/si/name",                                  // expected
            alpha_si_name_attribute_value_path.as_str()  // actual
        );
        assert_eq!(
            "/si/name",                                    // expected
            charlie_si_name_attribute_value_path.as_str()  // actual
        );
        assert_eq!(
            "/domain/name",                                 // expected
            beta_domain_name_attribute_value_path.as_str()  // actual
        );
        assert_eq!(
            "/domain/name",                                    // expected
            charlie_domain_name_attribute_value_path.as_str()  // actual
        );
    }

    // Perform all connections and commit. This is the core of the test!
    {
        value::subscribe(
            ctx,
            charlie_si_name_attribute_value_id,
            [(alpha.id(), alpha_si_name_attribute_value_path.as_str())],
        )
        .await?;
        value::subscribe(
            ctx,
            charlie_domain_name_attribute_value_id,
            [(beta.id(), beta_domain_name_attribute_value_path.as_str())],
        )
        .await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    }

    // Check the alpha MV.
    {
        let alpha_mv =
            dal_materialized_views::incoming_connections::assemble(ctx.clone(), alpha.id()).await?;
        assert_eq!(
            alpha.id(),  // expected
            alpha_mv.id  // actual
        );
        assert!(alpha_mv.connections.is_empty());
    }

    // Check the beta MV.
    {
        let beta_mv =
            dal_materialized_views::incoming_connections::assemble(ctx.clone(), beta.id()).await?;
        assert_eq!(
            beta.id(),  // expected
            beta_mv.id  // actual
        );
        assert!(beta_mv.connections.is_empty());
    }

    // Check the charlie MV.
    {
        let charlie_mv =
            dal_materialized_views::incoming_connections::assemble(ctx.clone(), charlie.id())
                .await?;
        assert_eq!(
            charlie.id(),  // expected
            charlie_mv.id  // actual
        );
        assert_eq!(
            vec![
                Connection::Prop {
                    from_component_id: alpha.id().into(),
                    from_attribute_value_id: alpha_si_name_attribute_value_id,
                    from_attribute_value_path: alpha_si_name_attribute_value_path,
                    from_prop_id: alpha_si_name_prop_id,
                    from_prop_path: alpha_si_name_prop_path.with_replaced_sep_and_prefix("/"),
                    to_component_id: charlie.id().into(),
                    to_attribute_value_id: charlie_si_name_attribute_value_id,
                    to_attribute_value_path: charlie_si_name_attribute_value_path,
                    to_prop_id: charlie_si_name_prop_id,
                    to_prop_path: charlie_si_name_prop_path.with_replaced_sep_and_prefix("/"),
                },
                Connection::Prop {
                    from_component_id: beta.id().into(),
                    from_attribute_value_id: beta_domain_name_attribute_value_id,
                    from_attribute_value_path: beta_domain_name_attribute_value_path,
                    from_prop_id: beta_domain_name_prop_id,
                    from_prop_path: beta_domain_name_prop_path.with_replaced_sep_and_prefix("/"),
                    to_component_id: charlie.id().into(),
                    to_attribute_value_id: charlie_domain_name_attribute_value_id,
                    to_attribute_value_path: charlie_domain_name_attribute_value_path,
                    to_prop_id: charlie_domain_name_prop_id,
                    to_prop_path: charlie_domain_name_prop_path.with_replaced_sep_and_prefix("/"),
                },
            ], // expected
            charlie_mv.connections // actual
        );
    }

    Ok(())
}
