use dal::attribute::value::DependentValueGraph;
use dal::component::{ComponentGeometry, DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::diagram::Diagram;
use dal::prop::{Prop, PropPath};
use dal::property_editor::values::PropertyEditorValues;
use dal::workspace_snapshot::graph::detect_conflicts_and_updates::DetectConflictsAndUpdates;
use dal::{AttributeValue, AttributeValueId, WorkspaceSnapshot};
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::helpers::{
    connect_components_with_socket_names, create_component_for_default_schema_name,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

struct TestSnapshot<'a> {
    pub ctx: &'a DalContext,
    pub snapshot: WorkspaceSnapshot,
}

#[test]
async fn associated_component_id(ctx: &mut DalContext) {
    // Schema Variant "Docker Image"
    // |--[]--> Prop (root)
    // |  |--[]--> Prop "si"
    // |     |--[]--> Prop "name"
    // |        |--[]--> Attribute Prototype
    // |           |--(Func) "si:unset"
    // |           |--[]--> Attribute Prototype Argument
    // |              |--(Func Arg) ""
    // |              |--[]--> Static Argument Value
    // |--[]--> Input Socket "Docker Hub Credential"
    // |  |--[]--> Attribute Prototype
    // |     |--(Func) "si:identity"
    // |     |--[]--> Attribute Prototype Argument
    // |        |--(Func Arg) "identity"
    // |        |--[]--> Static Argument Value
    // |--[]--> Output Socket "Container Image"
    //    |--[]--> Attribute Prototype
    //       |--(Func) "si:identity"
    //       |--[]--> Attribute Prototype Argument
    //          |--(Func Arg) "identity"
    //          |--[]--> Static Argument Value
    //
    // Function: "si:setString"
    // |--[]--> Func Arg "value"
    //
    // Root:
    // |--[]--> Category
    //    |--[]--> Schema "Docker Image"
    //    |--[]--> Module
    //    |--[]--> Action (empty)
    //    |--[]--> Secret (empty)
    //    |--[]--> DV Root

    let component =
    create_component_for_default_schema_name(ctx, "Docker Image", "original name")
        .await
        .expect("could not create component");

    
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let snapshot = ctx.workspace_snapshot().expect("Could not get workspace snapshot");

    //
    // Run each kind of thing through associated_component_id and make sure it yields what
    // we expect
    //

    //
    // Component:
    // |--[Root]--> AttributeValue
    //    |--[Contains]--> AttributeValue "si"
    //       |--[Contains]--> AttributeValue "name"
    //       |  |- (No edge to AttributePrototype)
    //       |--[Contains]--> AttributeValue "image"
    //          |--[]--> Attribute Prototype
    //             |--(Func) "si:setString"
    //             |--[]--> Attribute Prototype Argument
    //                |--(Func Arg) "value"
    //                |--[]--> Static Argument Value
    //
    snapshot.associated_component_id(ctx, snapshot.get_node_weight_by_index(component.id()));

    //
    // There should be conflicts in the component
    //

    let conflicts_and_updates = new_graph
        .detect_conflicts_and_updates(new_vector_clock_id, &base_graph, initial_vector_clock_id)
        .expect("Unable to detect conflicts and updates");

    assert_eq!(
        vec![Conflict::NodeContent {
            onto: NodeInformation {
                id: component_id.into(),
                index: base_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get component NodeIndex"),
                node_weight_kind: NodeWeightDiscriminants::Content,
            },
            to_rebase: NodeInformation {
                id: component_id.into(),
                index: new_graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get component NodeIndex"),
                node_weight_kind: NodeWeightDiscriminants::Content,
            },
        }],
        conflicts_and_updates.conflicts
    );
    assert!(conflicts_and_updates.updates.is_empty());
}
