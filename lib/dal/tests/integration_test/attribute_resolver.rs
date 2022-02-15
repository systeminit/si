use crate::test_setup;

use dal::{
    attribute_resolver::{AttributeResolverContext, UNSET_ID_VALUE},
    func::{
        backend::{array::FuncBackendArrayArgs, string::FuncBackendStringArgs},
        binding::FuncBinding,
    },
    test_harness::{
        billing_account_signup, create_component_for_schema, create_component_for_schema_variant,
        create_prop_of_kind_with_name, create_schema, create_schema_variant,
    },
    AttributeResolver, Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, PropKind,
    Schema, SchemaKind, StandardModel, SystemId, Tenancy, Visibility,
};
use pretty_assertions_sorted::assert_eq;

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech);
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
        .execute(&txn, &nats, veritech)
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
    )
    .await
    .expect("cannot create new attribute resolver");
}

// TODO: Flesh out the full precedence for attributes, and refactor this test so that it is less
// nuts.
#[tokio::test]
async fn find_value_for_prop_and_component() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let _unset_system_id: SystemId = UNSET_ID_VALUE.into();

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
        .execute(&txn, &nats, veritech.clone())
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
        .execute(&txn, &nats, veritech.clone())
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
        .execute(&txn, &nats, veritech.clone())
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
        .execute(&txn, &nats, veritech.clone())
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
        .execute(&txn, &nats, veritech)
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
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech);
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
        .execute(&txn, &nats, veritech.clone())
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
        .execute(&txn, &nats, veritech)
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
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech);
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
        .execute(&txn, &nats, veritech.clone())
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
    )
    .await
    .expect("cannot create new attribute resolver");

    let prop_string = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
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
        .execute(&txn, &nats, veritech.clone())
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
