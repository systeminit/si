use dal::code_view::CodeLanguage;
use dal::{ChangeSet, Component, ComponentType, DalContext};
use dal_test::test;
use dal_test::test_harness::{commit_and_update_snapshot, create_component_for_schema_name};
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;

#[test]
async fn get_diff_new_component(ctx: &mut DalContext) {
    let starfield_component =
        create_component_for_schema_name(ctx, "starfield", "this is a new component").await;
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let mut diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    dbg!(&diff);

    assert_eq!(starfield_component.id(), diff.component_id);
    assert_eq!(
        Some("{\n  \"si\": {\n    \"color\": \"#ffffff\",\n    \"name\": \"this is a new component\",\n    \"type\": \"component\"\n  },\n  \"domain\": {\n    \"name\": \"this is a new component\",\n    \"possible_world_a\": {\n      \"wormhole_1\": {\n        \"wormhole_2\": {\n          \"wormhole_3\": {}\n        }\n      }\n    },\n    \"possible_world_b\": {\n      \"wormhole_1\": {\n        \"wormhole_2\": {\n          \"wormhole_3\": {\n            \"naming_and_necessity\": \"not hesperus\"\n          }\n        }\n      }\n    },\n    \"universe\": {\n      \"galaxies\": []\n    }\n  }\n}".to_string()),
        diff.current.code
    );
    assert_eq!(CodeLanguage::Json, diff.current.language);
    assert_eq!(1, diff.diffs.len());
    let first_diff = diff.diffs.pop().expect("can't find a diff for the code");
    assert_eq!(CodeLanguage::Diff, first_diff.language);
    assert_eq!(Some("+{\n+  \"si\": {\n+    \"color\": \"#ffffff\",\n+    \"name\": \"this is a new component\",\n+    \"type\": \"component\"\n+  },\n+  \"domain\": {\n+    \"name\": \"this is a new component\",\n+    \"possible_world_a\": {\n+      \"wormhole_1\": {\n+        \"wormhole_2\": {\n+          \"wormhole_3\": {}\n+        }\n+      }\n+    },\n+    \"possible_world_b\": {\n+      \"wormhole_1\": {\n+        \"wormhole_2\": {\n+          \"wormhole_3\": {\n+            \"naming_and_necessity\": \"not hesperus\"\n+          }\n+        }\n+      }\n+    },\n+    \"universe\": {\n+      \"galaxies\": []\n+    }\n+  }\n+}".to_string()),
               first_diff.code);
}

#[test]
async fn get_diff_component_no_changes_from_head(ctx: &mut DalContext) {
    let starfield_component =
        create_component_for_schema_name(ctx, "starfield", "this is a new component").await;
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    // Apply the change set and perform a blocking commit.
    let applied_change_set = ChangeSet::apply_to_base_change_set(ctx, true)
        .await
        .expect("could not apply to base change set");
    let conflicts = ctx
        .blocking_commit()
        .await
        .expect("could not perform commit");
    assert!(conflicts.is_none());

    // Ensure the diff looks as expected on head.
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(
        applied_change_set
            .base_change_set_id
            .expect("base change set not found"),
    )
    .await
    .expect("could not update visibility and snapshot to visibility");
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
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    // Apply the change set and perform a blocking commit.
    ChangeSet::apply_to_base_change_set(ctx, true)
        .await
        .expect("could not apply to base change set");
    let conflicts = ctx
        .blocking_commit()
        .await
        .expect("could not perform commit");
    assert!(conflicts.is_none());

    // Create a new change set and perform a commit without rebasing.
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");
    ctx.commit_no_rebase()
        .await
        .expect("could not perform commit");

    starfield_component
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("Unable to change comp type");
    commit_and_update_snapshot(ctx).await;

    let mut diff = Component::get_diff(ctx, starfield_component.id())
        .await
        .expect("unable to get diff");

    assert_eq!(starfield_component.id(), diff.component_id);
    assert_eq!(1, diff.diffs.len());
    let first_diff = diff.diffs.pop().expect("can't find a diff for the code");
    assert_eq!(CodeLanguage::Diff, first_diff.language);

    // We expect there to be a diff as we have changed the componentType on this changeset but HEAD is a component
    assert_eq!(
        Some(" {\n   \"si\": {\n     \"color\": \"#ffffff\",\n     \"name\": \"this is a new component\",\n-    \"type\": \"component\"\n+    \"type\": \"configurationFrameDown\"\n   },\n   \"domain\": {\n     \"name\": \"this is a new component\",\n     \"possible_world_a\": {\n       \"wormhole_1\": {\n         \"wormhole_2\": {\n           \"wormhole_3\": {}\n         }\n       }\n     },\n     \"possible_world_b\": {\n       \"wormhole_1\": {\n         \"wormhole_2\": {\n           \"wormhole_3\": {\n             \"naming_and_necessity\": \"not hesperus\"\n           }\n         }\n       }\n     },\n     \"universe\": {\n       \"galaxies\": []\n     }\n   }\n }".to_string()), // expected
        first_diff.code // actual
    );
}
