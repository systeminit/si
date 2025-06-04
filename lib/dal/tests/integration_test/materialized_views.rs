use std::collections::HashSet;

use dal::{
    AttributeValue,
    Component,
    DalContext,
    Func,
    InputSocket,
    OutputSocket,
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
        connect_components_with_socket_names,
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
        connect_components_with_socket_names(ctx, alpha.id(), "two", beta.id(), "two").await?;
        connect_components_with_socket_names(ctx, beta.id(), "one", charlie.id(), "one").await?;
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

    // Cache everything else needed for assertions.
    let alpha_output_socket =
        OutputSocket::find_with_name_or_error(ctx, "two", alpha_schema_variant_id).await?;
    let beta_input_socket =
        InputSocket::find_with_name_or_error(ctx, "two", beta_schema_variant_id).await?;
    let beta_output_socket =
        OutputSocket::find_with_name_or_error(ctx, "one", beta_schema_variant_id).await?;
    let charlie_input_socket =
        InputSocket::find_with_name_or_error(ctx, "one", charlie_schema_variant_id).await?;

    let alpha_output_socket_attribute_value_id =
        OutputSocket::component_attribute_value_id(ctx, alpha_output_socket.id(), alpha.id())
            .await?;
    let beta_input_socket_attribute_value_id =
        InputSocket::component_attribute_value_id(ctx, beta_input_socket.id(), beta.id()).await?;
    let beta_output_socket_attribute_value_id =
        OutputSocket::component_attribute_value_id(ctx, beta_output_socket.id(), beta.id()).await?;
    let charlie_input_socket_attribute_value_id =
        InputSocket::component_attribute_value_id(ctx, charlie_input_socket.id(), charlie.id())
            .await?;

    let alpha_output_socket_attribute_value_path =
        AttributeValue::get_path_for_id(ctx, alpha_output_socket_attribute_value_id)
            .await?
            .expect("has input socket name");
    let beta_input_socket_attribute_value_path =
        AttributeValue::get_path_for_id(ctx, beta_input_socket_attribute_value_id)
            .await?
            .expect("has input socket name");
    let beta_output_socket_attribute_value_path =
        AttributeValue::get_path_for_id(ctx, beta_output_socket_attribute_value_id)
            .await?
            .expect("has input socket name");
    let charlie_input_socket_attribute_value_path =
        AttributeValue::get_path_for_id(ctx, charlie_input_socket_attribute_value_id)
            .await?
            .expect("has input socket name");

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
        assert_eq!(
            vec![Connection::Socket {
                from_component_id: alpha.id().into(),
                from_attribute_value_id: alpha_output_socket_attribute_value_id,
                from_attribute_value_path: alpha_output_socket_attribute_value_path,
                from_socket_id: alpha_output_socket.id(),
                from_socket_name: alpha_output_socket.name().to_owned(),
                to_component_id: beta.id().into(),
                to_socket_id: beta_input_socket.id(),
                to_socket_name: beta_input_socket.name().to_owned(),
                to_attribute_value_id: beta_input_socket_attribute_value_id,
                to_attribute_value_path: beta_input_socket_attribute_value_path,
            }], // expected
            beta_mv.connections // actual
        );
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
                Connection::Socket {
                    from_component_id: beta.id().into(),
                    from_attribute_value_id: beta_output_socket_attribute_value_id,
                    from_attribute_value_path: beta_output_socket_attribute_value_path,
                    from_socket_id: beta_output_socket.id(),
                    from_socket_name: beta_output_socket.name().to_owned(),
                    to_component_id: charlie.id().into(),
                    to_socket_id: charlie_input_socket.id(),
                    to_socket_name: charlie_input_socket.name().to_owned(),
                    to_attribute_value_id: charlie_input_socket_attribute_value_id,
                    to_attribute_value_path: charlie_input_socket_attribute_value_path,
                }
            ], // expected
            charlie_mv.connections // actual
        );
    }

    Ok(())
}
