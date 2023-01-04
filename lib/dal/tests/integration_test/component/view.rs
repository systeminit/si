use dal::{
    schema::RootProp, AttributeContext, AttributeValue, Component, ComponentView, DalContext, Prop,
    PropKind, Schema, SchemaKind, SchemaVariant, StandardModel,
};
use dal_test::test_harness::create_prop_and_set_parent;
use dal_test::{
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

mod complex_func;
mod cyclone_crypto;
mod properties;

/// Create a schema that looks like this:
/// ```json
/// { "queen": { "bohemian_rhapsody": "", "killer_queen": ""} }
/// ```
pub async fn create_schema_with_object_and_string_prop(
    ctx: &DalContext,
) -> (Schema, SchemaVariant, Prop, Prop, Prop, RootProp) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let queen_prop =
        create_prop_and_set_parent(ctx, PropKind::Object, "queen", root.domain_prop_id).await;
    let killer_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "killer_queen", *queen_prop.id()).await;
    let bohemian_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "bohemian_rhapsody", *queen_prop.id())
            .await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (
        schema,
        schema_variant,
        queen_prop,
        killer_prop,
        bohemian_prop,
        root,
    )
}

/// Create a schema that looks like this:
/// ```json
/// { "queen": { "bohemian_rhapsody": "", "killer_queen": "", "under_pressure": { "another_one_bites_the_dust": "" }} }
/// ```
pub async fn create_schema_with_nested_objects_and_string_prop(
    ctx: &DalContext,
) -> (
    Schema,
    SchemaVariant,
    Prop,
    Prop,
    Prop,
    Prop,
    Prop,
    RootProp,
) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let queen_prop =
        create_prop_and_set_parent(ctx, PropKind::Object, "queen", root.domain_prop_id).await;
    let bohemian_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "bohemian_rhapsody", *queen_prop.id())
            .await;
    let killer_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "killer_queen", *queen_prop.id()).await;
    let pressure_prop =
        create_prop_and_set_parent(ctx, PropKind::Object, "under_pressure", *queen_prop.id()).await;
    let dust_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "another_one_bites_the_dust",
        *pressure_prop.id(),
    )
    .await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (
        schema,
        schema_variant,
        queen_prop,
        bohemian_prop,
        killer_prop,
        pressure_prop,
        dust_prop,
        root,
    )
}

/// Create a schema that looks like this:
/// ```json
/// { "bohemian_rhapsody": "", "killer_queen": "" }
/// ```
pub async fn create_schema_with_string_props(
    ctx: &DalContext,
) -> (Schema, SchemaVariant, Prop, Prop, RootProp) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let bohemian_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "bohemian_rhapsody",
        root.domain_prop_id,
    )
    .await;
    let killer_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "killer_queen", root.domain_prop_id)
            .await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (schema, schema_variant, bohemian_prop, killer_prop, root)
}

/// Create a schema that looks like this:
/// ```json
/// { "sammy_hagar": ["standing hampton", "voa"] }
/// ```
pub async fn create_schema_with_array_of_string_props(
    ctx: &DalContext,
) -> (Schema, SchemaVariant, Prop, Prop, RootProp) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;

    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let sammy_prop =
        create_prop_and_set_parent(ctx, PropKind::Array, "sammy_hagar", root.domain_prop_id).await;
    let album_string_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "ignoreme", *sammy_prop.id()).await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (schema, schema_variant, sammy_prop, album_string_prop, root)
}

/// Create a schema that looks like this:
/// ```json
/// { "sammy_hagar": [
///    {"album": "standing_hampton", "songs": ["fall in love again", "surrender"]},
///    {"album": "voa", "songs": ["eagles fly", "cant drive 55"]}
///   ]
/// }
/// ```
pub async fn create_schema_with_nested_array_objects(
    ctx: &DalContext,
) -> (
    Schema,
    SchemaVariant,
    Prop,
    Prop,
    Prop,
    Prop,
    Prop,
    RootProp,
) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;

    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let sammy_prop =
        create_prop_and_set_parent(ctx, PropKind::Array, "sammy_hagar", root.domain_prop_id).await;
    let album_object_prop =
        create_prop_and_set_parent(ctx, PropKind::Object, "album_ignore", *sammy_prop.id()).await;
    let album_string_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "album", *album_object_prop.id()).await;
    let songs_array_prop =
        create_prop_and_set_parent(ctx, PropKind::Array, "songs", *album_object_prop.id()).await;
    let song_name_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "song_name_ignore",
        *songs_array_prop.id(),
    )
    .await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (
        schema,
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_name_prop,
        root,
    )
}

/// Create a schema that looks like this (its a map!):
/// ```json
/// "albums": {
///   "black_dahlia": "nocturnal",
///   "meshuggah": "destroy erase improve"
/// }
/// ```
pub async fn create_simple_map(ctx: &DalContext) -> (Schema, SchemaVariant, Prop, Prop, RootProp) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let album_prop =
        create_prop_and_set_parent(ctx, PropKind::Map, "albums", root.domain_prop_id).await;
    let album_item_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "album_ignore", *album_prop.id()).await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (schema, schema_variant, album_prop, album_item_prop, root)
}

/// Create a schema that looks like this:
/// ```json
/// { "sammy_hagar": [
///    {"album": "standing_hampton", "songs": [{ "fall in love again": "good", surrender": "ok"}]},
///   ]
/// }
/// ```
pub async fn create_schema_with_nested_array_objects_and_a_map(
    ctx: &DalContext,
) -> (
    Schema,
    SchemaVariant,
    Prop,
    Prop,
    Prop,
    Prop,
    Prop,
    Prop,
    RootProp,
) {
    let octx = ctx.clone_with_universal_head();
    let ctx = &octx;

    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let sammy_prop =
        create_prop_and_set_parent(ctx, PropKind::Array, "sammy_hagar", root.domain_prop_id).await;
    let album_object_prop =
        create_prop_and_set_parent(ctx, PropKind::Object, "album_ignore", *sammy_prop.id()).await;
    let album_string_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "album", *album_object_prop.id()).await;
    let songs_array_prop =
        create_prop_and_set_parent(ctx, PropKind::Array, "songs", *album_object_prop.id()).await;
    let song_map_prop = create_prop_and_set_parent(
        ctx,
        PropKind::Map,
        "song_map_ignore",
        *songs_array_prop.id(),
    )
    .await;
    let song_map_item_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "song_map_item_ignore",
        *song_map_prop.id(),
    )
    .await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    (
        schema,
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_map_prop,
        song_map_item_prop,
        root,
    )
}

#[test]
async fn only_string_props(ctx: &DalContext) {
    let (_schema, schema_variant, bohemian_prop, killer_prop, root_prop) =
        create_schema_with_string_props(ctx).await;
    let (component, _) = Component::new(ctx, "capoeira", *schema_variant.id())
        .await
        .expect("Unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("cannot create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not fetch domain AttributeValue")
        .expect("could not find domain AttributeValue");

    let bohemian_context = base_attribute_context
        .clone()
        .set_prop_id(*bohemian_prop.id())
        .to_context()
        .expect("cannot create bohemian AttributeContext");
    let bohemian_value = AttributeValue::find_for_context(ctx, bohemian_context.into())
        .await
        .expect("could not retrieve bohemian AttributeValue")
        .expect("could not find bohemian AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *bohemian_value.id(),
        Some(*domain_value.id()),
        bohemian_context,
        Some(serde_json::json!["Galileo"]),
        None,
    )
    .await
    .expect("could not update bohemian prop value");

    let killer_context = base_attribute_context
        .clone()
        .set_prop_id(*killer_prop.id())
        .to_context()
        .expect("cannot create killer AttributeContext");
    let killer_value = AttributeValue::find_for_context(ctx, killer_context.into())
        .await
        .expect("could not retrieve killer AttributeValue")
        .expect("could not find killer AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *killer_value.id(),
        Some(*domain_value.id()),
        killer_context,
        Some(serde_json::json!["woohoo"]),
        None,
    )
    .await
    .expect("could not update bohemian prop value");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "capoeira"
                },

                "domain": {
                    "bohemian_rhapsody": "Galileo",
                    "killer_queen": "woohoo"
                }
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn one_object_prop(ctx: &DalContext) {
    let (_schema, schema_variant, queen_prop, killer_prop, bohemian_prop, root_prop) =
        create_schema_with_object_and_string_prop(ctx).await;
    let (component, _) = Component::new(ctx, "santos dumont", *schema_variant.id())
        .await
        .expect("Unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("cannot create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not fetch domain AttributeValue")
        .expect("could not find domain AttributeValue");

    let queen_context = base_attribute_context
        .clone()
        .set_prop_id(*queen_prop.id())
        .to_context()
        .expect("cannot create queen AttributeContext");
    let unset_queen_value = AttributeValue::find_for_context(ctx, queen_context.into())
        .await
        .expect("could not retrieve queen AttributeValue")
        .expect("could not find queen AttributeValue");
    let (_, queen_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_queen_value.id(),
        Some(*domain_value.id()),
        queen_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not update queen AttributeValue");

    let bohemian_context = base_attribute_context
        .clone()
        .set_prop_id(*bohemian_prop.id())
        .to_context()
        .expect("cannot create bohemian AttributeContext");
    let unset_bohemian_value = AttributeValue::find_for_context(ctx, bohemian_context.into())
        .await
        .expect("could not retrieve bohemian AttributeValue")
        .expect("could not find bohemian AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *unset_bohemian_value.id(),
        Some(queen_value_id),
        bohemian_context,
        Some(serde_json::json!["Galileo"]),
        None,
    )
    .await
    .expect("could not update bohemian AttributeValue");

    let killer_context = base_attribute_context
        .clone()
        .set_prop_id(*killer_prop.id())
        .to_context()
        .expect("cannot create killer AttributeContext");
    let unset_killer_value = AttributeValue::find_for_context(ctx, killer_context.into())
        .await
        .expect("could not retrieve killer AttributeValue")
        .expect("could not find killer AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *unset_killer_value.id(),
        Some(queen_value_id),
        killer_context,
        Some(serde_json::json!["woohoo"]),
        None,
    )
    .await
    .expect("could not update killer AttributeValue");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        component_view.properties,
        serde_json::json![{
            "si": { "name": "santos dumont" },

            "domain": {
                "queen": {
                    "bohemian_rhapsody": "Galileo",
                    "killer_queen": "woohoo"
                }
            }
        }]
    );
}

#[test]
async fn nested_object_prop(ctx: &DalContext) {
    let (
        _schema,
        schema_variant,
        queen_prop,
        bohemian_prop,
        killer_prop,
        pressure_prop,
        dust_prop,
        root_prop,
    ) = create_schema_with_nested_objects_and_string_prop(ctx).await;
    let (component, _) = Component::new(ctx, "free ronaldinho", *schema_variant.id())
        .await
        .expect("Unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("cannot create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not fetch domain AttributeValue")
        .expect("could not find domain AttributeContext");

    let queen_context = base_attribute_context
        .clone()
        .set_prop_id(*queen_prop.id())
        .to_context()
        .expect("cannot create queen AttributeContext");
    let unset_queen_value = AttributeValue::find_for_context(ctx, queen_context.into())
        .await
        .expect("could not fetch queen AttributeValue")
        .expect("could not find queen AttributeValue");
    let (_, queen_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_queen_value.id(),
        Some(*domain_value.id()),
        queen_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not update queen AttributeValue");

    let bohemian_context = base_attribute_context
        .clone()
        .set_prop_id(*bohemian_prop.id())
        .to_context()
        .expect("cannot create bohemian AttributeContext");
    let unset_bohemian_value = AttributeValue::find_for_context(ctx, bohemian_context.into())
        .await
        .expect("could not fetch bohemian AttributeValue")
        .expect("could not find bohemian AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *unset_bohemian_value.id(),
        Some(queen_value_id),
        bohemian_context,
        Some(serde_json::json!["scaramouche"]),
        None,
    )
    .await
    .expect("could not update bohemian AttributeValue");

    let killer_context = base_attribute_context
        .clone()
        .set_prop_id(*killer_prop.id())
        .to_context()
        .expect("cannot create killer AttributeContext");
    let unset_killer_value = AttributeValue::find_for_context(ctx, killer_context.into())
        .await
        .expect("could not fetch killer AttributeValue")
        .expect("could not find killer AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *unset_killer_value.id(),
        Some(queen_value_id),
        killer_context,
        Some(serde_json::json!["cake"]),
        None,
    )
    .await
    .expect("could not update killer AttributeValue");

    let pressure_context = base_attribute_context
        .clone()
        .set_prop_id(*pressure_prop.id())
        .to_context()
        .expect("cannot create pressure AttributeContext");
    let unset_pressure_value = AttributeValue::find_for_context(ctx, pressure_context.into())
        .await
        .expect("could not fetch pressure AttributeValue")
        .expect("could not find pressure AttributeValue");
    let (_, pressure_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_pressure_value.id(),
        Some(queen_value_id),
        pressure_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not update pressure AttributeValue");

    let dust_context = base_attribute_context
        .clone()
        .set_prop_id(*dust_prop.id())
        .to_context()
        .expect("cannot create dust AttributeContext");
    let unset_dust_value = AttributeValue::find_for_context(ctx, dust_context.into())
        .await
        .expect("could not fetch dust AttributeValue")
        .expect("could not find dust AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *unset_dust_value.id(),
        Some(pressure_value_id),
        dust_context,
        Some(serde_json::json!["another one gone"]),
        None,
    )
    .await
    .expect("could not update dust AttributeValue");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "free ronaldinho"
                },

                "domain": {
                    "queen": {
                        "bohemian_rhapsody": "scaramouche",
                        "killer_queen": "cake",
                        "under_pressure": {
                            "another_one_bites_the_dust": "another one gone"
                        }
                    }
                }
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn simple_array_of_strings(ctx: &DalContext) {
    let (_schema, schema_variant, sammy_prop, album_prop, root_prop) =
        create_schema_with_array_of_string_props(ctx).await;

    let (component, _) = Component::new(ctx, "tim maia", *schema_variant.id())
        .await
        .expect("Unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("could not create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not retrieve domain AttributeValue")
        .expect("could not find domain AttributeValue");

    let sammy_context = base_attribute_context
        .clone()
        .set_prop_id(*sammy_prop.id())
        .to_context()
        .expect("could not create sammy AttributeContext");
    let unset_sammy_value = AttributeValue::find_for_context(ctx, sammy_context.into())
        .await
        .expect("could not retrieve sammy AttributeValue")
        .expect("could not find sammy AttributeValue");
    let (_, sammy_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_sammy_value.id(),
        Some(*domain_value.id()),
        sammy_context,
        Some(serde_json::json![[]]),
        None,
    )
    .await
    .expect("could not update sammy AttributeValue");

    let album_context = base_attribute_context
        .clone()
        .set_prop_id(*album_prop.id())
        .to_context()
        .expect("could not create album AttributeContext");
    let _ = AttributeValue::insert_for_context(
        ctx,
        album_context,
        sammy_value_id,
        Some(serde_json::json!["standing_hampton"]),
        None,
    )
    .await
    .expect("could not insert album AttributeValue");

    let _ = AttributeValue::insert_for_context(
        ctx,
        album_context,
        sammy_value_id,
        Some(serde_json::json!["voa"]),
        None,
    )
    .await
    .expect("could not insert album AttributeValue");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "tim maia"
                },

                "domain": {
                    "sammy_hagar": ["standing_hampton", "voa"]
                }
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn complex_nested_array_of_objects_and_arrays(ctx: &DalContext) {
    let (
        _schema,
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_name_prop,
        root_prop,
    ) = create_schema_with_nested_array_objects(ctx).await;
    let (component, _) = Component::new(
        ctx,
        "An Integralist Doesn't Run, It Flies",
        *schema_variant.id(),
    )
    .await
    .expect("Unable to create component");

    let unset_attribute_context = AttributeContext::builder();
    let mut base_attribute_context = unset_attribute_context;
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("could not create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not fetch domain AttributeValue")
        .expect("could not find domain AttributeValue");

    let unset_sammy_context = unset_attribute_context
        .clone()
        .set_prop_id(*sammy_prop.id())
        .to_context()
        .expect("could not create sammy AttributeContext");
    let unset_sammy_value = AttributeValue::find_for_context(ctx, unset_sammy_context.into())
        .await
        .expect("could not fetch sammy AttributeValue")
        .expect("could not find sammy AttributeValue");
    let sammy_context = base_attribute_context
        .clone()
        .set_prop_id(*sammy_prop.id())
        .to_context()
        .expect("could not create sammy AttributeContext");
    let (_, sammy_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_sammy_value.id(),
        Some(*domain_value.id()),
        sammy_context,
        Some(serde_json::json![[]]),
        None,
    )
    .await
    .expect("could not update sammy AttributeValue");

    let album_object_context = base_attribute_context
        .clone()
        .set_prop_id(*album_object_prop.id())
        .to_context()
        .expect("could not create album object AttributeContext");
    let standing_hampton_album_value_id = AttributeValue::insert_for_context(
        ctx,
        album_object_context,
        sammy_value_id,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not insert album object AttributeValue");

    let album_string_context = base_attribute_context
        .clone()
        .set_prop_id(*album_string_prop.id())
        .to_context()
        .expect("could not create album string AttributeContext");
    let standing_hampton_album_string_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        Some(standing_hampton_album_value_id),
        None,
        album_string_context.into(),
    )
    .await
    .expect("could not retrieve album string AttributeValue")
    .expect("could not find album string AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *standing_hampton_album_string_value.id(),
        Some(standing_hampton_album_value_id),
        album_string_context,
        Some(serde_json::json!["standing_hampton"]),
        None,
    )
    .await
    .expect("could not update standing hampton album string AttributeValue");

    let songs_array_context = base_attribute_context
        .clone()
        .set_prop_id(*songs_array_prop.id())
        .to_context()
        .expect("could not create songs array AttributeContext");
    let standing_hampton_songs_array_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        Some(standing_hampton_album_value_id),
        None,
        songs_array_context.into(),
    )
    .await
    .expect("could not fetch songs array AttributeValue")
    .expect("could not find songs array AttributeValue");
    let (_, standing_hampton_songs_array_value_id) = AttributeValue::update_for_context(
        ctx,
        *standing_hampton_songs_array_value.id(),
        Some(standing_hampton_album_value_id),
        songs_array_context,
        Some(serde_json::json![[]]),
        None,
    )
    .await
    .expect("could not update standing hampton songs array AttributeValue");

    let song_name_context = base_attribute_context
        .clone()
        .set_prop_id(*song_name_prop.id())
        .to_context()
        .expect("could not create song name AttributeContext");
    let _ = AttributeValue::insert_for_context(
        ctx,
        song_name_context,
        standing_hampton_songs_array_value_id,
        Some(serde_json::json!["fall in love again"]),
        None,
    )
    .await
    .expect("could not insert fall in love again in standing hampton songs array");

    let _ = AttributeValue::insert_for_context(
        ctx,
        song_name_context,
        standing_hampton_songs_array_value_id,
        Some(serde_json::json!["surrender"]),
        None,
    )
    .await
    .expect("could not insert surrender in standing hampton songs array");

    let voa_album_value_id = AttributeValue::insert_for_context(
        ctx,
        album_object_context,
        sammy_value_id,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not insert voa album object into albums array");

    let voa_album_string_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        Some(voa_album_value_id),
        None,
        album_string_context.into(),
    )
    .await
    .expect("could not retrieve voa album string AttributeValue")
    .expect("could not find voa album string AttributeValue");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        *voa_album_string_value.id(),
        Some(voa_album_value_id),
        album_string_context,
        Some(serde_json::json!["voa"]),
        None,
    )
    .await
    .expect("could not set voa album string AttributeValue");

    let voa_songs_array_value = AttributeValue::find_with_parent_and_key_for_context(
        ctx,
        Some(voa_album_value_id),
        None,
        songs_array_context.into(),
    )
    .await
    .expect("could not fetch songs array AttributeValue")
    .expect("could not find songs array AttributeValue");
    let (_, voa_songs_value_id) = AttributeValue::update_for_context(
        ctx,
        *voa_songs_array_value.id(),
        Some(voa_album_value_id),
        songs_array_context,
        Some(serde_json::json![[]]),
        None,
    )
    .await
    .expect("could not update voa songs array AttributeValue");

    let _ = AttributeValue::insert_for_context(
        ctx,
        song_name_context,
        voa_songs_value_id,
        Some(serde_json::json!["eagles fly"]),
        None,
    )
    .await
    .expect("could not insert eagles fly into voa songs array");

    let _ = AttributeValue::insert_for_context(
        ctx,
        song_name_context,
        voa_songs_value_id,
        Some(serde_json::json!["can't drive 55"]),
        None,
    )
    .await
    .expect("could not insert can't drive 55 into voa songs array");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![{
            "si": {"name": "An Integralist Doesn't Run, It Flies"},

            "domain": {
                "sammy_hagar": [
                    {
                        "album": "standing_hampton",
                        "songs": [
                            "fall in love again",
                            "surrender"
                        ]
                    },
                    {
                        "album": "voa",
                        "songs": [
                            "eagles fly",
                            "can't drive 55"
                        ]
                    }
                ]
            }
        }], // expected
        component_view.properties, // actual
    );
}

#[test]
async fn simple_map(ctx: &DalContext) {
    let (_schema, schema_variant, album_prop, album_item_prop, root_prop) =
        create_simple_map(ctx).await;
    let (component, _) = Component::new(ctx, "E como isso afeta o Grêmio?", *schema_variant.id())
        .await
        .expect("Unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("could not create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not retrieve domain AttributeValue")
        .expect("could not find domain AttributeValue");

    let album_context = base_attribute_context
        .clone()
        .set_prop_id(*album_prop.id())
        .to_context()
        .expect("could not create album AttributeContext");
    let unset_album_value = AttributeValue::find_for_context(ctx, album_context.into())
        .await
        .expect("could not retrieve album AttributeValue")
        .expect("could not find album AttributeValue");
    let (_, album_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_album_value.id(),
        Some(*domain_value.id()),
        album_context,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not update album AttributeValue");

    let album_item_context = base_attribute_context
        .clone()
        .set_prop_id(*album_item_prop.id())
        .to_context()
        .expect("could not create album item AttributeContext");
    let _ = AttributeValue::insert_for_context(
        ctx,
        album_item_context,
        album_value_id,
        Some(serde_json::json!["nocturnal"]),
        Some("black_dahlia".to_string()),
    )
    .await
    .expect("could not insert album item");

    let _ = AttributeValue::insert_for_context(
        ctx,
        album_item_context,
        album_value_id,
        Some(serde_json::json!["destroy erase improve"]),
        Some("meshuggah".to_string()),
    )
    .await
    .expect("could not insert album item");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![{
            "si": {"name": "E como isso afeta o Grêmio?"},

            "domain": {
                "albums": {
                    "black_dahlia": "nocturnal",
                    "meshuggah": "destroy erase improve",
                }
            }
        }], // expected
        component_view.properties, // actual
    );
}

#[test]
async fn complex_nested_array_of_objects_with_a_map(ctx: &DalContext) {
    let (
        _schema,
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_map_prop,
        song_map_item_prop,
        root_prop,
    ) = create_schema_with_nested_array_objects_and_a_map(ctx).await;
    let (component, _) = Component::new(ctx, "E como isso afeta o Grêmio?", *schema_variant.id())
        .await
        .expect("Unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("could not create domain AttributeContext");
    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("could not fetch domain AttributeValue")
        .expect("could not find domain AttributeValue");

    let sammy_context = base_attribute_context
        .clone()
        .set_prop_id(*sammy_prop.id())
        .to_context()
        .expect("could not create sammy AttributeContext");
    let unset_sammy_value = AttributeValue::find_for_context(ctx, sammy_context.into())
        .await
        .expect("could not fetch sammy AttributeValue")
        .expect("could not find sammy AttributeValue");
    let (_, sammy_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_sammy_value.id(),
        Some(*domain_value.id()),
        sammy_context,
        Some(serde_json::json!([])),
        None,
    )
    .await
    .expect("could not update sammy AttributeValue");

    let album_object_context = base_attribute_context
        .clone()
        .set_prop_id(*album_object_prop.id())
        .to_context()
        .expect("could not create album object context");
    let standing_hampton_value_id = AttributeValue::insert_for_context(
        ctx,
        album_object_context,
        sammy_value_id,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not insert standing_hampton into albums array");

    let album_string_context = base_attribute_context
        .clone()
        .set_prop_id(*album_string_prop.id())
        .to_context()
        .expect("could not create album string AttributeContext");
    let unset_album_string_value =
        AttributeValue::find_for_context(ctx, album_string_context.into())
            .await
            .expect("could not fetch album string AttributeValue")
            .expect("could not find album string AttributeValue");
    let _ = AttributeValue::update_for_context(
        ctx,
        *unset_album_string_value.id(),
        Some(standing_hampton_value_id),
        album_string_context,
        Some(serde_json::json!["standing_hampton"]),
        None,
    )
    .await
    .expect("could not update standing hampton album string AttributeValue");

    let songs_array_context = base_attribute_context
        .clone()
        .set_prop_id(*songs_array_prop.id())
        .to_context()
        .expect("could not create songs array AttributeContext");
    let unset_songs_array_value = AttributeValue::find_for_context(ctx, songs_array_context.into())
        .await
        .expect("could not fetch songs array AttributeValue")
        .expect("could not find songs array AttributeValue");
    let (_, songs_array_value_id) = AttributeValue::update_for_context(
        ctx,
        *unset_songs_array_value.id(),
        Some(standing_hampton_value_id),
        songs_array_context,
        Some(serde_json::json![[]]),
        None,
    )
    .await
    .expect("could not update songs array AttributeValue");

    let song_map_context = base_attribute_context
        .clone()
        .set_prop_id(*song_map_prop.id())
        .to_context()
        .expect("could not create song map AttributeContext");
    let song_map_value_id = AttributeValue::insert_for_context(
        ctx,
        song_map_context,
        songs_array_value_id,
        Some(serde_json::json![{}]),
        None,
    )
    .await
    .expect("could not insert song map into songs array");

    let song_map_item_context = base_attribute_context
        .clone()
        .set_prop_id(*song_map_item_prop.id())
        .to_context()
        .expect("could not create song map item AttributeContext");
    let _ = AttributeValue::insert_for_context(
        ctx,
        song_map_item_context,
        song_map_value_id,
        Some(serde_json::json!["good"]),
        Some("fall in love again".to_string()),
    )
    .await
    .expect("could not insert fall in love again into standing hampton songs map");

    let _ = AttributeValue::insert_for_context(
        ctx,
        song_map_item_context,
        song_map_value_id,
        Some(serde_json::json!["ok"]),
        Some("surrender".to_string()),
    )
    .await
    .expect("could not insert surrender into standing hampton song map");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![{
            "si": { "name": "E como isso afeta o Grêmio?" },

            "domain": {
                "sammy_hagar": [
                    {
                        "album": "standing_hampton",
                        "songs": [
                            { "fall in love again": "good", "surrender": "ok"},
                        ]
                    },
                ]
            }
        }], // expected
        component_view.properties, // actual
    );
}
