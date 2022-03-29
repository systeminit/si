use crate::test_setup;

use crate::dal::test;
use dal::{
    attribute::context::{AttributeContext, AttributeContextBuilder},
    attribute::prototype::AttributePrototype,
    func::{backend::string::FuncBackendStringArgs, binding::FuncBinding},
    test_harness::{
        create_component_for_schema, create_prop_of_kind_with_name, create_schema,
        create_schema_variant, create_schema_variant_with_root,
    },
    AttributePrototypeError, AttributeReadContext, AttributeValue, Component, ComponentView, Func,
    FuncBackendKind, FuncBackendResponseType, HistoryActor, PropKind, ReadTenancy, Schema,
    SchemaKind, StandardModel, Tenancy, Visibility,
};
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};

#[test]
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
        .default_variant(
            &txn,
            &tenancy
                .clone_into_read_tenancy(&txn)
                .await
                .expect("unable to generate read tenancy"),
            &visibility,
        )
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
        &(&tenancy).into(),
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
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    let func_binding_return_value = func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");

    let context = AttributeContext::builder()
        .set_prop_id(*first_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*default_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot create context");
    let _attribute_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        *func_binding_return_value.id(),
        context,
        None,
        None,
    )
    .await
    .expect("cannot create new attribute prototype");
}

#[test]
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
        *schema.id(),
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

    let mut base_prototype_context = AttributeContext::builder();
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

    let albums_prototype_context = AttributeContext::builder()
        .set_prop_id(*albums_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .to_context()
        .expect("cannot create attribute context");

    let _albums_prop_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        albums_prototype_context,
    )
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

    let _album_prop_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        album_prototype_context,
    )
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
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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
        &(&tenancy).into(),
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
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    let func_binding_return_value = func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");

    let component_name_prototype_context =
        AttributeContextBuilder::from(album_name_prototype_context)
            .set_component_id(*component.id())
            .to_context()
            .expect("cannot create attribute context");

    let component_album_name_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        *func_binding_return_value.id(),
        component_name_prototype_context,
        None,
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let found_album_name_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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

#[test]
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
        *schema.id(),
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

    let mut base_prototype_context = AttributeContext::builder();
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

    let _albums_prop_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        albums_prototype_context,
    )
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

    let _album_prop_prototype = AttributePrototype::list_for_context(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        album_prototype_context,
    )
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
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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
        &(&tenancy).into(),
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
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("1993".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    let func_binding_return_value = undertow_prop_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let undertow_prop_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *func.id(),
        *undertow_prop_func_binding.id(),
        *func_binding_return_value.id(),
        prop_hash_key_prototype_context,
        Some("Undertow".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let lateralus_prop_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("2001".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    let func_binding_return_value = lateralus_prop_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let lateralus_prop_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *func.id(),
        *lateralus_prop_func_binding.id(),
        *func_binding_return_value.id(),
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
        AttributeContextBuilder::from(prop_hash_key_prototype_context)
            .set_component_id(*component.id())
            .to_context()
            .expect("cannot create attribute context");

    let lateralus_component_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("The Early 2000s".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    let func_binding_return_value = lateralus_component_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let lateralus_component_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *func.id(),
        *lateralus_component_func_binding.id(),
        *func_binding_return_value.id(),
        component_hash_key_prototype_context,
        Some("Lateralus".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let fear_inoculum_component_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(FuncBackendStringArgs::new("2019".to_string()))
            .expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    let func_binding_return_value = fear_inoculum_component_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");

    let fear_inoculum_component_prototype = AttributePrototype::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *func.id(),
        *fear_inoculum_component_func_binding.id(),
        *func_binding_return_value.id(),
        component_hash_key_prototype_context,
        Some("Fear Inoculum".to_string()),
        None,
    )
    .await
    .expect("cannot create attribute prototype for component album name");

    let found_hash_key_prototypes = AttributePrototype::list_for_context(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
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

/// Test attribute prototype removal corresponding to a least specific context.
#[test]
async fn remove_least_specific() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encryption_key,
    );
    let tenancy = Tenancy::new_universal();
    let read_tenancy = ReadTenancy::try_from_tenancy(&txn, tenancy.clone())
        .await
        .expect("could not convert tenancy to read tenancy");
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "toddhoward",
    )
    .await;

    let context = AttributeContextBuilder::new()
        .set_prop_id(*prop.id())
        .to_context()
        .expect("could not build context");

    let prototypes =
        AttributePrototype::list_for_context(&txn, &read_tenancy, &visibility, context)
            .await
            .expect("could not list attribute prototypes for context");

    for prototype in prototypes {
        let result = AttributePrototype::remove(
            &txn,
            &nats,
            &(&tenancy).into(),
            &visibility,
            &history_actor,
            prototype.id(),
        )
        .await;
        if let Err(AttributePrototypeError::LeastSpecificContextPrototypeRemovalNotAllowed(id)) =
            result
        {
            assert_eq!(prototype.id(), &id);
        } else {
            panic!("expected least-specific context not allowed for removal error, found the following result: {:?}", result);
        }
    }
}

/// Test attribute prototype removal corresponding to a component-specific context.
#[test]
async fn remove_component_specific() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encryption_key,
    );
    let tenancy = Tenancy::new_universal();
    let read_tenancy = ReadTenancy::try_from_tenancy(&txn, tenancy.clone())
        .await
        .expect("could not convert tenancy to read tenancy");
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
    let (schema_variant, root) = create_schema_variant_with_root(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encryption_key,
        *schema.id(),
    )
    .await;
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
    let prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "god",
    )
    .await;
    prop.set_parent_prop(
        &txn,
        &nats,
        &visibility,
        &history_actor,
        root.domain_prop_id,
    )
    .await
    .expect("cannot set parent of prop");
    let (component, _) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        "toddhoward",
        schema.id(),
    )
    .await
    .expect("cannot create component");

    let read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let component_view = ComponentView::for_context(&txn, &read_tenancy, &visibility, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "toddhoward",
                },
                "domain": {}
            }
        ],
        component_view.properties,
    );

    let context = AttributeContextBuilder::new()
        .set_prop_id(*prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("could not build context");

    let prototypes =
        AttributePrototype::list_for_context(&txn, &read_tenancy, &visibility, context)
            .await
            .expect("could not list attribute prototypes for context");

    for prototype in prototypes {
        // Ensure that performing remove on base prototypes on props results in failure.
        assert!(AttributePrototype::remove(
            &txn,
            &nats,
            &(&tenancy).into(),
            &visibility,
            &history_actor,
            prototype.id(),
        )
        .await
        .is_err());

        // Update the prototype for our component-specific context using its immediate value(s).
        // Updating each value for our context will result in our prototype being updated as well.
        let values = prototype
            .attribute_values(&txn, &tenancy, &visibility)
            .await
            .expect("could not get attribute values");
        for value in values {
            let parent_value_id = match value
                .parent_attribute_value(&txn, &visibility)
                .await
                .expect("could not get parent attribute_value")
            {
                Some(parent) => Some(*parent.id()),
                None => None,
            };

            AttributeValue::update_for_context(
                &txn,
                &nats,
                veritech.clone(),
                &encryption_key,
                &(&tenancy).into(),
                &visibility,
                &history_actor,
                *value.id(),
                parent_value_id,
                context,
                None,
                None,
            )
            .await
            .expect("could not update value");
        }

        // Now that the prototype's value(s) have been updated with our component-specific context,
        // we can perform removal.
        let updated_prototypes =
            AttributePrototype::list_for_context(&txn, &read_tenancy, &visibility, context)
                .await
                .expect("could not list attribute prototypes for context");

        for updated_prototype in updated_prototypes {
            // Find all the nested values and their corresponding prototypes for the updated
            // prototype. We will need them to check if they have been successfully deleted.
            let updated_values = updated_prototype
                .attribute_values(&txn, &tenancy, &visibility)
                .await
                .expect("could not get attribute values");

            let mut confirm_deletion_prototype_ids = vec![*updated_prototype.id()];
            let mut confirm_deletion_value_ids = Vec::new();

            let mut nested_values_work_queue = updated_values;
            while let Some(nested_value) = nested_values_work_queue.pop() {
                let child_attribute_values = nested_value
                    .child_attribute_values(&txn, &tenancy, &visibility)
                    .await
                    .expect("could not get child attribute values");
                if !child_attribute_values.is_empty() {
                    nested_values_work_queue.extend(child_attribute_values);
                }
                if let Some(current_prototype) = nested_value
                    .attribute_prototype(&txn, &visibility)
                    .await
                    .expect("could not get attribute prototype")
                {
                    confirm_deletion_prototype_ids.push(*current_prototype.id());
                }
                confirm_deletion_value_ids.push(*nested_value.id());
            }

            // Perform removal on the prototype.
            assert!(AttributePrototype::remove(
                &txn,
                &nats,
                &(&tenancy).into(),
                &visibility,
                &history_actor,
                updated_prototype.id(),
            )
            .await
            .is_ok());

            // Confirm the prototype, its nested values and their corresponding prototypes have
            // been deleted.
            for confirm_deletion_prototype_id in &confirm_deletion_prototype_ids {
                assert!(AttributePrototype::get_by_id(
                    &txn,
                    &tenancy,
                    &visibility,
                    &confirm_deletion_prototype_id
                )
                .await
                .expect("could not get attribute prototype by id")
                .is_none());
            }
            for confirm_deletion_value_id in confirm_deletion_value_ids {
                assert!(AttributeValue::get_by_id(
                    &txn,
                    &tenancy,
                    &visibility,
                    &confirm_deletion_value_id
                )
                .await
                .expect("could not get attribute value by id")
                .is_none());
            }
        }
    }
}
