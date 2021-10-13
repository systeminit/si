use si_model::resolver::{
    ResolverBackendKindBooleanBinding, ResolverBackendKindJsBinding, ResolverBackendKindJsonBinding,
};
use si_model::test::{
    create_change_set, create_custom_node, create_edit_session,
    create_new_prop, create_new_prop_with_name,
    create_new_schema, one_time_setup,
    signup_new_billing_account, TestContext,
};
use si_model::{Resolver, ResolverBackendKindArrayBinding, ResolverBackendKindBinding, ResolverBackendKindNumberBinding, ResolverBackendKindObjectBinding, ResolverBackendKindStringBinding, ResolverBinding, PropKind};
use std::option::Option::None;

#[tokio::test]
async fn get_properties_for_entity_empty() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let (schema, schema_variant) = create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;
    let (prop, prop_variant) = create_new_prop(&txn, &nats, PropKind::String, &nba, &change_set, &edit_session).await;

    let node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &schema.entity_type,
    )
    .await;

    let props =
        si_model::resolver::get_properties_for_entity(&txn, &schema_variant.id, &node.object_id)
            .await
            .expect("cannot get properties for entity");
    assert_eq!(props, serde_json::json!({}));
}

// #[tokio::test]
// async fn get_properties_for_entity_nested_object_from_one_js_function() {
//     one_time_setup().await.expect("one time setup failed");
//     let ctx = TestContext::init().await;
//     let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//     let nats = nats_conn.transaction();
//     let mut conn = pg.get().await.expect("cannot connect to pg");
//     let txn = conn.transaction().await.expect("cannot create txn");
//
//     let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//     let change_set = create_change_set(&txn, &nats, &nba).await;
//     let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//
//     let (schema, schema_variant) = create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//     let prop_first_object =
//         create_new_prop_object_with_name(&txn, &nats, &schema, None, "poop".to_string()).await;
//     let prop_second_object = create_new_prop_object_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "canoe".to_string(),
//     )
//     .await;
//     let _prop_third_object = create_new_prop_string_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_second_object.id.clone()),
//         "who".to_string(),
//     )
//     .await;
//     let _prop_fourth_object = create_new_prop_boolean_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "pair".to_string(),
//     )
//     .await;
//     let prop_map_object = create_new_prop_map_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "mapperton".to_string(),
//     )
//     .await;
//     let _prop_map_object_item =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_map_object.id.clone())).await;
//     let prop_array = create_new_prop_array_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "arraymonster".to_string(),
//     )
//     .await;
//     let _prop_array_object_item =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_array.id.clone())).await;
//
//     let node = create_custom_node(
//         &pg,
//         &txn,
//         &nats_conn,
//         &nats,
//         &veritech,
//         &nba,
//         &change_set,
//         &edit_session,
//         &schema.entity_type,
//     )
//     .await;
//
//     let resulting_object = serde_json::json!(
//         {
//             "poop": {
//                 "canoe": {
//                     "who": "fletcher"
//                 },
//                 "pair": true,
//                 "mapperton": {
//                     "slow": "moving increments",
//                     "pressure": "is crushing me",
//                 },
//                 "arraymonster": [ "foo", "bar", "baz" ]
//             }
//         }
//     );
//
//     let _array_item_prop_string =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_array.id.clone())).await;
//
//     dbg!(&resulting_object);
//
//     let js_resolver = Resolver::find_by_name(&txn, "si:setJs")
//         .await
//         .expect("cannot get resolver");
//     let js_resolver_backend_binding =
//         ResolverBackendKindBinding::Js(ResolverBackendKindJsBinding {
//             code: r#"
//                 console.log("you love me");
//                 console.log("I know you do");
//                 f = {
//                     "poop": {
//                         "canoe": {
//                             "who": "fletcher"
//                         },
//                         "pair": true,
//                         "mapperton": {
//                             "slow": "moving increments",
//                             "pressure": "is crushing me",
//                         },
//                         "arraymonster": [ "foo", "bar", "baz" ]
//                     }
//                 }
//             "#
//             .to_string(),
//         });
//
//     let _schema_resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &js_resolver.id,
//         js_resolver_backend_binding.clone(),
//         schema_variant.id.clone(),
//         None,
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let properties =
//         si_model::resolver::get_properties_for_entity(&txn, &schema_variant.id, &node.object_id)
//             .await
//             .expect("cannot get properties for entity");
//     txn.commit().await.expect("nope");
//     dbg!(&properties);
//     assert_eq!(&properties, &resulting_object);
// }
//
// #[tokio::test]
// async fn get_properties_for_entity_nested_object_from_one_json_binding() {
//     one_time_setup().await.expect("one time setup failed");
//     let ctx = TestContext::init().await;
//     let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//     let nats = nats_conn.transaction();
//     let mut conn = pg.get().await.expect("cannot connect to pg");
//     let txn = conn.transaction().await.expect("cannot create txn");
//
//     let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//     let change_set = create_change_set(&txn, &nats, &nba).await;
//     let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//
//     let (schema, schema_variant) = create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//     let prop_first_object =
//         create_new_prop_object_with_name(&txn, &nats, &schema, None, "poop".to_string()).await;
//     let prop_second_object = create_new_prop_object_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "canoe".to_string(),
//     )
//     .await;
//     let _prop_third_object = create_new_prop_string_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_second_object.id.clone()),
//         "who".to_string(),
//     )
//     .await;
//     let _prop_fourth_object = create_new_prop_boolean_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "pair".to_string(),
//     )
//     .await;
//     let prop_map_object = create_new_prop_map_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "mapperton".to_string(),
//     )
//     .await;
//     let _prop_map_object_item =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_map_object.id.clone())).await;
//     let prop_array = create_new_prop_array_with_name(
//         &txn,
//         &nats,
//         &schema,
//         Some(prop_first_object.id.clone()),
//         "arraymonster".to_string(),
//     )
//     .await;
//     let _prop_array_object_item =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_array.id.clone())).await;
//
//     let node = create_custom_node(
//         &pg,
//         &txn,
//         &nats_conn,
//         &nats,
//         &veritech,
//         &nba,
//         &change_set,
//         &edit_session,
//         &schema.entity_type,
//     )
//     .await;
//
//     let resulting_object = serde_json::json!(
//         {
//             "poop": {
//                 "canoe": {
//                     "who": "fletcher"
//                 },
//                 "pair": true,
//                 "mapperton": {
//                     "slow": "moving increments",
//                     "pressure": "is crushing me",
//                 },
//                 "arraymonster": [ "foo", "bar", "baz" ]
//             }
//         }
//     );
//
//     let _array_item_prop_string =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_array.id.clone())).await;
//
//     dbg!(&resulting_object);
//
//     let json_resolver = Resolver::find_by_name(&txn, "si:setJson")
//         .await
//         .expect("cannot get resolver");
//     let json_resolver_backend_binding =
//         ResolverBackendKindBinding::Json(ResolverBackendKindJsonBinding {
//             value: resulting_object.clone(),
//         });
//
//     let _schema_resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &json_resolver.id,
//         json_resolver_backend_binding.clone(),
//         schema_variant.id.clone(),
//         None,
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let properties =
//         si_model::resolver::get_properties_for_entity(&txn, &schema_variant.id, &node.object_id)
//             .await
//             .expect("cannot get properties for entity");
//     txn.commit().await.expect("nope");
//     dbg!(&properties);
//     assert_eq!(&properties, &resulting_object);
// }
//
// #[tokio::test]
// async fn get_properties_for_entity_nested_object() {
//     one_time_setup().await.expect("one time setup failed");
//     let ctx = TestContext::init().await;
//     let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//     let nats = nats_conn.transaction();
//     let mut conn = pg.get().await.expect("cannot connect to pg");
//     let txn = conn.transaction().await.expect("cannot create txn");
//
//     let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//     let change_set = create_change_set(&txn, &nats, &nba).await;
//     let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//     let (schema, schema_variant) = create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//     let node = create_custom_node(
//         &pg,
//         &txn,
//         &nats_conn,
//         &nats,
//         &veritech,
//         &nba,
//         &change_set,
//         &edit_session,
//         &schema.entity_type,
//     )
//     .await;
//
//     let prop_first_object = create_new_prop_object(&txn, &nats, &schema, None).await;
//     let prop_first_resolver = Resolver::find_by_name(&txn, "si:setObject")
//         .await
//         .expect("cannot get resolver");
//     let prop_first_backend_binding = ResolverBackendKindBinding::EmptyObject;
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &prop_first_resolver.id,
//         prop_first_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_first_object.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_second_object =
//         create_new_prop_object(&txn, &nats, &schema, Some(prop_first_object.id.clone())).await;
//     let prop_second_resolver = Resolver::find_by_name(&txn, "si:setObject")
//         .await
//         .expect("cannot get resolver");
//     let prop_second_backend_binding = ResolverBackendKindBinding::EmptyObject;
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &prop_second_resolver.id,
//         prop_second_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_second_object.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_third_object =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_second_object.id.clone())).await;
//     let prop_third_resolver = Resolver::find_by_name(&txn, "si:setString")
//         .await
//         .expect("cannot get resolver");
//     let prop_third_backend_binding =
//         ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//             value: "fletcher".to_string(),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &prop_third_resolver.id,
//         prop_third_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_third_object.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_fourth_object =
//         create_new_prop_boolean(&txn, &nats, &schema, Some(prop_first_object.id.clone())).await;
//     let prop_fourth_resolver = Resolver::find_by_name(&txn, "si:setBoolean")
//         .await
//         .expect("cannot get resolver");
//     let prop_fourth_backend_binding =
//         ResolverBackendKindBinding::Boolean(ResolverBackendKindBooleanBinding { value: true });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &prop_fourth_resolver.id,
//         prop_fourth_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_fourth_object.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let properties =
//         si_model::resolver::get_properties_for_entity(&txn, &schema_variant.id, &node.object_id)
//             .await
//             .expect("cannot get properties for entity");
//     txn.commit().await.expect("nope");
//     dbg!(&properties);
//     assert_eq!(
//         properties[&prop_first_object.name][&prop_second_object.name][&prop_third_object.name],
//         serde_json::json!("fletcher")
//     );
//     assert_eq!(
//         properties[&prop_first_object.name][&prop_fourth_object.name],
//         serde_json::json!(true)
//     );
// }
//
// #[tokio::test]
// async fn get_properties_for_entity_with_primitive_values() {
//     one_time_setup().await.expect("one time setup failed");
//     let ctx = TestContext::init().await;
//     let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//     let nats = nats_conn.transaction();
//     let mut conn = pg.get().await.expect("cannot connect to pg");
//     let txn = conn.transaction().await.expect("cannot create txn");
//     let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//     let change_set = create_change_set(&txn, &nats, &nba).await;
//     let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//     let (schema, schema_variant) = create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//     let node = create_custom_node(
//         &pg,
//         &txn,
//         &nats_conn,
//         &nats,
//         &veritech,
//         &nba,
//         &change_set,
//         &edit_session,
//         &schema.entity_type,
//     )
//     .await;
//
//     let prop_string = create_new_prop_string(&txn, &nats, &schema, None).await;
//     let string_resolver = Resolver::find_by_name(&txn, "si:setString")
//         .await
//         .expect("cannot get resolver");
//     let backend_binding = ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//         value: String::from("spiritbox"),
//     });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &string_resolver.id,
//         backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_string.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_number = create_new_prop_number(&txn, &nats, &schema).await;
//     let number_resolver = Resolver::find_by_name(&txn, "si:setNumber")
//         .await
//         .expect("cannot get resolver");
//     let number_backend_binding =
//         ResolverBackendKindBinding::Number(ResolverBackendKindNumberBinding { value: 42 });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &number_resolver.id,
//         number_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_number.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_boolean = create_new_prop_boolean(&txn, &nats, &schema, None).await;
//     let boolean_resolver = Resolver::find_by_name(&txn, "si:setBoolean")
//         .await
//         .expect("cannot get resolver");
//     let boolean_backend_binding =
//         ResolverBackendKindBinding::Boolean(ResolverBackendKindBooleanBinding { value: true });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &boolean_resolver.id,
//         boolean_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_boolean.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_object = create_new_prop_object(&txn, &nats, &schema, None).await;
//     let object_resolver = Resolver::find_by_name(&txn, "si:setObject")
//         .await
//         .expect("cannot get resolver");
//     let object_backend_binding =
//         ResolverBackendKindBinding::Object(ResolverBackendKindObjectBinding {
//             value: serde_json::json!({}),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &object_resolver.id,
//         object_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_object.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_array = create_new_prop_array(&txn, &nats, &schema, None).await;
//     let array_resolver = Resolver::find_by_name(&txn, "si:setArray")
//         .await
//         .expect("cannot get resolver");
//     let array_backend_binding =
//         ResolverBackendKindBinding::Array(ResolverBackendKindArrayBinding {
//             value: serde_json::json!([]),
//         });
//     let array_resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &array_resolver.id,
//         array_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_array.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//     let array_item_prop_string =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_array.id.clone())).await;
//     dbg!("won't make it here");
//     let array_item_backend_binding =
//         ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//             value: String::from("system of a down"),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &string_resolver.id,
//         array_item_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(array_item_prop_string.id.clone()),
//         Some(array_resolver_binding.id.clone()),
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//     let array_item_backend_binding_two =
//         ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//             value: String::from("dream theater"),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &string_resolver.id,
//         array_item_backend_binding_two.clone(),
//         schema_variant.id.clone(),
//         Some(array_item_prop_string.id.clone()),
//         Some(array_resolver_binding.id.clone()),
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_map = create_new_prop_map(&txn, &nats, &schema, None).await;
//     let map_resolver = Resolver::find_by_name(&txn, "si:setObject")
//         .await
//         .expect("cannot get resolver");
//     let map_backend_binding = ResolverBackendKindBinding::EmptyObject;
//     let prop_map_resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &map_resolver.id,
//         map_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_map.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_map_item_value =
//         create_new_prop_string(&txn, &nats, &schema, Some(prop_map.id.clone())).await;
//     let prop_map_item_resolver = Resolver::find_by_name(&txn, "si:setString")
//         .await
//         .expect("cannot get resolver");
//     let prop_map_item_resolver_backend =
//         ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//             value: "pretenders".to_string(),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &prop_map_item_resolver.id,
//         prop_map_item_resolver_backend.clone(),
//         schema_variant.id.clone(),
//         Some(prop_map_item_value.id.clone()),
//         Some(prop_map_resolver_binding.id.clone()),
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         Some("band".to_string()),
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_map_item_resolver_backend_again =
//         ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//             value: "against me".to_string(),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &prop_map_item_resolver.id,
//         prop_map_item_resolver_backend_again.clone(),
//         schema_variant.id.clone(),
//         Some(prop_map_item_value.id.clone()),
//         Some(prop_map_resolver_binding.id.clone()),
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         Some("punkRock".to_string()),
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let properties =
//         si_model::resolver::get_properties_for_entity(&txn, &schema_variant.id, &node.object_id)
//             .await
//             .expect("cannot get properties for entity");
//     txn.commit().await.expect("nope");
//     dbg!(&properties);
//     assert_eq!(
//         properties[&prop_string.name],
//         serde_json::json!("spiritbox")
//     );
//     assert_eq!(properties[&prop_number.name], serde_json::json!(42));
//     assert_eq!(properties[&prop_boolean.name], serde_json::json!(true));
//     assert_eq!(properties[&prop_object.name], serde_json::json!({}));
//     assert_eq!(
//         properties[&prop_array.name],
//         serde_json::json!(["system of a down", "dream theater"])
//     );
//     assert_eq!(
//         properties[&prop_map.name],
//         serde_json::json!({"band": "pretenders", "punkRock": "against me"})
//     );
// }
//
// #[tokio::test]
// async fn get_properties_for_entity_complex_map() {
//     one_time_setup().await.expect("one time setup failed");
//     let ctx = TestContext::init().await;
//     let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//     let nats = nats_conn.transaction();
//     let mut conn = pg.get().await.expect("cannot connect to pg");
//     let txn = conn.transaction().await.expect("cannot create txn");
//
//     let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//     let change_set = create_change_set(&txn, &nats, &nba).await;
//     let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//
//     let (schema, schema_variant) = create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//     let prop_map_top = create_new_prop_map(&txn, &nats, &schema, None).await;
//     let nested_map = create_new_prop_map(&txn, &nats, &schema, Some(prop_map_top.id.clone())).await;
//     let _nested_map_item_value =
//         create_new_prop_string(&txn, &nats, &schema, Some(nested_map.id.clone())).await;
//     let node = create_custom_node(
//         &pg,
//         &txn,
//         &nats_conn,
//         &nats,
//         &veritech,
//         &nba,
//         &change_set,
//         &edit_session,
//         &schema.entity_type,
//     )
//     .await;
//
//     let map_resolver = Resolver::find_by_name(&txn, "si:setObject")
//         .await
//         .expect("cannot get resolver");
//     let map_resolver_backend_binding = ResolverBackendKindBinding::EmptyObject;
//
//     let prop_map_top_resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &map_resolver.id,
//         map_resolver_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_map_top.id.clone()),
//         None,
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         None,
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let acdc_nested_map_resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &map_resolver.id,
//         map_resolver_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(nested_map.id.clone()),
//         Some(prop_map_top_resolver_binding.id.clone()),
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         Some("acdc".to_string()),
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let prop_string = create_new_prop_string(&txn, &nats, &schema, None).await;
//     let string_resolver = Resolver::find_by_name(&txn, "si:setString")
//         .await
//         .expect("cannot get resolver");
//     let acdc_value_backend_binding =
//         ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//             value: String::from("money talks"),
//         });
//     let _resolver_binding = ResolverBinding::new(
//         &txn,
//         &nats,
//         &string_resolver.id,
//         acdc_value_backend_binding.clone(),
//         schema_variant.id.clone(),
//         Some(prop_string.id.clone()),
//         Some(acdc_nested_map_resolver_binding.id.clone()),
//         Some(node.object_id.clone()),
//         None,
//         None,
//         None,
//         Some("hit song".to_string()),
//     )
//     .await
//     .expect("cannot create resolver binding");
//
//     let properties =
//         si_model::resolver::get_properties_for_entity(&txn, &schema_variant.id, &node.object_id)
//             .await
//             .expect("cannot get properties for entity");
//     txn.commit().await.expect("nope");
//     dbg!(&properties);
//     assert_eq!(
//         properties[&prop_map_top.name],
//         serde_json::json!({ "acdc": { "hit song": "money talks" } }),
//     );
// }
//
// mod resolver {
//     use si_model::test::{one_time_setup, TestContext};
//     use si_model::{Resolver, ResolverBackendKind, ResolverOutputKind};
//
//     #[tokio::test]
//     async fn new() {
//         one_time_setup().await.expect("one time setup failed");
//         let ctx = TestContext::init().await;
//         let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
//         let nats = nats_conn.transaction();
//         let mut conn = pg.get().await.expect("cannot connect to pg");
//         let txn = conn.transaction().await.expect("cannot create txn");
//
//         let resolver = Resolver::new(
//             &txn,
//             &nats,
//             "poop",
//             "poop canoe",
//             ResolverBackendKind::String,
//             ResolverOutputKind::String,
//         )
//         .await
//         .expect("cannot create resolver");
//
//         let fetch = Resolver::find_by_name(&txn, "poop")
//             .await
//             .expect("cannot get resolver");
//         assert_eq!(resolver, fetch);
//     }
//
//     #[tokio::test]
//     async fn get_by_name() {
//         one_time_setup().await.expect("one time setup failed");
//         let ctx = TestContext::init().await;
//         let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
//         let nats = nats_conn.transaction();
//         let mut conn = pg.get().await.expect("cannot connect to pg");
//         let txn = conn.transaction().await.expect("cannot create txn");
//
//         let resolver = Resolver::new(
//             &txn,
//             &nats,
//             "poop",
//             "poop canoe",
//             ResolverBackendKind::String,
//             ResolverOutputKind::String,
//         )
//         .await
//         .expect("cannot create resolver");
//         assert_eq!(resolver.name, "poop");
//         assert_eq!(resolver.description, "poop canoe");
//         assert_eq!(resolver.backend, ResolverBackendKind::String);
//         assert_eq!(resolver.output_kind, ResolverOutputKind::String);
//     }
// }
//
// mod resolver_binding {
//     use si_model::test::{
//         create_change_set, create_custom_node, create_edit_session, create_new_prop_string,
//         create_new_schema, one_time_setup, signup_new_billing_account, TestContext,
//     };
//     use si_model::{
//         resolver::{ResolverBackendKindBinding, ResolverBackendKindStringBinding},
//         Resolver, ResolverBackendKind, ResolverBinding, ResolverOutputKind,
//     };
//     use std::option::Option::None;
//
//     #[tokio::test]
//     async fn new() {
//         one_time_setup().await.expect("one time setup failed");
//         let ctx = TestContext::init().await;
//         let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//         let nats = nats_conn.transaction();
//         let mut conn = pg.get().await.expect("cannot connect to pg");
//         let txn = conn.transaction().await.expect("cannot create txn");
//
//         let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//         let change_set = create_change_set(&txn, &nats, &nba).await;
//         let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//         let (schema, schema_variant) =
//             create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//         let prop = create_new_prop_string(&txn, &nats, &schema, None).await;
//         let node = create_custom_node(
//             &pg,
//             &txn,
//             &nats_conn,
//             &nats,
//             &veritech,
//             &nba,
//             &change_set,
//             &edit_session,
//             &schema.entity_type,
//         )
//         .await;
//
//         let resolver = Resolver::new(
//             &txn,
//             &nats,
//             "string",
//             "string Resolver",
//             ResolverBackendKind::String,
//             ResolverOutputKind::String,
//         )
//         .await
//         .expect("cannot create resolver");
//
//         let backend_binding =
//             ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//                 value: String::from("spiritbox"),
//             });
//
//         let resolver_binding = ResolverBinding::new(
//             &txn,
//             &nats,
//             &resolver.id,
//             backend_binding.clone(),
//             schema_variant.id.clone(),
//             Some(prop.id.clone()),
//             None,
//             Some(node.object_id.clone()),
//             None,
//             None,
//             None,
//             None,
//         )
//         .await
//         .expect("cannot create resolver binding");
//
//         assert_eq!(&resolver_binding.entity_id, &Some(node.object_id));
//         assert_eq!(&resolver_binding.resolver_id, &resolver.id);
//         assert_eq!(&resolver_binding.backend_binding, &backend_binding);
//         assert_eq!(resolver_binding.change_set_id, None);
//         assert_eq!(resolver_binding.edit_session_id, None);
//         assert_eq!(resolver_binding.system_id, None);
//     }
//
//     #[tokio::test]
//     async fn resolve() {
//         one_time_setup().await.expect("one time setup failed");
//         let ctx = TestContext::init().await;
//         let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
//         let nats = nats_conn.transaction();
//         let mut conn = pg.get().await.expect("cannot connect to pg");
//         let txn = conn.transaction().await.expect("cannot create txn");
//
//         let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
//         let change_set = create_change_set(&txn, &nats, &nba).await;
//         let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
//         let (schema, schema_variant) =
//             create_new_schema(&txn, &nats, &change_set, &edit_session).await;
//         let prop = create_new_prop_string(&txn, &nats, &schema, None).await;
//         let node = create_custom_node(
//             &pg,
//             &txn,
//             &nats_conn,
//             &nats,
//             &veritech,
//             &nba,
//             &change_set,
//             &edit_session,
//             &schema.entity_type,
//         )
//         .await;
//
//         let resolver = Resolver::new(
//             &txn,
//             &nats,
//             "string",
//             "string Resolver",
//             ResolverBackendKind::String,
//             ResolverOutputKind::String,
//         )
//         .await
//         .expect("cannot create resolver");
//
//         let backend_binding =
//             ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
//                 value: String::from("spiritbox"),
//             });
//
//         let resolver_binding = ResolverBinding::new(
//             &txn,
//             &nats,
//             &resolver.id,
//             backend_binding.clone(),
//             schema_variant.id.clone(),
//             Some(prop.id.clone()),
//             None,
//             Some(node.object_id.clone()),
//             None,
//             None,
//             None,
//             None,
//         )
//         .await
//         .expect("cannot create resolver binding");
//         let json_string = resolver_binding
//             .resolve(&txn, &nats)
//             .await
//             .expect("cannot resolve binding");
//         assert_eq!(json_string, Some(serde_json::json!["spiritbox"]));
//     }
// }
//