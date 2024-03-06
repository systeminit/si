use dal::{
    socket::SocketArity, DalContext, ExternalProvider, InternalProvider, Schema, SchemaVariant,
    StandardModel,
};
use dal_test::{
    connection_annotation_string,
    helpers::setup_identity_func,
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};

mod inter_component;
mod intra_component;

#[test]
async fn new_external(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (schema_variant, _root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let (func_id, func_binding_id, func_binding_return_value_id, _) =
        setup_identity_func(ctx).await;

    let (external_provider, output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "poop",
        None,
        func_id,
        func_binding_id,
        func_binding_return_value_id,
        connection_annotation_string!("poop"),
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create external provider");

    let found_external_provider = ExternalProvider::find_for_socket(ctx, *output_socket.id())
        .await
        .expect("could not find external provider for output socket")
        .expect("external provider for output socket not found");

    assert_eq!(
        *found_external_provider.id(), // actual
        *external_provider.id()        // expected
    );
}

#[test]
async fn new_implicit_internal(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (schema_variant, _root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let (func_id, func_binding_id, func_binding_return_value_id, _) =
        setup_identity_func(ctx).await;

    let (explicit_internal_provider, input_socket) = InternalProvider::new_explicit_with_socket(
        ctx,
        *schema_variant.id(),
        "poop",
        func_id,
        func_binding_id,
        func_binding_return_value_id,
        connection_annotation_string!("poop"),
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create (explicit internal provider");

    let found_explicit_internal_provider =
        InternalProvider::find_explicit_for_socket(ctx, *input_socket.id())
            .await
            .expect("could not find explicit internal provider for input socket")
            .expect("explicit internal provider for input socket not found");

    assert_eq!(
        *found_explicit_internal_provider.id(), // actual
        *explicit_internal_provider.id()        // expected
    );
}

/// Use the following environment variable when running the test:
/// ```bash
/// SI_TEST_BUILTIN_SCHEMAS=test
/// ```
#[test]
async fn is_for_root_prop(ctx: &DalContext) {
    let schema = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("could not find the schema");
    let schema_variant_id = *schema
        .default_schema_variant_id()
        .expect("no default schema variant id");
    let root_prop = SchemaVariant::find_root_prop(ctx, schema_variant_id)
        .await
        .expect("could not perform find root prop")
        .expect("no root prop found");
    let internal_provider_for_root_prop = InternalProvider::find_for_prop(ctx, *root_prop.id())
        .await
        .expect("could not perform find for prop")
        .expect("no internal provider found");

    // Check if the query works.
    let is_for_root_prop =
        InternalProvider::is_for_root_prop(ctx, *internal_provider_for_root_prop.id())
            .await
            .expect("could not check if the internal provider is for a root prop");
    assert!(is_for_root_prop);

    // Now ensure it fails for non-root props.
    let children_of_root = root_prop
        .child_props(ctx)
        .await
        .expect("could not get child props");
    for child in children_of_root {
        let internal_provider = InternalProvider::find_for_prop(ctx, *child.id())
            .await
            .expect("could not perform find for prop")
            .expect("no internal provider found");
        let is_for_root_prop = InternalProvider::is_for_root_prop(ctx, *internal_provider.id())
            .await
            .expect("could not check if the internal provider is for a root prop");
        assert!(!is_for_root_prop);
    }
}
