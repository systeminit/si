use dal::{
    Component,
    ComponentType,
    DalContext,
    code_view::CodeLanguage,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;

#[test(enable_veritech)]
async fn get_diff_new_component(ctx: &mut DalContext) {
    let starfield_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "starfield",
        "this is a new component",
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    assert_eq!(starfield_component.id(), diff.component_id);
    let expected = Some(
        "{\n  \"si\": {\n    \"name\": \"this is a new component\",\n    \"type\": \"component\",\n    \"color\": \"#ffffff\"\n  },\n  \"domain\": {\n    \"name\": \"this is a new component\",\n    \"possible_world_b\": {\n      \"wormhole_1\": {\n        \"wormhole_2\": {\n          \"wormhole_3\": {\n            \"naming_and_necessity\": \"not hesperus\"\n          }\n        }\n      }\n    },\n    \"universe\": {\n      \"galaxies\": []\n    }\n  }\n}".to_string(),
    );
    assert_eq!(expected, diff.current.code);
    assert_eq!(CodeLanguage::Json, diff.current.language);
    let inner_diff = diff.diff.expect("can't find a diff for the code");
    assert_eq!(CodeLanguage::Diff, inner_diff.language);
    let expected = Some(
        "+{\n+  \"si\": {\n+    \"name\": \"this is a new component\",\n+    \"type\": \"component\",\n+    \"color\": \"#ffffff\"\n+  },\n+  \"domain\": {\n+    \"name\": \"this is a new component\",\n+    \"possible_world_b\": {\n+      \"wormhole_1\": {\n+        \"wormhole_2\": {\n+          \"wormhole_3\": {\n+            \"naming_and_necessity\": \"not hesperus\"\n+          }\n+        }\n+      }\n+    },\n+    \"universe\": {\n+      \"galaxies\": []\n+    }\n+  }\n+}".to_string(),
    );
    assert_eq!(expected, inner_diff.code);
}

#[test(enable_veritech)]
async fn get_diff_component_no_changes_from_head(ctx: &mut DalContext) {
    let starfield_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "starfield",
        "this is a new component",
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Apply the change set and perform a blocking commit and ensure the diff looks as expected on head.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");
    let diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    assert_eq!(starfield_component.id(), diff.component_id);
    assert!(diff.diff.is_none());
    let code = diff.current.code.expect("code not found");

    // We expect there to be no marked diffs as the component is the same on HEAD. Since the diff
    // isn't marked, it is valid json and we can deserialize it.
    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "this is a new component",
                "type": "component"
            },
            "domain": {
                "name": "this is a new component",
                "possible_world_b": {
                    "wormhole_1": {
                        "wormhole_2": {
                            "wormhole_3": {
                                "naming_and_necessity": "not hesperus"
                            }
                        }
                    }
                },
                "universe": {
                    "galaxies": []
                },
            }
        }], // expected
        serde_json::from_str::<Value>(code.as_str()).expect("could not deserialize") // actual
    );
}

#[test(enable_veritech)]
async fn get_diff_component_change_comp_type(ctx: &mut DalContext) {
    let starfield_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "starfield",
        "this is a new component",
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Apply the change set and create a new change set.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork change set");

    Component::set_type_by_id(
        ctx,
        starfield_component.id(),
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not update type");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    assert_eq!(starfield_component.id(), diff.component_id);
    let inner_diff = diff.diff.expect("can't find a diff for the code");
    assert_eq!(CodeLanguage::Diff, inner_diff.language);

    let expected = Some(
        " {\n   \"si\": {\n     \"name\": \"this is a new component\",\n-    \"type\": \"component\",\n+    \"type\": \"configurationFrameDown\",\n     \"color\": \"#ffffff\"\n   },\n   \"domain\": {\n     \"name\": \"this is a new component\",\n     \"possible_world_b\": {\n       \"wormhole_1\": {\n         \"wormhole_2\": {\n           \"wormhole_3\": {\n             \"naming_and_necessity\": \"not hesperus\"\n           }\n         }\n       }\n     },\n     \"universe\": {\n       \"galaxies\": []\n     }\n   }\n }".to_string(),
    );

    // We expect there to be a diff as we have changed the componentType on this changeset but HEAD is a component
    assert_eq!(
        expected,
        inner_diff.code // actual
    );
}
