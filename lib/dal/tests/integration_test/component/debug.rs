use dal::{
    AttributeValue,
    Component,
    DalContext,
    Prop,
    component::debug::ComponentDebugView,
    prop::PropPath,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn get_debug_view(ctx: &mut DalContext) {
    //create a new component for starfield schema
    let component: Component =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", "new component")
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    //get the debug view
    let component_debug_view = ComponentDebugView::new(ctx, component.id())
        .await
        .expect("couldn't get component debug");

    //make sure the debug view matches the SchemaVariant ID and component name
    let sv_id = component
        .schema_variant(ctx)
        .await
        .expect("couldn't get schema variant");

    assert_eq!(component_debug_view.schema_variant_id, sv_id.id());
    assert_eq!(component_debug_view.name, "new component");

    // get attribute values for the root prop for the component and make sure the paths match
    let maybe_root_avs = component
        .attribute_values_for_prop(ctx, &["root"])
        .await
        .expect("couldn't get root prop");
    assert_eq!(maybe_root_avs.len(), 1);

    let root_av = Component::root_attribute_value_id(ctx, component.id())
        .await
        .expect("couldn't get root av");
    let maybe_root_av = maybe_root_avs
        .first()
        .copied()
        .expect("able to get the root av");
    assert_eq!(root_av, maybe_root_av);

    let prop_path = AttributeValue::get_path_for_id(ctx, root_av)
        .await
        .expect("can't get path");
    assert_eq!(prop_path, Some("root".to_string()));

    // get a more deeply nested prop/attribute value and
    let rigid_prop_path = PropPath::new([
        "root",
        "domain",
        "possible_world_a",
        "wormhole_1",
        "wormhole_2",
        "wormhole_3",
        "rigid_designator",
    ]);
    let rigid_designator_prop_id = Prop::find_prop_id_by_path(ctx, sv_id.id(), &rigid_prop_path)
        .await
        .expect("able to find 'rigid_designator' prop");
    let rigid_designator_values =
        Component::attribute_values_for_prop_id(ctx, component.id(), rigid_designator_prop_id)
            .await
            .expect("able to get attribute value for rigid_designator prop");

    let rigid_designator_value_id = rigid_designator_values
        .first()
        .copied()
        .expect("get first value id");

    let attribute_path = AttributeValue::get_path_for_id(ctx, rigid_designator_value_id)
        .await
        .expect("can't get the path");
    println!("attribute_path: {attribute_path:?}");
    assert_eq!(
        attribute_path,
        Some(rigid_prop_path.with_replaced_sep("/").to_string())
    );
}
