use dal::code_view::CodeLanguage;
use dal::{Component, ComponentType, DalContext};
use dal_test::helpers::create_component_for_schema_name;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;

#[test]
async fn get_diff_new_component(ctx: &mut DalContext) {
    let starfield_component =
        create_component_for_schema_name(ctx, "starfield", "this is a new component").await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let mut diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    assert_eq!(starfield_component.id(), diff.component_id);
    assert_eq!(
        Some("{\n  \"si\": {\n    \"name\": \"this is a new component\",\n    \"type\": \"component\",\n    \"color\": \"#ffffff\"\n  },\n  \"domain\": {\n    \"name\": \"this is a new component\",\n    \"possible_world_a\": {\n      \"wormhole_1\": {\n        \"wormhole_2\": {\n          \"wormhole_3\": {}\n        }\n      }\n    },\n    \"possible_world_b\": {\n      \"wormhole_1\": {\n        \"wormhole_2\": {\n          \"wormhole_3\": {\n            \"naming_and_necessity\": \"not hesperus\"\n          }\n        }\n      }\n    },\n    \"universe\": {\n      \"galaxies\": []\n    }\n  }\n}".to_string()),
        diff.current.code
    );
    assert_eq!(CodeLanguage::Json, diff.current.language);
    assert_eq!(1, diff.diffs.len());
    let first_diff = diff.diffs.pop().expect("can't find a diff for the code");
    assert_eq!(CodeLanguage::Diff, first_diff.language);
    assert_eq!(
        Some("+{\n+  \"si\": {\n+    \"name\": \"this is a new component\",\n+    \"type\": \"component\",\n+    \"color\": \"#ffffff\"\n+  },\n+  \"domain\": {\n+    \"name\": \"this is a new component\",\n+    \"possible_world_a\": {\n+      \"wormhole_1\": {\n+        \"wormhole_2\": {\n+          \"wormhole_3\": {}\n+        }\n+      }\n+    },\n+    \"possible_world_b\": {\n+      \"wormhole_1\": {\n+        \"wormhole_2\": {\n+          \"wormhole_3\": {\n+            \"naming_and_necessity\": \"not hesperus\"\n+          }\n+        }\n+      }\n+    },\n+    \"universe\": {\n+      \"galaxies\": []\n+    }\n+  }\n+}".to_string()),
        first_diff.code
    );
}

#[test]
async fn get_diff_component_no_changes_from_head(ctx: &mut DalContext) {
    let starfield_component =
        create_component_for_schema_name(ctx, "starfield", "this is a new component").await;
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
    assert!(diff.diffs.is_empty());
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
                "possible_world_a": {
                    "wormhole_1": {
                        "wormhole_2": {
                            "wormhole_3": {}
                        }
                    }
                },
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

#[test]
async fn get_diff_component_change_comp_type(ctx: &mut DalContext) {
    let starfield_component =
        create_component_for_schema_name(ctx, "starfield", "this is a new component").await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Apply the change set and perform a blocking commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Create a new change set and perform a commit without rebasing.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork change set");

    starfield_component
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("Unable to change comp type");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let mut diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    assert_eq!(starfield_component.id(), diff.component_id);
    assert_eq!(1, diff.diffs.len());
    let first_diff = diff.diffs.pop().expect("can't find a diff for the code");
    assert_eq!(CodeLanguage::Diff, first_diff.language);

    dbg!(&first_diff.code);

    // We expect there to be a diff as we have changed the componentType on this changeset but HEAD is a component
    assert_eq!(
        Some(" {\n   \"si\": {\n     \"name\": \"this is a new component\",\n-    \"type\": \"component\",\n+    \"type\": \"configurationFrameDown\",\n     \"color\": \"#ffffff\"\n   },\n   \"domain\": {\n     \"name\": \"this is a new component\",\n     \"possible_world_a\": {\n       \"wormhole_1\": {\n         \"wormhole_2\": {\n           \"wormhole_3\": {}\n         }\n       }\n     },\n     \"possible_world_b\": {\n       \"wormhole_1\": {\n         \"wormhole_2\": {\n           \"wormhole_3\": {\n             \"naming_and_necessity\": \"not hesperus\"\n           }\n         }\n       }\n     },\n     \"universe\": {\n       \"galaxies\": []\n     }\n   }\n }".to_string()),
        first_diff.code // actual
    );
}
