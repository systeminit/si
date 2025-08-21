use std::collections::HashSet;

use dal::{
    AttributeValue,
    Component,
    DalContext,
    Func,
    Prop,
    SchemaVariant,
    SchemaVariantId,
    action::{
        Action,
        prototype::ActionPrototype,
    },
    attribute::{
        path::AttributePath,
        prototype::argument::AttributePrototypeArgument,
        value::subscription::ValueSubscription,
    },
    prop::PropPath,
    qualification::QualificationSummary,
    workspace_snapshot::{
        graph::validator::connections::PropConnection,
        node_weight::reason_node_weight::Reason,
    },
};
use dal_materialized_views::component::map_diff_status;
use dal_test::{
    Result,
    helpers::{
        attribute::value,
        change_set,
        component::{
            self,
            ComponentKey,
        },
        connect_components_with_socket_names,
        create_component_for_default_schema_name_in_default_view,
        schema::variant,
    },
    prelude::{
        ChangeSetTestHelpers,
        OptionExt,
    },
    test,
};
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;
use si_events::{
    ActionKind,
    ActionState,
};
use si_frontend_mv_types::{
    action::ActionView,
    component::{
        Component as ComponentMv,
        ComponentDiffStatus,
        ComponentInList as ComponentInListMv,
        ComponentList,
        ComponentTextDiff,
    },
    incoming_connections::Connection,
    reference::ReferenceKind,
};
use si_id::AttributePrototypeArgumentId;

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
async fn cached_schema_variant_not_found(ctx: &DalContext) -> Result<()> {
    // Use a non-existent SchemaVariantId
    let non_existent_id = SchemaVariantId::new();

    // Should return an error for non-existent variant
    let result =
        dal_materialized_views::cached::schema::variant::assemble(ctx.clone(), non_existent_id)
            .await;
    assert!(
        result.is_err(),
        "Should return error for non-existent schema variant"
    );

    Ok(())
}

#[test]
async fn component_diff(ctx: &mut DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Value", kind: "string" },
                        { name: "Values", kind: "array",
                            entry: { name: "ValuesItem", kind: "string" },
                        },
                        { name: "ValueMap", kind: "map",
                            entry: { name: "ValueMapItem", kind: "string" },
                        },
                    ]
                };
            }
        "#,
    )
    .await?;

    // First, put components "modified" and "unchanged" into HEAD; then create a new changeset.
    component::create_and_set(
        ctx,
        "test",
        "modified",
        serde_json::from_str(
            r#"{
                "/domain/Value": "value",
                "/domain/Values/0": "value1",
                "/domain/Values/1": "value2",
                "/domain/ValueMap/foo": "fooValue",
                "/domain/ValueMap/bar": "barValue"
            }"#,
        )?,
    )
    .await?;
    component::create_and_set(
        ctx,
        "test",
        "unchanged",
        serde_json::from_str(
            r#"{
            "/domain/Value": "value",
            "/domain/Values/0": "value1",
            "/domain/Values/1": "value2",
            "/domain/ValueMap/foo": "fooValue",
            "/domain/ValueMap/bar": "barValue"
        }"#,
        )?,
    )
    .await?;
    change_set::commit(ctx).await?;
    change_set::apply_and_refork(ctx).await?;

    // First, empty diff should show nothing
    assert_component_diff_mv(ctx, "modified", "None", r#"{}"#).await;
    assert_component_diff_mv(ctx, "unchanged", "None", r#"{}"#).await;

    // Now remove, add and modify components
    component::create_and_set(
        ctx,
        "test",
        "added",
        serde_json::from_str(
            r#"{
                "/domain/Value": "value",
                "/domain/Values/0": "value1",
                "/domain/Values/1": "value2",
                "/domain/ValueMap/foo": "fooValue",
                "/domain/ValueMap/bar": "barValue"
            }"#,
        )?,
    )
    .await?;
    component::update(
        ctx,
        "modified",
        serde_json::from_str(
            r#"{
                "/domain/Value": "new_value",
                "/domain/Values/0": "new_value1",
                "/domain/Values/1": "new_value2",
                "/domain/ValueMap/foo": "new_fooValue",
                "/domain/ValueMap/bar": "new_barValue"
            }"#,
        )?,
    )
    .await?;
    change_set::commit(ctx).await?;

    serde_json::from_str::<si_frontend_mv_types::component::component_diff::AttributeSource>(
        r#"{ "value": "new_value" }"#,
    )?;
    serde_json::from_str::<si_frontend_mv_types::component::component_diff::AttributeSourceAndValue>(
        r#"{
            "$source": { "value": "new_value" },
            "$value": "new_value"
        }"#,
    )?;
    serde_json::from_str::<si_frontend_mv_types::component::component_diff::AttributeDiff>(
        r#"{
            "new": {
                "$source": { "value": "new_value" },
                "$value": "new_value"
            }
        }"#,
    )?;
    serde_json::from_str::<si_frontend_mv_types::component::component_diff::AttributeDiff>(
        r#"{
            "old": {
                "$source": { "value": "value" },
                "$value": "value"
            }
        }"#,
    )?;
    serde_json::from_str::<si_frontend_mv_types::component::component_diff::AttributeDiff>(
        r#"{
            "new": {
                "$source": { "value": "new_value" },
                "$value": "new_value"
            },
            "old": {
                "$source": { "value": "value" },
                "$value": "value"
            }
        }"#,
    )?;
    // Check the diffs for modified, unchanged and added components!
    assert_component_diff_mv(
        ctx,
        "modified",
        "Modified",
        r#"{
            "/domain/Value": {
                "new": {
                    "$source": { "value": "new_value" },
                    "$value": "new_value"
                },
                "old": {
                    "$source": { "value": "value" },
                    "$value": "value"
                }
            },
            "/domain/Values/0": {
                "new": {
                    "$source": { "value": "new_value1" },
                    "$value": "new_value1"
                },
                "old": {
                    "$source": { "value": "value1" },
                    "$value": "value1"
                }
            },
            "/domain/Values/1": {
                "new": {
                    "$source": { "value": "new_value2" },
                    "$value": "new_value2"
                },
                "old": {
                    "$source": { "value": "value2" },
                    "$value": "value2"
                }
            },
            "/domain/ValueMap/foo": {
                "new": {
                    "$source": { "value": "new_fooValue" },
                    "$value": "new_fooValue"
                },
                "old": {
                    "$source": { "value": "fooValue" },
                    "$value": "fooValue"
                }
            },
            "/domain/ValueMap/bar": {
                "new": {
                    "$source": { "value": "new_barValue" },
                    "$value": "new_barValue"
                },
                "old": {
                    "$source": { "value": "barValue" },
                    "$value": "barValue"
                }
            }
        }"#,
    )
    .await;

    assert_component_diff_mv(ctx, "unchanged", "None", r#"{}"#).await;

    // NOTE added components show way too much right now; we're just going to check the paths
    // and status for now
    let added_mv = component_diff_mv(ctx, "added").await?;
    assert_eq!(added_mv.id, component::id(ctx, "added").await?);
    assert_eq!(added_mv.diff_status, ComponentDiffStatus::Added);
    assert_eq!(
        vec![
            "",
            "/si",
            "/si/name",
            "/si/protected",
            "/si/type",
            "/si/color",
            "/si/resourceId",
            "/si/tags",
            "/domain",
            "/domain/Value",
            "/domain/Values",
            "/domain/Values/0",
            "/domain/Values/1",
            "/domain/ValueMap",
            "/domain/ValueMap/foo",
            "/domain/ValueMap/bar",
            "/secrets",
            "/resource",
            "/resource/status",
            "/resource/message",
            "/resource/payload",
            "/resource/last_synced",
            "/resource_value",
            "/code",
            "/qualification",
            "/deleted_at",
        ],
        added_mv
            .attribute_diffs
            .iter()
            .map(|(path, _)| path.as_str())
            .collect_vec()
    );
    // Check one of the values to make sure it's filled in right at least
    assert_eq!(
        Some(serde_json::from_str(
            r#"{
                "new": {
                    "$source": { "value": "value" },
                    "$value": "value"
                }
            }"#
        )?),
        added_mv
            .attribute_diffs
            .into_iter()
            .filter(|(path, _)| path == "/domain/Value")
            .map(|(_, diff)| diff)
            .next(),
    );

    Ok(())
}

async fn assert_component_diff_mv(
    ctx: &DalContext,
    component: impl ComponentKey,
    diff_status: &str,
    attribute_diffs: &str,
) {
    let component = component::id(ctx, component).await.unwrap();
    let expected = expected_component_diff_mv(ctx, component, diff_status, attribute_diffs)
        .await
        .unwrap();
    let actual = component_diff_mv(ctx, component).await.unwrap();
    assert_eq!(expected, actual);
}

// Generate the component diff MV, and return it as JSON
async fn component_diff_mv(
    ctx: &DalContext,
    component: impl ComponentKey,
) -> Result<si_frontend_mv_types::component::component_diff::ComponentDiff> {
    let component_id = component::id(ctx, component).await?;
    let mv = dal_materialized_views::component::component_diff::assemble(ctx.clone(), component_id)
        .await?;
    Ok(mv)
}

async fn expected_component_diff_mv(
    ctx: &DalContext,
    component: impl ComponentKey,
    diff_status: &str,
    attribute_diffs: &str,
) -> Result<si_frontend_mv_types::component::component_diff::ComponentDiff> {
    let id = component::id(ctx, component).await?;
    let json = format!(
        r#"{{
            "id": {},
            "diffStatus": {},
            "attributeDiffs": {}
        }}"#,
        serde_json::to_string(&id)?,
        serde_json::to_string(diff_status)?,
        attribute_diffs
    );
    println!("JSON {}", &json);
    Ok(serde_json::from_str(&json)?)
}

#[test]
async fn component(ctx: &DalContext) -> Result<()> {
    let components = dal_materialized_views::component_list::assemble(ctx.clone()).await?;
    assert_eq!(
        ComponentList {
            id: ctx.workspace_pk()?,
            components: Vec::new(),
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
        ReferenceKind::ComponentInList, // expected
        reference.kind,                 // actual
    );
    assert_eq!(
        created_component.id(), // expected
        reference.id.0          // actual
    );

    let component_in_list =
        dal_materialized_views::component::assemble_in_list(ctx.clone(), created_component.id())
            .await?;
    let component =
        dal_materialized_views::component::assemble(ctx.clone(), created_component.id()).await?;
    let schema = created_component.schema(ctx).await?;
    let schema_variant = created_component.schema_variant(ctx).await?;
    let stats = QualificationSummary::individual_stats(ctx, created_component.id())
        .await?
        .into();

    let resource_diff = ComponentTextDiff {
        current: Some(String::from(
            "{\n  \"si\": {\n    \"name\": \"starfield\",\n    \"type\": \"component\",\n    \"color\": \"#ffffff\"\n  }\n}",
        )),
        diff: Some(String::from(
            "+{\n+  \"si\": {\n+    \"name\": \"starfield\",\n+    \"type\": \"component\",\n+    \"color\": \"#ffffff\"\n+  }\n+}",
        )),
    };

    let sv_id = created_component.schema_variant(ctx).await?.id();

    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, sv_id).await?;

    let diff_status =
        map_diff_status(Component::has_diff_from_head(ctx, created_component.id()).await?);

    let has_resource = Component::resource_by_id(ctx, created_component.id())
        .await?
        .is_some();

    let prop_path_raw = ["root", "si", "resourceId"];
    let mut resource_id = None;
    if has_resource {
        resource_id = if let Some(prop_id) =
            Prop::find_prop_id_by_path_opt(ctx, schema_variant.id(), &PropPath::new(prop_path_raw))
                .await?
        {
            let av_id_for_prop_id =
                Component::attribute_value_for_prop_id(ctx, created_component.id(), prop_id)
                    .await?;
            dal::AttributeValue::view(ctx, av_id_for_prop_id).await?
        } else {
            None
        };
    }
    let qualification_totals = QualificationSummary::individual_stats(ctx, created_component.id())
        .await?
        .into();

    assert_eq!(
        ComponentMv {
            id: created_component.id(),
            name: component_name.to_owned(),
            color: created_component.color(ctx).await?.to_owned(),
            schema_name: schema_name.to_owned(),
            schema_id: schema.id(),
            schema_variant_id: sv_id.into(),
            schema_members: schema.id().into(),
            schema_variant_name: schema_variant.display_name().to_owned(),
            schema_category: schema_variant.category().to_owned(),
            schema_variant_description: schema_variant.description().to_owned(),
            schema_variant_doc_link: schema_variant.link().to_owned(),
            has_resource: false,
            qualification_totals: stats,
            input_count: 0,
            resource_diff,
            is_secret_defining,
            to_delete: false,
        }, // expected
        component // actual
    );

    assert_eq!(
        ComponentInListMv {
            id: created_component.id(),
            name: component_name.to_owned(),
            color: created_component.color(ctx).await?.to_owned(),
            schema_name: schema_name.to_owned(),
            schema_id: schema.id(),
            schema_variant_id: sv_id,
            schema_variant_name: schema_variant.display_name().to_owned(),
            schema_category: schema_variant.category().to_owned(),
            has_resource: false,
            qualification_totals,
            input_count: 0,
            to_delete: false,
            resource_id,
            diff_status,
            has_socket_connections: false,
        },
        component_in_list
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
    let echo =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "echo")
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
    // First let's connect these using socket connections - simple case
    {
        connect_components_with_socket_names(ctx, alpha.id(), "two", beta.id(), "two").await?;
        connect_components_with_socket_names(ctx, alpha.id(), "two", echo.id(), "two").await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    }

    // check alpha mv
    {
        let alpha_av_tree_mv =
            dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), alpha.id())
                .await?;

        let alpha_avs_with_sockets = alpha_av_tree_mv
            .attribute_values
            .values()
            .filter(|av| av.has_socket_connection)
            .collect_vec();
        assert!(alpha_avs_with_sockets.is_empty());
    }

    // check beta mv
    {
        let beta_av_tree_mv =
            dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), beta.id())
                .await?;

        let beta_avs_with_sockets = beta_av_tree_mv
            .attribute_values
            .values()
            .filter(|av| av.has_socket_connection)
            .collect_vec();
        // beta has exactly one av with a socket connection
        assert!(beta_avs_with_sockets.iter().exactly_one().is_ok());

        let beta_component_in_list =
            dal_materialized_views::component::assemble_in_list(ctx.clone(), beta.id()).await?;

        assert!(beta_component_in_list.has_socket_connections);
    }

    // check echo mv
    {
        let echo_av_tree_mv =
            dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), echo.id())
                .await?;
        // mostly concerned with the values for the "two" paths
        let echo_avs_with_sockets = echo_av_tree_mv
            .attribute_values
            .values()
            .filter(|av| av.has_socket_connection)
            .collect_vec();
        // echo has one socket connection too
        assert!(echo_avs_with_sockets.iter().exactly_one().is_ok());
        let echo_component_in_list =
            dal_materialized_views::component::assemble_in_list(ctx.clone(), echo.id()).await?;
        assert!(echo_component_in_list.has_socket_connections);
    }

    // Then let's migrate:
    {
        let snapshot = ctx.workspace_snapshot()?.as_legacy_snapshot()?;

        let inferred_connections = snapshot
            .inferred_connection_graph(ctx)
            .await?
            .inferred_connections_for_all_components(ctx)
            .await?
            .into_iter()
            .map(|connection| {
                dal::workspace_snapshot::graph::validator::connections::SocketConnection {
                    from: (connection.source_component_id, connection.output_socket_id),
                    to: (
                        connection.destination_component_id,
                        connection.input_socket_id,
                    ),
                }
            });

        let connection_migrations = snapshot.connection_migrations(inferred_connections).await?;
        for migration in connection_migrations {
            match &migration.issue {
                // If there's no issue, we can migrate.
                None => {}
                // If there's an issue, we can't migrate it.
                Some(_) => panic!("These should migrate cleanly"),
            }

            let mut did_something = false;
            for prop_connection in &migration.prop_connections {
                if add_prop_connection(ctx, prop_connection).await? {
                    did_something = true;
                }
            }

            if remove_socket_connection(ctx, migration.explicit_connection_id).await? {
                did_something = true;
            }
            assert!(did_something);
        }
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    }
    // Post migration, prop subscriptions should have replaced socket connections and the MVs update accordingly

    // Check the alpha MV.
    {
        let alpha_mv =
            dal_materialized_views::incoming_connections::assemble(ctx.clone(), alpha.id()).await?;
        assert_eq!(
            alpha.id(),  // expected
            alpha_mv.id  // actual
        );
        assert!(alpha_mv.connections.is_empty());
        let alpha_av_tree_mv =
            dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), alpha.id())
                .await?;
        // mostly concerned with the values for the "two" paths
        let alpha_avs_with_sockets = alpha_av_tree_mv
            .attribute_values
            .values()
            .filter(|av| av.has_socket_connection)
            .collect_vec();
        assert!(alpha_avs_with_sockets.is_empty());
    }

    // Check the beta MV.
    {
        let beta_mv =
            dal_materialized_views::incoming_connections::assemble(ctx.clone(), beta.id()).await?;
        assert_eq!(
            beta.id(),  // expected
            beta_mv.id  // actual
        );
        assert!(beta_mv.connections.iter().exactly_one().is_ok());

        let beta_av_tree_mv =
            dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), beta.id())
                .await?;
        let beta_avs_with_sockets = beta_av_tree_mv
            .attribute_values
            .values()
            .filter(|av| av.has_socket_connection)
            .collect_vec();
        assert!(beta_avs_with_sockets.is_empty());

        let beta_avs_with_subs = beta_av_tree_mv
            .attribute_values
            .values()
            .filter(
                |av: &&si_frontend_mv_types::component::attribute_tree::AttributeValue| {
                    av.external_sources.is_some()
                },
            )
            .collect_vec();
        assert!(beta_avs_with_subs.iter().exactly_one().is_ok());

        let beta_component_in_list =
            dal_materialized_views::component::assemble_in_list(ctx.clone(), beta.id()).await?;
        assert!(!beta_component_in_list.has_socket_connections);
    }
    // Check the echo MV.
    {
        let echo_mv =
            dal_materialized_views::incoming_connections::assemble(ctx.clone(), echo.id()).await?;
        assert_eq!(
            echo.id(),  // expected
            echo_mv.id  // actual
        );
        assert!(echo_mv.connections.iter().exactly_one().is_ok());

        let echo_av_tree_mv =
            dal_materialized_views::component::attribute_tree::assemble(ctx.clone(), echo.id())
                .await?;
        let echo_avs_with_sockets = echo_av_tree_mv
            .attribute_values
            .values()
            .filter(|av| av.has_socket_connection)
            .collect_vec();
        assert!(echo_avs_with_sockets.is_empty());
        let echo_avs_with_subs = echo_av_tree_mv
            .attribute_values
            .values()
            .filter(
                |av: &&si_frontend_mv_types::component::attribute_tree::AttributeValue| {
                    av.external_sources.is_some()
                },
            )
            .collect_vec();

        assert!(echo_avs_with_subs.iter().exactly_one().is_ok());
        let echo_component_in_list =
            dal_materialized_views::component::assemble_in_list(ctx.clone(), echo.id()).await?;
        assert!(!echo_component_in_list.has_socket_connections);
    }

    // but also add some manual prop connections for fun
    {
        value::subscribe(
            ctx,
            charlie_si_name_attribute_value_id,
            (alpha.id(), alpha_si_name_attribute_value_path.as_str()),
        )
        .await?;
        value::subscribe(
            ctx,
            charlie_domain_name_attribute_value_id,
            (beta.id(), beta_domain_name_attribute_value_path.as_str()),
        )
        .await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
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
                    from_component_id: alpha.id(),
                    from_attribute_value_id: alpha_si_name_attribute_value_id,
                    from_attribute_value_path: alpha_si_name_attribute_value_path,
                    from_prop_id: alpha_si_name_prop_id,
                    from_prop_path: alpha_si_name_prop_path.with_replaced_sep_and_prefix("/"),
                    to_component_id: charlie.id(),
                    to_attribute_value_id: charlie_si_name_attribute_value_id,
                    to_attribute_value_path: charlie_si_name_attribute_value_path,
                    to_prop_id: charlie_si_name_prop_id,
                    to_prop_path: charlie_si_name_prop_path.with_replaced_sep_and_prefix("/"),
                },
                Connection::Prop {
                    from_component_id: beta.id(),
                    from_attribute_value_id: beta_domain_name_attribute_value_id,
                    from_attribute_value_path: beta_domain_name_attribute_value_path,
                    from_prop_id: beta_domain_name_prop_id,
                    from_prop_path: beta_domain_name_prop_path.with_replaced_sep_and_prefix("/"),
                    to_component_id: charlie.id(),
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
async fn add_prop_connection(
    ctx: &DalContext,
    &PropConnection {
        from: (from_component_id, ref from_path),
        to: (to_component_id, ref to_path),
        func_id,
    }: &PropConnection,
) -> Result<bool> {
    // If the destination already has an explicit value, we keep it instead of replacing it!
    let to_root_av_id = Component::root_attribute_value_id(ctx, to_component_id).await?;
    let to_path = AttributePath::from_json_pointer(to_path.to_string());
    let to_av_id = to_path.vivify(ctx, to_root_av_id).await?;
    if AttributeValue::component_prototype_id(ctx, to_av_id)
        .await?
        .is_some()
    {
        return Ok(false);
    }

    // Create the subscription
    let from_root_av_id = Component::root_attribute_value_id(ctx, from_component_id).await?;
    let from_path = AttributePath::from_json_pointer(from_path.to_string());
    AttributeValue::set_to_subscription(
        ctx,
        to_av_id,
        ValueSubscription {
            attribute_value_id: from_root_av_id,
            path: from_path,
        },
        Some(func_id),
        Reason::new_user_added(ctx),
    )
    .await?;

    Ok(true)
}

/// Remove the existing socket connection (unless it was inferred, in which case there isn't one)
async fn remove_socket_connection(
    ctx: &DalContext,
    explicit_connection_id: Option<AttributePrototypeArgumentId>,
) -> Result<bool> {
    // We don't remove inferred connections
    let Some(explicit_connection_id) = explicit_connection_id else {
        return Ok(false);
    };

    // Remove the connection
    AttributePrototypeArgument::remove(ctx, explicit_connection_id).await?;

    Ok(true)
}
