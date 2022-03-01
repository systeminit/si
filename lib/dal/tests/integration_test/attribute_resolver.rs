use crate::test_setup;

use dal::{
    attribute_resolver::AttributeResolverContext,
    func::{
        backend::{array::FuncBackendArrayArgs, string::FuncBackendStringArgs},
        binding::FuncBinding,
    },
    system::UNSET_SYSTEM_ID,
    test_harness::{
        billing_account_signup, create_component_for_schema, create_component_for_schema_variant,
        create_prop_of_kind_with_name, create_schema, create_schema_variant,
        find_or_create_production_system,
    },
    AttributeResolver, ComponentView, Func, FuncBackendKind, FuncBackendResponseType, HistoryActor,
    PropKind, Schema, SchemaKind, StandardModel, Tenancy, Visibility,
};
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encr_key
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image")
    .pop()
    .expect("no docker image found");

    let default_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("gatecreeper".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");

    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_component_id(*component.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create new attribute resolver");
}

// TODO: Flesh out the full precedence for attributes, and refactor this test so that it is less
// nuts.
#[tokio::test]
async fn find_value_for_prop_and_component() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech, encr_key);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image")
    .pop()
    .expect("no docker image found");

    let default_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props")
        .first()
        .cloned()
        .expect("no prop found");

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("prop".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    let _attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        Some(&serde_json::json!["prop"]),
        func_binding_return_value.value(),
    );

    let args = FuncBackendStringArgs::new("prop_and_schema".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_schema_id(*schema.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        Some(&serde_json::json!["prop_and_schema"]),
        func_binding_return_value.value(),
    );

    let args = FuncBackendStringArgs::new("prop_and_schema_variant".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_schema_variant_id(*default_variant.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        Some(&serde_json::json!["prop_and_schema_variant"]),
        func_binding_return_value.value(),
    );

    let args = FuncBackendStringArgs::new("prop_and_schema_variant".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_schema_variant_id(*default_variant.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        Some(&serde_json::json!["prop_and_schema_variant"]),
        func_binding_return_value.value(),
    );

    let args = FuncBackendStringArgs::new("prop_and_component".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_component_id(*component.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        Some(&serde_json::json!["prop_and_component"]),
        func_binding_return_value.value(),
    );
}

#[tokio::test]
async fn upsert() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encr_key
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image")
    .pop()
    .expect("no docker image found");

    let default_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("gatecreeper".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_component_id(*component.id());
    let _first_attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context.clone(),
        None,
    )
    .await
    .expect("cannot create new attribute resolver");

    let second_args = FuncBackendStringArgs::new("bleed from within".to_string());
    let second_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(second_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    second_func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");
    let second_attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *second_func_binding.id(),
        attribute_resolver_context,
        None,
    )
    .await
    .expect("cannot create new attribute resolver");

    assert_eq!(
        *second_func_binding.id(),
        second_attribute_resolver.func_binding_id(),
    );
}

#[tokio::test]
async fn update_parent_index_map() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encr_key
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Implementation,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let prop_array_spiritbox = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "spiritbox",
    )
    .await;

    prop_array_spiritbox
        .add_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot add prop to schema variant");

    let set_array_func = Func::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"si:setArray".to_string(),
    )
    .await
    .expect("cannot find function")
    .pop()
    .expect("returned no function");

    let array_args = FuncBackendArrayArgs::new(Vec::new());
    let array_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(array_args).expect("cannot turn args into json"),
        *set_array_func.id(),
        *set_array_func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    array_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let mut array_attribute_resolver_context = AttributeResolverContext::new();
    array_attribute_resolver_context.set_prop_id(*prop_array_spiritbox.id());
    array_attribute_resolver_context.set_component_id(*component.id());
    let array_attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *set_array_func.id(),
        *array_func_binding.id(),
        array_attribute_resolver_context.clone(),
        None,
    )
    .await
    .expect("cannot create new attribute resolver");

    let prop_string = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "stringbean",
    )
    .await;
    prop_string
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *prop_array_spiritbox.id(),
        )
        .await
        .expect("cannot set parent prop");

    let set_string_func = Func::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"si:setString".to_string(),
    )
    .await
    .expect("cannot find function")
    .pop()
    .expect("returned no function");

    let string_args = FuncBackendStringArgs::new("wonderful".to_string());
    let string_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(string_args).expect("cannot turn args into json"),
        *set_string_func.id(),
        *set_string_func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    string_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let mut string_attribute_resolver_context = AttributeResolverContext::new();
    string_attribute_resolver_context.set_prop_id(*prop_string.id());
    string_attribute_resolver_context.set_component_id(*component.id());
    let string_attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *set_string_func.id(),
        *string_func_binding.id(),
        string_attribute_resolver_context.clone(),
        None,
    )
    .await
    .expect("cannot create new attribute resolver");

    let fetched_array_attribute_resolver =
        AttributeResolver::get_by_id(&txn, &tenancy, &visibility, array_attribute_resolver.id())
            .await
            .expect("cannot get attribute resolver")
            .expect("attribute resolve was not found");
    let index_map = fetched_array_attribute_resolver
        .index_map()
        .expect("there must be an index map now");
    assert_eq!(&[*string_attribute_resolver.id()], index_map.order());
}

#[tokio::test]
async fn siblings_have_values() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let object_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "top-level_object",
    )
    .await;
    object_prop
        .add_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot associate prop with schema variant");

    let child_prop_one = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "some child prop",
    )
    .await;
    child_prop_one
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *object_prop.id())
        .await
        .expect("cannot set parent prop for child one");

    let child_prop_two = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "another child prop",
    )
    .await;
    child_prop_two
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *object_prop.id())
        .await
        .expect("cannot set parent prop for child two");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let (_, object_attribute_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &object_prop,
            Some(serde_json::json![{}]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve object prop");

    let (_, child_one_attribute_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &child_prop_one,
            None,
            Some(object_attribute_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve child prop one");

    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &child_prop_two,
            None,
            Some(object_attribute_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve child prop two");

    assert!(
        !AttributeResolver::any_siblings_are_set(
            &txn,
            &tenancy,
            &visibility,
            child_one_attribute_resolver_id
        )
        .await
        .expect("cannot check siblings for values"),
        "no siblings should have a value"
    );

    let (_, child_two_attribute_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &child_prop_two,
            Some(serde_json::json!("second child's value")),
            Some(object_attribute_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve child prop two with a value");

    assert!(
        AttributeResolver::any_siblings_are_set(
            &txn,
            &tenancy,
            &visibility,
            child_one_attribute_resolver_id
        )
        .await
        .expect("cannot check siblings for values"),
        "siblings of child one should have a value"
    );
    assert!(
        !AttributeResolver::any_siblings_are_set(
            &txn,
            &tenancy,
            &visibility,
            child_two_attribute_resolver_id
        )
        .await
        .expect("cannot check siblings for values"),
        "no siblings of child two should have a value"
    );
}

#[tokio::test]
async fn update_for_context_will_unset_parent_resolvers() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );

    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let prod_system =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    // ["
    //    { "name": "Astro-Creep: 2000", "release_year": "1995" }
    // ]
    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let albums_array_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "albums_array",
    )
    .await;
    albums_array_prop
        .add_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot associate root array with schema variant");

    let album_object_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "album_object",
    )
    .await;
    album_object_prop
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *albums_array_prop.id(),
        )
        .await
        .expect("cannot set parent prop for album object");

    let album_name_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "album_name",
    )
    .await;
    album_name_prop
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent prop for album name");

    let album_release_year_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "release_year",
    )
    .await;
    album_release_year_prop
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent prop for album release year");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let mut albums_resolver_context = AttributeResolverContext::new();
    albums_resolver_context.set_prop_id(*albums_array_prop.id());
    let albums_resolver =
        AttributeResolver::find_for_context(&txn, &tenancy, &visibility, albums_resolver_context)
            .await
            .expect("cannot retrieve resolver for albums array")
            .expect("resolver for albums array not found");
    let mut astro_creep_album_resolver_context = AttributeResolverContext::new();
    astro_creep_album_resolver_context.set_prop_id(*album_object_prop.id());
    astro_creep_album_resolver_context.set_component_id(*component.id());
    astro_creep_album_resolver_context.set_system_id(*prod_system.id());
    let (_, astro_creep_album_resolver_id) = AttributeResolver::insert_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        astro_creep_album_resolver_context,
        *albums_resolver.id(),
        Some(serde_json::json![()]),
        None,
    )
    .await
    .expect("cannot insert a new album");

    let mut astro_creep_album_name_resolver_context = AttributeResolverContext::new();
    astro_creep_album_name_resolver_context.set_prop_id(*album_name_prop.id());
    astro_creep_album_name_resolver_context.set_component_id(*component.id());
    astro_creep_album_name_resolver_context.set_system_id(*prod_system.id());
    let (_, astro_creep_album_name_resolver_id) = AttributeResolver::insert_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        astro_creep_album_name_resolver_context.clone(),
        astro_creep_album_resolver_id,
        Some(serde_json::json!["Astro-Creep: 2000"]),
        None,
    )
    .await
    .expect("cannot insert album name");

    let mut astro_creep_album_release_year_resolver_context = AttributeResolverContext::new();
    astro_creep_album_release_year_resolver_context.set_prop_id(*album_release_year_prop.id());
    astro_creep_album_release_year_resolver_context.set_component_id(*component.id());
    astro_creep_album_release_year_resolver_context.set_system_id(*prod_system.id());
    let (_, astro_creep_album_release_year_resolver_id) = AttributeResolver::insert_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        astro_creep_album_release_year_resolver_context.clone(),
        astro_creep_album_resolver_id,
        Some(serde_json::json!["1995"]),
        None,
    )
    .await
    .expect("cannot insert album release year");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "albums_array": [
                    { "album_name": "Astro-Creep: 2000", "release_year": "1995" }
                ]
            }
        ],
        component_view.properties,
    );

    AttributeResolver::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        astro_creep_album_release_year_resolver_id,
        astro_creep_album_release_year_resolver_context,
        None,
        None,
    )
    .await
    .expect("could not unset album release year");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "albums_array": [
                    { "album_name": "Astro-Creep: 2000" }
                ]
            }
        ],
        component_view.properties,
    );

    AttributeResolver::update_for_context(
        &txn,
        &nats,
        veritech,
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        astro_creep_album_name_resolver_id,
        astro_creep_album_name_resolver_context,
        None,
        None,
    )
    .await
    .expect("could not unset album name");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(serde_json::json![{}], component_view.properties,);
}

/// Set default values for two prop strings in a prop object. We use the prototype attribute
/// resolver for the object and its two string fields. Override those values to different
/// values in a component. Finally, remove each value individually and test that the fallback to
/// the default values works properly.
#[tokio::test]
async fn remove_for_context() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );

    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let prod_system =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    // { "name": "Lateralus", "release_year": "2001" }
    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let album_object_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "album_object",
    )
    .await;
    album_object_prop
        .add_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot set schema variant for album object");

    let album_name_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "album_name",
    )
    .await;
    album_name_prop
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent prop for album name");

    let album_release_year_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "release_year",
    )
    .await;
    album_release_year_prop
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent prop for album release year");

    let mut album_resolver_context = AttributeResolverContext::new();
    album_resolver_context.set_prop_id(*album_object_prop.id());
    let album_resolver = AttributeResolver::find_for_context(
        &txn,
        &tenancy,
        &visibility,
        album_resolver_context.clone(),
    )
    .await
    .expect("cannot retrieve resolver for album object")
    .expect("resolver for album object not found");

    let mut album_name_resolver_context = AttributeResolverContext::new();
    album_name_resolver_context.set_prop_id(*album_name_prop.id());
    let album_name_resolver = AttributeResolver::find_for_context(
        &txn,
        &tenancy,
        &visibility,
        album_name_resolver_context.clone(),
    )
    .await
    .expect("cannot retrieve resolver for album name")
    .expect("resolver for album name not found");
    album_name_resolver
        .set_parent_attribute_resolver(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *album_resolver.id(),
        )
        .await
        .expect("could not set parent attribute resolver for album name resolver");

    let mut album_release_year_resolver_context = AttributeResolverContext::new();
    album_release_year_resolver_context.set_prop_id(*album_release_year_prop.id());
    let album_release_year_resolver = AttributeResolver::find_for_context(
        &txn,
        &tenancy,
        &visibility,
        album_release_year_resolver_context.clone(),
    )
    .await
    .expect("cannot retrieve resolver for album release year")
    .expect("resolver for album release year not found");
    album_release_year_resolver
        .set_parent_attribute_resolver(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            *album_resolver.id(),
        )
        .await
        .expect("could not set parent attribute resolver for album release year resolver");

    let (_, album_name_resolver_id) = AttributeResolver::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        *album_name_resolver.id(),
        album_name_resolver_context.clone(),
        Some(serde_json::json!["Lateralus"]),
        None,
    )
    .await
    .expect("cannot insert album name");

    let (_, album_release_year_resolver_id) = AttributeResolver::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        *album_release_year_resolver.id(),
        album_release_year_resolver_context.clone(),
        Some(serde_json::json!["2001"]),
        None,
    )
    .await
    .expect("cannot insert album release year");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            { "album_object":
                    { "album_name": "Lateralus", "release_year": "2001" } }
        ],
        component_view.properties,
    );

    album_name_resolver_context.set_component_id(*component.id());
    album_name_resolver_context.set_system_id(*prod_system.id());
    let (_, updated_album_name_resolver_id) = AttributeResolver::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        album_name_resolver_id,
        album_name_resolver_context.clone(),
        Some(serde_json::json!["Astro-Creep: 2000"]),
        None,
    )
    .await
    .expect("cannot insert album name");

    album_release_year_resolver_context.set_component_id(*component.id());
    album_release_year_resolver_context.set_system_id(*prod_system.id());
    let (_, updated_album_release_year_resolver_id) = AttributeResolver::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        album_release_year_resolver_id,
        album_release_year_resolver_context.clone(),
        Some(serde_json::json!["1995"]),
        None,
    )
    .await
    .expect("cannot insert album release year");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            { "album_object":
                    { "album_name": "Astro-Creep: 2000", "release_year": "1995" } }
        ],
        component_view.properties,
    );

    AttributeResolver::remove_for_context(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        updated_album_name_resolver_id,
        album_name_resolver_context,
    )
    .await
    .expect("could not remove album name");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            { "album_object":
                    { "album_name": "Lateralus", "release_year": "1995" } }
        ],
        component_view.properties,
    );

    AttributeResolver::remove_for_context(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        updated_album_release_year_resolver_id,
        album_release_year_resolver_context,
    )
    .await
    .expect("could not remove album release year");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *prod_system.id(),
    )
    .await
    .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            { "album_object":
                    { "album_name": "Lateralus", "release_year": "2001" } }
        ],
        component_view.properties,
    );
}
