use crate::test_setup;

use dal::{
    deculture::{
        attribute_prototype::AttributePrototype,
        attribute_resolver_context::{AttributeResolverContext, AttributeResolverContextBuilder},
    },
    func::{backend::string::FuncBackendStringArgs, binding::FuncBinding},
    test_harness::{
        create_component_for_schema, create_prop_of_kind_with_name, create_schema,
        create_schema_variant,
    },
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, PropKind, Schema, SchemaKind,
    StandardModel, Tenancy, Visibility,
};
use pretty_assertions_sorted::assert_eq;

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
        veritech.clone(),
        &encr_key,
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

    let args = FuncBackendStringArgs::new("eldenring".to_string());
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

    let context = AttributeResolverContext::builder()
        .set_prop_id(*first_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*default_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot create context");
    let _attribute_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        context,
        None,
        None,
    )
    .await
    .expect("cannot create new attribute prototype");
}

#[tokio::test]
async fn list_for_context() {
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

    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;

    let schema_variant = create_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encr_key,
    )
    .await;
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

    let mut base_prototype_context = AttributeResolverContext::builder();
    base_prototype_context
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // {
    //   albums: [
    //     { name: String, artist: String, },
    //   ]
    // }
    let albums_prop = create_prop_of_kind_with_name(
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
    albums_prop
        .add_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot set schema variant for album object");

    let albums_prototype_context = AttributeResolverContext::builder()
        .set_prop_id(*albums_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .to_context()
        .expect("cannot create attribute context");

    let _albums_prop_prototype =
        AttributePrototype::list_for_context(&txn, &tenancy, &visibility, albums_prototype_context)
            .await
            .expect("cannot retrieve attribute prototype for album")
            .pop()
            .expect("no attribute prototype found for albums");

    let album_prop = create_prop_of_kind_with_name(
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
    album_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *albums_prop.id())
        .await
        .expect("cannot set parent prop for album object");

    let album_prototype_context = base_prototype_context
        .clone()
        .set_prop_id(*album_prop.id())
        .to_context()
        .expect("cannot create attribute context");

    let _album_prop_prototype =
        AttributePrototype::list_for_context(&txn, &tenancy, &visibility, album_prototype_context)
            .await
            .expect("cannot retrieve attribute prototype for album")
            .pop()
            .expect("no attribute prototype found for album");

    let name_prop = create_prop_of_kind_with_name(
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
    name_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *album_prop.id())
        .await
        .expect("cannot set parent prop for album name");

    let album_name_prototype_context = base_prototype_context
        .clone()
        .set_prop_id(*name_prop.id())
        .to_context()
        .expect("cannot create attribute context");

    let album_name_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        album_name_prototype_context,
    )
    .await
    .expect("cannot retrieve attribute prototype for album name")
    .pop()
    .expect("no attribute prototype found for album name");

    let artist_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "artist_name",
    )
    .await;
    artist_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *album_prop.id())
        .await
        .expect("cannot set parent prop for album artist");

    let album_artist_prototype_context = base_prototype_context
        .clone()
        .set_prop_id(*artist_prop.id())
        .to_context()
        .expect("cannot create attribute context");

    let _album_artist_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        album_artist_prototype_context,
    )
    .await
    .expect("cannot retrieve attribute prototype for album artist")
    .pop()
    .expect("no attribute prototype found for album artist");

    let component = create_component_for_schema(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
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
        "si:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("Undertow".to_string());
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
    .expect("cannot create func binding");
    func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");

    let component_name_prototype_context =
        AttributeResolverContextBuilder::from(album_name_prototype_context)
            .set_component_id(*component.id())
            .to_context()
            .expect("cannot create attribute context");

    let component_album_name_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        component_name_prototype_context,
        None,
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let found_album_name_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        album_name_prototype_context,
    )
    .await
    .expect("could not retrieve album name prototype")
    .pop()
    .expect("no album name prototype found");

    assert_eq!(album_name_prototype, found_album_name_prototype,);

    let found_component_album_name_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        component_name_prototype_context,
    )
    .await
    .expect("could not retrieve album name prototype")
    .pop()
    .expect("no album name prototype found");

    assert_eq!(
        component_album_name_prototype,
        found_component_album_name_prototype,
    );
}

#[tokio::test]
async fn list_for_context_with_a_hash() {
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

    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;

    let schema_variant = create_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encr_key,
    )
    .await;
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

    let mut base_prototype_context = AttributeResolverContext::builder();
    base_prototype_context
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // {
    //   albums: [
    //     { String: String, },
    //   ]
    // }
    let albums_prop = create_prop_of_kind_with_name(
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
    albums_prop
        .add_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot set schema variant for album object");

    let albums_prototype_context = base_prototype_context
        .clone()
        .set_prop_id(*albums_prop.id())
        .to_context()
        .expect("cannot build attribute context");

    let _albums_prop_prototype =
        AttributePrototype::list_for_context(&txn, &tenancy, &visibility, albums_prototype_context)
            .await
            .expect("cannot retrieve attribute prototype for album")
            .pop()
            .expect("no attribute prototype found for albums");

    let album_prop = create_prop_of_kind_with_name(
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
    album_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *albums_prop.id())
        .await
        .expect("cannot set parent prop for album object");

    let album_prototype_context = base_prototype_context
        .clone()
        .set_prop_id(*album_prop.id())
        .to_context()
        .expect("cannot build attribute context");

    let _album_prop_prototype =
        AttributePrototype::list_for_context(&txn, &tenancy, &visibility, album_prototype_context)
            .await
            .expect("cannot retrieve attribute prototype for album")
            .pop()
            .expect("no attribute prototype found for album");

    let hash_key_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "album_hash_key",
    )
    .await;
    hash_key_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *album_prop.id())
        .await
        .expect("cannot set parent prop for album hash key");

    let prop_hash_key_prototype_context = base_prototype_context
        .clone()
        .set_prop_id(*hash_key_prop.id())
        .to_context()
        .expect("cannot build attribute context");

    let prop_hash_key_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        prop_hash_key_prototype_context,
    )
    .await
    .expect("cannot retrieve attribute prototype for album hash key")
    .pop()
    .expect("no attribute prototype found for album hash key");

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "si:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let undertow_prop_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("1993".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    undertow_prop_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let undertow_prop_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *undertow_prop_func_binding.id(),
        prop_hash_key_prototype_context,
        Some("Undertow".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let lateralus_prop_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("2001".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    lateralus_prop_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let lateralus_prop_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *lateralus_prop_func_binding.id(),
        prop_hash_key_prototype_context,
        Some("Lateralus".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let component = create_component_for_schema(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let component_hash_key_prototype_context =
        AttributeResolverContextBuilder::from(prop_hash_key_prototype_context)
            .set_component_id(*component.id())
            .to_context()
            .expect("cannot create attribute context");

    let lateralus_component_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("The Early 2000s".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    lateralus_component_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let lateralus_component_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *lateralus_component_func_binding.id(),
        component_hash_key_prototype_context,
        Some("Lateralus".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let fear_inoculum_component_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("2019".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    fear_inoculum_component_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let fear_inoculum_component_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *fear_inoculum_component_func_binding.id(),
        component_hash_key_prototype_context,
        Some("Fear Inoculum".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let found_hash_key_prototypes = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        component_hash_key_prototype_context,
    )
    .await
    .expect("could not retrieve component prototypes");

    assert_eq!(
        vec![
            fear_inoculum_component_prototype,
            lateralus_component_prototype,
            undertow_prop_prototype.clone(),
            prop_hash_key_prototype.clone(),
        ],
        found_hash_key_prototypes,
    );

    let found_hash_key_prototypes = AttributePrototype::list_for_context(
        &txn,
        &tenancy,
        &visibility,
        prop_hash_key_prototype_context,
    )
    .await
    .expect("could not retrieve prop prototypes");

    assert_eq!(
        vec![
            lateralus_prop_prototype,
            undertow_prop_prototype,
            prop_hash_key_prototype,
        ],
        found_hash_key_prototypes,
    );
}
