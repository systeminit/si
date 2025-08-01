use dal::{
    Component,
    DalContext,
    Func,
    action::{
        Action,
        prototype::ActionPrototype,
    },
    attribute::{
        attributes::{
            self,
            AttributeSources,
        },
        value::AttributeValueError,
    },
    func::intrinsics::IntrinsicFunc,
};
use dal_test::{
    Result,
    helpers::{
        attribute::value::{
            self,
        },
        change_set,
        component::{
            self,
        },
        create_component_for_default_schema_name_in_default_view,
        schema::variant,
    },
    prelude::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use si_events::{
    ActionKind,
    ActionState,
};

// Test that updating attributes sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn update_attributes(ctx: &DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Parent", kind: "object", children: [
                            { name: "Updated", kind: "string" },
                            { name: "New", kind: "string" },
                            { name: "Unchanged", kind: "string" },
                            { name: "Missing", kind: "string" },
                            { name: "Default", kind: "string", defaultValue: "default" },
                        ]},
                        { name: "Missing", kind: "string" },
                        { name: "Default", kind: "string", defaultValue: "default" },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Set some initial values and make sure they are set correctly without messing with other
    // values or defaults.
    let component_id = component::create(ctx, "test", "test").await?;
    attributes::update_attributes(
        ctx,
        component_id,
        serde_json::from_value(json!({
            "/domain/Parent/Updated": "old",
            "/domain/Parent/Unchanged": "old",
        }))?,
    )
    .await?;
    assert_eq!(
        json!({
            "Parent": {
                "Updated": "old",
                "Unchanged": "old",
                "Default": "default",
            },
            "Default": "default",
        }),
        component::domain(ctx, "test").await?
    );
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Updated")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/New")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Unchanged")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Missing")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Default")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Default")).await?);

    // Update values and make sure they are updated correctly without messing with other values
    // or defaults.
    attributes::update_attributes(
        ctx,
        component_id,
        serde_json::from_value(json!({
            "/domain/Parent/Updated": "new",
            "/domain/Parent/New": "new",
        }))?,
    )
    .await?;
    assert_eq!(
        json!({
            "Parent": {
                "Updated": "new",
                "New": "new",
                "Unchanged": "old",
                "Default": "default",
            },
            "Default": "default",
        }),
        component::domain(ctx, "test").await?
    );
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Updated")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/New")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Unchanged")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Missing")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Default")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Default")).await?);

    Ok(())
}

// Test that updating an attribute value via the new subscription interface correctly enqueues
// update actions
#[test]
async fn update_attributes_enqueues_update_fn(ctx: &mut DalContext) -> Result<()> {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component_jack =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "jack antonoff")
            .await?;
    let component_swift =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "taylor swift")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    Action::remove_all_for_component_id(ctx, component_jack.id()).await?;

    // ======================================================
    // Updating values in a component that has a resource should enqueue an update action
    // ======================================================

    attributes::update_attributes(
        ctx,
        component_jack.id(),
        serde_json::from_value(json!({
            "/domain/name": "whomever",
        }))?,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    let mut update_action_count = 0;

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;

        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
            let component_id = Action::component_id(ctx, action_id)
                .await?
                .expect("is some");
            if prototype.kind == ActionKind::Update.into() && component_id == component_jack.id() {
                update_action_count += 1;
            };
        }
    }
    assert_eq!(1, update_action_count);

    // ======================================================
    // Updating values in a component that has a resource should not enqueue an update
    // action if the value didn't change
    // ======================================================

    attributes::update_attributes(
        ctx,
        component_swift.id(),
        serde_json::from_value(json!({
            "/domain/name": "taylor swift",
        }))?,
    )
    .await?;
    change_set::commit(ctx).await?;
    Action::remove_all_for_component_id(ctx, component_swift.id()).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    let mut update_action_count = 0;

    for action_id in &action_ids {
        let action_id = *action_id;
        let action = Action::get_by_id(ctx, action_id).await?;
        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
            let component_id = Action::component_id(ctx, action_id)
                .await?
                .expect("is some");
            if prototype.kind == ActionKind::Update.into() && component_id == component_swift.id() {
                update_action_count += 1;
            };
        }
    }
    // didn't actually change the value, so there should not be an update function for swifty!
    assert_eq!(0, update_action_count);

    Ok(())
}

// Test that attribute updates are processed in the order they are specified sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn update_attributes_runs_in_order_and_allows_duplicates(ctx: &mut DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Arr", kind: "array", entry:
                            { name: "ArrayItem", kind: "string" },
                        },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Subscribe source.Obj -> dest.Obj, source.Arr -> dest.Arr
    let test = component::create(ctx, "test", "test").await?;
    attributes::update_attributes(
        ctx,
        test,
        serde_json::from_str(
            r#"{
                "/domain/Arr/-": "0",
                "/domain/Arr/-": "1",
                "/domain/Arr/2": "oops",
                "/domain/Arr/3": "3",
                "/domain/Arr/4": "4",
                "/domain/Arr/-": "5",
                "/domain/Arr/-": "6",
                "/domain/Arr/-": "7",
                "/domain/Arr/-": "8",
                "/domain/Arr/-": "9",
                "/domain/Arr/10": "oops",
                "/domain/Arr/10": "10",
                "/domain/Arr/11": "11",
                "/domain/Arr/2": "2"
            }"#,
        )?,
    )
    .await?;

    assert_eq!(
        json!({ "Arr": ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"] }),
        component::domain(ctx, "test").await?,
    );

    Ok(())
}

// Test that attribute updates are processed in the order they are specified sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn component_sources_in_order(ctx: &mut DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Foo", kind: "string" },
                        { name: "Bar", kind: "string" },
                        { name: "Arr", kind: "array", entry:
                            { name: "ArrItem", kind: "string" },
                        },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Subscribe source.Obj -> dest.Obj, source.Arr -> dest.Arr
    let test = component::create(ctx, "test", "test").await?;

    // If none of the values are set, sources should be empty
    assert_eq!(
        json!({
            "/si/name": "test",
            "/si/type": "component",
        }),
        serde_json::to_value(AttributeSources::from(Component::sources(ctx, test).await?))?
    );

    // If some of the values are set, sources should contain them
    value::set(ctx, ("test", "/domain/Foo"), "foo").await?;
    assert_eq!(
        json!({
            "/si/name": "test",
            "/si/type": "component",
            "/domain/Foo": "foo"
        }),
        serde_json::to_value(AttributeSources::from(Component::sources(ctx, test).await?))?
    );

    // If all of the values are set, sources should contain them
    value::set(ctx, ("test", "/domain/Bar"), "bar").await?;
    value::set(ctx, ("test", "/domain/Arr"), ["a", "b"]).await?;
    assert_eq!(
        json!({
            "/si/name": "test",
            "/si/type": "component",
            "/domain/Foo": "foo",
            "/domain/Bar": "bar",
            "/domain/Arr/0": "a",
            "/domain/Arr/1": "b",
        }),
        serde_json::to_value(AttributeSources::from(Component::sources(ctx, test).await?))?
    );

    // If some of the array items are subscriptions, sources should show that
    let subscriber = component::create(ctx, "test", "subscriber").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Foo"),
        [("test", "/domain/Foo")],
    )
    .await?;
    value::set(ctx, ("subscriber", "/domain/Bar"), "bar2").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Arr/-"),
        [("test", "/domain/Arr/0")],
    )
    .await?;
    value::set(ctx, ("subscriber", "/domain/Arr/-"), "a2").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Arr/-"),
        [("test", "/domain/Arr/1")],
    )
    .await?;
    value::set(ctx, ("subscriber", "/domain/Arr/-"), "b2").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Foo": "foo",
            "Bar": "bar2",
            "Arr": [
                "a",
                "a2",
                "b",
                "b2"
            ]
        }),
        component::domain(ctx, "subscriber").await?
    );
    assert_eq!(
        json!({
            "/si/name": "subscriber",
            "/si/type": "component",
            "/domain/Foo": { "$source": { "component": test.to_string(), "path": "/domain/Foo" } },
            "/domain/Bar": "bar2",
            "/domain/Arr/0": { "$source": { "component": test.to_string(), "path": "/domain/Arr/0" } },
            "/domain/Arr/1": "a2",
            "/domain/Arr/2": { "$source": { "component": test.to_string(), "path": "/domain/Arr/1" } },
            "/domain/Arr/3": "b2"
        }),
        serde_json::to_value(AttributeSources::from(
            Component::sources(ctx, subscriber).await?
        ))?
    );

    // If the entire array is a subscription, child values should not be included even if DVU has run
    let subscriber2 = component::create(ctx, "test", "subscriber2").await?;
    value::subscribe(
        ctx,
        ("subscriber2", "/domain/Arr"),
        [("test", "/domain/Arr")],
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Arr": [
                "a",
                "b"
            ]
        }),
        component::domain(ctx, "subscriber2").await?
    );
    let normalize_to_array = Func::find_intrinsic(ctx, IntrinsicFunc::NormalizeToArray).await?;
    assert_eq!(
        json!({
            "/si/name": "subscriber2",
            "/si/type": "component",
            "/domain/Arr": { "$source": {
                "component": test.to_string(),
                "path": "/domain/Arr",
                "func": normalize_to_array.to_string()
            } },
        }),
        serde_json::to_value(AttributeSources::from(
            Component::sources(ctx, subscriber2).await?
        ))?
    );
    Ok(())
}

// Test that updating attributes sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn update_attribute_child_of_subscription(ctx: &mut DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Obj", kind: "object", children: [
                            { name: "Field", kind: "string" },
                        ]},
                        { name: "Map", kind: "map", entry:
                            { name: "MapItem", kind: "string" },
                        },
                        { name: "Arr", kind: "array", entry:
                            { name: "ArrayItem", kind: "string" },
                        },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Subscribe source.Obj -> dest.Obj, source.Arr -> dest.Arr
    component::create(ctx, "test", "source").await?;
    value::set(ctx, ("source", "/domain/Obj/Field"), "value").await?;
    value::set(ctx, ("source", "/domain/Map/a"), "valueA").await?;
    value::set(ctx, ("source", "/domain/Map/b"), "valueB").await?;
    value::set(ctx, ("source", "/domain/Arr"), ["a", "b"]).await?;
    let dest = component::create(ctx, "test", "dest").await?;
    value::subscribe(ctx, ("dest", "/domain/Obj"), [("source", "/domain/Obj")]).await?;
    value::subscribe(ctx, ("dest", "/domain/Map"), [("source", "/domain/Map")]).await?;
    value::subscribe(ctx, ("dest", "/domain/Arr"), [("source", "/domain/Arr")]).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Obj": {
                "Field": "value",
            },
            "Map": {
                "a": "valueA",
                "b": "valueB",
            },
            "Arr": ["a", "b"],
        }),
        component::domain(ctx, "dest").await?
    );

    // Check that updating a child value of an object/map/array yields an error
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Obj/Field": "new",
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Map/a": "updated",
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Arr/0": "new",
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Arr/-": "new",
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));

    // Check that removing a child value of an object/map/array yields an error
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Obj/Field": { "$source": null },
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Map/a": { "$source": null },
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));
    assert!(matches!(
        attributes::update_attributes(
            ctx,
            dest,
            serde_json::from_value(json!({
                "/domain/Arr/0": { "$source": null },
            }))?,
        )
        .await,
        Err(attributes::Error::AttributeValue(
            AttributeValueError::CannotSetChildOfDynamicValue(..)
        )),
    ));

    Ok(())
}

// Test that updating attributes sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn update_attribute_child_of_connection(ctx: &mut DalContext) -> Result<()> {
    variant::create(
        ctx,
        "source",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Obj", kind: "object", children: [
                            { name: "Field", kind: "string" },
                        ]},
                        { name: "Map", kind: "map", entry:
                            { name: "MapItem", kind: "string" },
                        },
                        { name: "Arr", kind: "array", entry:
                            { name: "ArrayItem", kind: "string" },
                        },
                    ],
                    outputSockets: [
                        { name: "Obj", arity: "many", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Obj" ] }, connectionAnnotations: "[\"Obj\"]" },
                        { name: "Map", arity: "many", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Map" ] }, connectionAnnotations: "[\"Map\"]" },
                        { name: "Arr", arity: "many", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Arr" ] }, connectionAnnotations: "[\"Arr\"]" },
                    ],
                };
            }
        "#,
    )
    .await?;

    variant::create(
        ctx,
        "dest",
        r#"
        function main() {
            return {
                props: [
                    { name: "Obj", kind: "object", valueFrom: { kind: "inputSocket", socket_name: "Obj" }, children: [
                        { name: "Field", kind: "string" },
                    ]},
                    { name: "Map", kind: "map", valueFrom: { kind: "inputSocket", socket_name: "Map" }, entry:
                        { name: "MapItem", kind: "string" },
                    },
                    { name: "Arr", kind: "array", valueFrom: { kind: "inputSocket", socket_name: "Arr" }, entry:
                        { name: "ArrayItem", kind: "string" },
                    },
                ],
                inputSockets: [
                    { name: "Obj", arity: "one", connectionAnnotations: "[\"Obj\"]" },
                    { name: "Map", arity: "one", connectionAnnotations: "[\"Map\"]" },
                    { name: "Arr", arity: "one", connectionAnnotations: "[\"Arr\"]" },
                ],
            };
        }
    "#,
    )
    .await?;

    // Subscribe source.Obj -> dest.Obj, source.Arr -> dest.Arr
    component::create(ctx, "source", "source").await?;
    value::set(ctx, ("source", "/domain/Obj/Field"), "value").await?;
    value::set(ctx, ("source", "/domain/Map/a"), "valueA").await?;
    value::set(ctx, ("source", "/domain/Map/b"), "valueB").await?;
    value::set(ctx, ("source", "/domain/Arr"), ["a", "b"]).await?;
    let dest = component::create(ctx, "dest", "dest").await?;
    component::connect(ctx, ("source", "Obj"), ("dest", "Obj")).await?;
    component::connect(ctx, ("source", "Map"), ("dest", "Map")).await?;
    component::connect(ctx, ("source", "Arr"), ("dest", "Arr")).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Obj": {
                "Field": "value",
            },
            "Map": {
                "a": "valueA",
                "b": "valueB",
            },
            "Arr": ["a", "b"],
        }),
        component::domain(ctx, "dest").await?
    );

    // Check that updating a child value of an object/map/array works (and overrides the connection)
    attributes::update_attributes(
        ctx,
        dest,
        serde_json::from_value(json!({
            "/domain/Obj/Field": "new",
            "/domain/Map/a": "new",
            "/domain/Arr/-": "new",
        }))?,
    )
    .await?;
    change_set::commit(ctx).await?;

    assert_eq!(
        json!({
            "Obj": {
                "Field": "new",
            },
            "Map": {
                "a": "new",
            },
            "Arr": ["new"],
        }),
        component::domain(ctx, "dest").await?
    );

    Ok(())
}
