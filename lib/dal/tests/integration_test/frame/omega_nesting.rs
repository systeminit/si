use dal::{
    AttributeValue,
    ComponentType,
    DalContext,
    component::frame::Frame,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_schema_name_with_type_on_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn down_frames_omega_nesting(ctx: &mut DalContext) {
    let level_one_schema_name = "large even lego";
    let level_two_schema_name = "medium even lego";
    let level_three_schema_name = "medium odd lego";
    let level_three_no_children_schema_name = "large odd lego";
    let level_four_schema_name = "small even lego";
    let level_five_schema_name = "medium even lego";
    let level_five_no_children_schema_name = "medium odd lego";
    let level_six_schema_name = "large odd lego";

    // Declare all component names based on schema names to keep us sane.
    let level_one_component_name = format!("LEVEL 1 ({level_one_schema_name})");
    let level_two_component_name = format!("LEVEL 2 ({level_two_schema_name})");
    let level_three_component_name = format!("LEVEL 3 ({level_three_schema_name})");
    let level_three_no_children_component_name =
        format!("LEVEL 3 NO CHILDREN ({level_three_no_children_schema_name})");
    let level_four_component_name = format!("LEVEL 4 ({level_four_schema_name})");
    let level_five_component_name = format!("LEVEL 5 ({level_five_schema_name})");
    let level_five_no_children_component_name =
        format!("LEVEL 5 NO CHILDREN ({level_five_no_children_schema_name})");
    let level_six_component_name = format!("LEVEL 6 ({level_six_schema_name})");

    // Create all components, set all types and commit.
    let level_one = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_one_schema_name,
        level_one_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_two = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_two_schema_name,
        level_two_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_three = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_three_schema_name,
        level_three_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_three_no_children = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_three_no_children_schema_name,
        level_three_no_children_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_four = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_four_schema_name,
        level_four_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_five = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_five_schema_name,
        level_five_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_five_no_children = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_five_no_children_schema_name,
        level_five_no_children_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let level_six = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        level_six_schema_name,
        level_six_component_name.as_str(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Perform all frame-related connections and commit.
    {
        Frame::upsert_parent_for_tests(ctx, level_two.id(), level_one.id())
            .await
            .expect("could not upsert parent");
        Frame::upsert_parent_for_tests(ctx, level_three.id(), level_two.id())
            .await
            .expect("could not upsert parent");
        Frame::upsert_parent_for_tests(ctx, level_three_no_children.id(), level_two.id())
            .await
            .expect("could not upsert parent");
        Frame::upsert_parent_for_tests(ctx, level_four.id(), level_three.id())
            .await
            .expect("could not upsert parent");
        Frame::upsert_parent_for_tests(ctx, level_five.id(), level_four.id())
            .await
            .expect("could not upsert parent");
        Frame::upsert_parent_for_tests(ctx, level_five_no_children.id(), level_four.id())
            .await
            .expect("could not upsert parent");
        Frame::upsert_parent_for_tests(ctx, level_six.id(), level_five.id())
            .await
            .expect("could not upsert parent");
    }

    // Update values that we need to see propagate. Commit after each update.
    {
        // Update level one's "five" field.
        let mut attribute_value_ids = level_one
            .attribute_values_for_prop(ctx, &["root", "domain", "five"])
            .await
            .expect("could not find attribute values");
        let attribute_value_id = attribute_value_ids.pop().expect("empty attribute values");
        assert!(attribute_value_ids.is_empty());
        AttributeValue::update(ctx, attribute_value_id, Some(serde_json::json!["5"]))
            .await
            .expect("could not update attribute value");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Update level two's "one" field.
        let mut attribute_value_ids = level_two
            .attribute_values_for_prop(ctx, &["root", "domain", "one"])
            .await
            .expect("could not find attribute values");
        let attribute_value_id = attribute_value_ids.pop().expect("empty attribute values");
        assert!(attribute_value_ids.is_empty());
        AttributeValue::update(ctx, attribute_value_id, Some(serde_json::json!["1"]))
            .await
            .expect("could not update attribute value");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Update level two's "three" field.
        let mut attribute_value_ids = level_two
            .attribute_values_for_prop(ctx, &["root", "domain", "three"])
            .await
            .expect("could not find attribute values");
        let attribute_value_id = attribute_value_ids.pop().expect("empty attribute values");
        assert!(attribute_value_ids.is_empty());
        AttributeValue::update(ctx, attribute_value_id, Some(serde_json::json!["3"]))
            .await
            .expect("could not update attribute value");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Update leve three's "two" field.
        let mut attribute_value_ids = level_three
            .attribute_values_for_prop(ctx, &["root", "domain", "two"])
            .await
            .expect("could not find attribute values");
        let attribute_value_id = attribute_value_ids.pop().expect("empty attribute values");
        assert!(attribute_value_ids.is_empty());
        AttributeValue::update(ctx, attribute_value_id, Some(serde_json::json!["2"]))
            .await
            .expect("could not update attribute value");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Update level four's "one" field.
        let mut attribute_value_ids = level_four
            .attribute_values_for_prop(ctx, &["root", "domain", "one"])
            .await
            .expect("could not find attribute values");
        let attribute_value_id = attribute_value_ids.pop().expect("empty attribute values");
        assert!(attribute_value_ids.is_empty());
        AttributeValue::update(ctx, attribute_value_id, Some(serde_json::json!["1"]))
            .await
            .expect("could not update attribute value");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
    }

    // Collect all views and ensure everything looks as we expect.
    {
        // First, collect all views.
        let level_one_view = level_one
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_two_view = level_two
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_three_view = level_three
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_three_no_children_view = level_three_no_children
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_four_view = level_four
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_five_view = level_five
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_five_no_children_view = level_five_no_children
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_six_view = level_six
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");

        // Second, ensure everything looks as we expect.
        assert_eq!(
            vec![
                serde_json::json![{
                    "si": {
                        "name": level_one_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_one_component_name,
                        "five": "5"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_two_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_two_component_name,
                        "one": "1",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_three_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_three_component_name,
                        "one": "1",
                        "two": "2",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_three_no_children_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_three_no_children_component_name,
                        "one": "1",
                        "three": "3",
                        "five": "5"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_four_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_four_component_name,
                        "one": "1",
                        "two": "2",
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_five_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_five_component_name,
                        "two": "2"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_five_no_children_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_five_no_children_component_name,
                        "one": "1",
                        "three": "3",
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_six_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_six_component_name,
                        "five": "5"
                    },
                    "resource_value": {}
                }],
            ], // expected
            vec![
                level_one_view,
                level_two_view,
                level_three_view,
                level_three_no_children_view,
                level_four_view,
                level_five_view,
                level_five_no_children_view,
                level_six_view
            ] // actual
        );
    }

    // Update level one's "five" field again and commit.
    {
        let mut attribute_value_ids = level_one
            .attribute_values_for_prop(ctx, &["root", "domain", "five"])
            .await
            .expect("could not find attribute values");
        let attribute_value_id = attribute_value_ids.pop().expect("empty attribute values");
        assert!(attribute_value_ids.is_empty());
        AttributeValue::update(
            ctx,
            attribute_value_id,
            Some(serde_json::json!["5-updated"]),
        )
        .await
        .expect("could not update attribute value");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
    }

    // Collect all views and ensure everything looks as we expect.
    {
        // First, collect all views.
        let level_one_view = level_one
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_two_view = level_two
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_three_view = level_three
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_three_no_children_view = level_three_no_children
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_four_view = level_four
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_five_view = level_five
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_five_no_children_view = level_five_no_children
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_six_view = level_six
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");

        // Second, ensure everything looks as we expect.
        assert_eq!(
            vec![
                serde_json::json![{
                    "si": {
                        "name": level_one_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_one_component_name,
                        "five": "5-updated"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_two_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_two_component_name,
                        "one": "1",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_three_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_three_component_name,
                        "one": "1",
                        "two": "2",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_three_no_children_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_three_no_children_component_name,
                        "one": "1",
                        "three": "3",
                        "five": "5-updated"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_four_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_four_component_name,
                        "one": "1",
                        "two": "2",
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_five_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_five_component_name,
                        "two": "2"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_five_no_children_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_five_no_children_component_name,
                        "one": "1",
                        "three": "3",
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_six_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_six_component_name,
                        "five": "5-updated"
                    },
                    "resource_value": {}
                }],
            ], // expected
            vec![
                level_one_view,
                level_two_view,
                level_three_view,
                level_three_no_children_view,
                level_four_view,
                level_five_view,
                level_five_no_children_view,
                level_six_view
            ] // actual
        );
    }

    // Orphan level two from level one. All children (descending) of level two should still be
    // attached (descending).
    {
        Frame::orphan_child(ctx, level_two.id())
            .await
            .expect("could not orphan");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
    }

    // Collect all views and ensure everything looks as we expect.
    {
        // First, collect all views.
        let level_one_view = level_one
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_two_view = level_two
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_three_view = level_three
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_three_no_children_view = level_three_no_children
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_four_view = level_four
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_five_view = level_five
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_five_no_children_view = level_five_no_children
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");
        let level_six_view = level_six
            .view(ctx)
            .await
            .expect("could not get view")
            .expect("empty view");

        // Second, ensure everything looks as we expect.
        assert_eq!(
            vec![
                serde_json::json![{
                    "si": {
                        "name": level_one_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_one_component_name,
                        "five": "5-updated"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_two_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_two_component_name,
                        "one": "1",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_three_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_three_component_name,
                        "one": "1",
                        "two": "2",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_three_no_children_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_three_no_children_component_name,
                        "one": "1",
                        "three": "3"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_four_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_four_component_name,
                        "one": "1",
                        "two": "2",
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_five_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_five_component_name,
                        "two": "2"
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_five_no_children_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_five_no_children_component_name,
                        "one": "1",
                        "three": "3",
                    },
                    "resource_value": {}
                }],
                serde_json::json![{
                    "si": {
                        "name": level_six_component_name,
                        "type": "configurationFrameDown",
                        "color": "#ffffff"
                    },
                    "domain": {
                        "name": level_six_component_name
                    },
                    "resource_value": {}
                }],
            ], // expected
            vec![
                level_one_view,
                level_two_view,
                level_three_view,
                level_three_no_children_view,
                level_four_view,
                level_five_view,
                level_five_no_children_view,
                level_six_view
            ] // actual
        );
    }
}
