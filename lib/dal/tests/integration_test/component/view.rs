use crate::dal::test;
use dal::{
    system::UNSET_SYSTEM_ID,
    test_harness::{
        create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root,
        find_or_create_production_system,
    },
    Component, ComponentView, HistoryActor, Prop, PropKind, SchemaKind, SchemaVariant,
    StandardModel, Tenancy, Visibility,
};
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};
use si_data::{NatsTxn, PgTxn};
use tokio::sync::mpsc;
use veritech::EncryptionKey;

use crate::test_setup;

/// Create a schema that looks like this:
/// ```json
/// { "queen": { "bohemian_rhapsody": "", "killer_queen": ""} }
/// ```
pub async fn create_schema_with_object_and_string_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaVariant {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let bohemian_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "bohemian_rhapsody",
    )
    .await;

    let killer_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "killer_queen",
    )
    .await;

    let queen_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech,
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "queen",
    )
    .await;
    queen_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    queen_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent prop");
    killer_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *queen_prop.id())
        .await
        .expect("cannot set parent prop");
    bohemian_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *queen_prop.id())
        .await
        .expect("cannot set parent prop");

    schema_variant
}

/// Create a schema that looks like this:
/// ```json
/// { "queen": { "bohemian_rhapsody": "", "killer_queen": "", "under_pressure": { "another_one_bites_the_dust": "" }} }
/// ```
pub async fn create_schema_with_nested_objects_and_string_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> (SchemaVariant, Prop, Prop, Prop, Prop, Prop) {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let bohemian_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "bohemian_rhapsody",
    )
    .await;

    let killer_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "killer_queen",
    )
    .await;

    let pressure_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "under_pressure",
    )
    .await;

    let dust_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "another_one_bites_the_dust",
    )
    .await;
    dust_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *pressure_prop.id())
        .await
        .expect("cannot set parent prop");

    let queen_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech,
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "queen",
    )
    .await;
    queen_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    queen_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent prop");
    killer_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *queen_prop.id())
        .await
        .expect("cannot set parent prop");
    bohemian_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *queen_prop.id())
        .await
        .expect("cannot set parent prop");
    pressure_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *queen_prop.id())
        .await
        .expect("cannot set parent prop");

    (
        schema_variant,
        queen_prop,
        bohemian_prop,
        killer_prop,
        pressure_prop,
        dust_prop,
    )
}

/// Create a schema that looks like this:
/// ```json
/// { "bohemian_rhapsody": "", "killer_queen": "" }
/// ```
pub async fn create_schema_with_string_props(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaVariant {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let bohemian_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "bohemian_rhapsody",
    )
    .await;
    bohemian_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    bohemian_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent prop");

    let killer_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech,
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "killer_queen",
    )
    .await;
    killer_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    killer_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent prop");
    schema_variant
}

/// Create a schema that looks like this:
/// ```json
/// { "sammy_hagar": ["standing hampton", "voa"] }
/// ```
pub async fn create_schema_with_array_of_string_props(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaVariant {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let sammy_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "sammy_hagar",
    )
    .await;
    sammy_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    sammy_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent");

    let album_string_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech,
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "ignoreme",
    )
    .await;
    album_string_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *sammy_prop.id())
        .await
        .expect("cannot set parent");
    schema_variant
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
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> (SchemaVariant, Prop, Prop, Prop, Prop, Prop) {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let sammy_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "sammy_hagar",
    )
    .await;
    sammy_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    sammy_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent");

    let album_object_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "album_ignore",
    )
    .await;
    album_object_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *sammy_prop.id())
        .await
        .expect("cannot set parent");

    let album_string_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "album",
    )
    .await;
    album_string_prop
        .set_parent_prop(
            txn,
            nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent");

    let songs_array_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "songs",
    )
    .await;
    songs_array_prop
        .set_parent_prop(
            txn,
            nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent");

    let song_name_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "song_name_ignore",
    )
    .await;
    song_name_prop
        .set_parent_prop(
            txn,
            nats,
            &visibility,
            &history_actor,
            *songs_array_prop.id(),
        )
        .await
        .expect("cannot set parent");

    (
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_name_prop,
    )
}

/// Create a schema that looks like this (its a map!):
/// ```json
/// "albums": {
///   "black_dahlia": "nocturnal",
///   "meshuggah": "destroy erase improve"
/// }
/// ```
pub async fn create_simple_map(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> (SchemaVariant, Prop, Prop) {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let album_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Map,
        "albums",
    )
    .await;
    album_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    album_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent");

    let album_item_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "album_ignore",
    )
    .await;
    album_item_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *album_prop.id())
        .await
        .expect("cannot set parent");

    (schema_variant, album_prop, album_item_prop)
}

/// Create a schema that looks like this:
/// ```json
/// { "sammy_hagar": [
///    {"album": "standing_hampton", "songs": [{ "fall in love again": "good", surrender": "ok"}]},
///   ]
/// }
/// ```
pub async fn create_schema_with_nested_array_objects_and_a_map(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> (SchemaVariant, Prop, Prop, Prop, Prop, Prop, Prop) {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut schema = create_schema(
        txn,
        nats,
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
    )
    .await;
    schema_variant
        .set_schema(txn, nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            txn,
            nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let sammy_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "sammy_hagar",
    )
    .await;
    sammy_prop
        .add_schema_variant(txn, nats, &visibility, &history_actor, schema_variant.id())
        .await
        .expect("cannot associate prop with schema variant");
    sammy_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, root.domain_prop_id)
        .await
        .expect("cannot set parent");

    let album_object_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "album_ignore",
    )
    .await;
    album_object_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *sammy_prop.id())
        .await
        .expect("cannot set parent");

    let album_string_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "album",
    )
    .await;
    album_string_prop
        .set_parent_prop(
            txn,
            nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent");

    let songs_array_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Array,
        "songs",
    )
    .await;
    songs_array_prop
        .set_parent_prop(
            txn,
            nats,
            &visibility,
            &history_actor,
            *album_object_prop.id(),
        )
        .await
        .expect("cannot set parent");

    let song_map_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Map,
        "song_map_ignore",
    )
    .await;
    song_map_prop
        .set_parent_prop(
            txn,
            nats,
            &visibility,
            &history_actor,
            *songs_array_prop.id(),
        )
        .await
        .expect("cannot set parent");

    let song_map_item_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "song_map_item_ignore",
    )
    .await;
    song_map_item_prop
        .set_parent_prop(txn, nats, &visibility, &history_actor, *song_map_prop.id())
        .await
        .expect("cannot set parent");

    (
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_map_prop,
        song_map_item_prop,
    )
}

#[test]
async fn only_string_props() {
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
    let schema_variant =
        create_schema_with_string_props(&txn, &nats, veritech.clone(), encr_key).await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "capoeira",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let props = schema_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props for schema_variant");
    for prop in props
        .iter()
        .filter(|p| !["root", "si", "domain", "name"].contains(&p.name()))
    {
        component
            .resolve_attribute(
                &txn,
                &nats,
                veritech.clone(),
                encr_key,
                &tenancy,
                &visibility,
                &history_actor,
                prop,
                Some(serde_json::json!["woohoo"]),
                None,
                None,
                UNSET_SYSTEM_ID,
            )
            .await
            .expect("cannot resolve the attributes for the component");
    }
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    assert_eq!(
        component_view.properties,
        serde_json::json![{ "si": { "name": "capoeira" }, "domain": { "bohemian_rhapsody": "woohoo", "killer_queen": "woohoo" } }]
    );
}

#[test]
async fn one_object_prop() {
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
    let schema_variant =
        create_schema_with_object_and_string_prop(&txn, &nats, veritech.clone(), encr_key).await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "santos dumont",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let props = schema_variant
        .all_props(&txn, &visibility)
        .await
        .expect("cannot get all props");
    let object_prop = props
        .iter()
        .find(|p| p.name() == "queen")
        .expect("could not get object prop");
    // TODO: Resolving the object's value should happen automatically when children get values, but isn't handled yet
    let (_, object_attribute_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            object_prop,
            Some(serde_json::json![{}]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve object attribute to empty object");
    for prop in props
        .iter()
        .filter(|p| !["queen", "root", "si", "domain", "name"].contains(&p.name()))
    {
        component
            .resolve_attribute(
                &txn,
                &nats,
                veritech.clone(),
                encr_key,
                &tenancy,
                &visibility,
                &history_actor,
                prop,
                Some(serde_json::json!["woohoo"]),
                Some(object_attribute_resolver_id),
                None,
                UNSET_SYSTEM_ID,
            )
            .await
            .expect("cannot resolve the attributes for the component");
    }
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    assert_eq_sorted!(
        component_view.properties,
        serde_json::json![{
            "si": { "name": "santos dumont" },
            "domain": { "queen": { "bohemian_rhapsody": "woohoo", "killer_queen": "woohoo" } }
        }]
    );
}

#[test]
async fn nested_object_prop() {
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
    let (schema_variant, queen_prop, bohemian_prop, killer_prop, pressure_prop, dust_prop) =
        create_schema_with_nested_objects_and_string_prop(&txn, &nats, veritech.clone(), encr_key)
            .await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "free ronaldinho",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let (_, queen_object_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &queen_prop,
            Some(serde_json::json![{}]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve queen object prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &bohemian_prop,
            Some(serde_json::json!["scaramouche"]),
            Some(queen_object_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve bohemian prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &killer_prop,
            Some(serde_json::json!["cake"]),
            Some(queen_object_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve killer prop");
    let (_, pressure_object_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &pressure_prop,
            Some(serde_json::json![{}]),
            Some(queen_object_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve pressure prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &dust_prop,
            Some(serde_json::json!["another one gone"]),
            Some(pressure_object_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve dust prop");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    assert_eq!(
        component_view.properties,
        serde_json::json![{ "si": { "name": "free ronaldinho" }, "domain": {"queen": {"bohemian_rhapsody": "scaramouche", "killer_queen": "cake", "under_pressure": { "another_one_bites_the_dust": "another one gone"}}}}]
    );
}

#[test]
async fn simple_array_of_strings() {
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
    let schema_variant =
        create_schema_with_array_of_string_props(&txn, &nats, veritech.clone(), encr_key).await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "tim maia",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let props = schema_variant
        .all_props(&txn, &visibility)
        .await
        .expect("cannot get props for schema_variant");
    dbg!(&props);
    let array_prop = props
        .iter()
        .find(|p| p.name() == "sammy_hagar")
        .expect("could not find array prop");
    let (_, array_attribute_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            array_prop,
            Some(serde_json::json![[]]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve the attributes for the component");
    for prop in props
        .iter()
        .filter(|p| !["sammy_hagar", "root", "si", "domain", "name"].contains(&p.name()))
    {
        component
            .resolve_attribute(
                &txn,
                &nats,
                veritech.clone(),
                encr_key,
                &tenancy,
                &visibility,
                &history_actor,
                prop,
                Some(serde_json::json!["standing_hampton"]),
                Some(array_attribute_resolver_id),
                None,
                UNSET_SYSTEM_ID,
            )
            .await
            .expect("cannot resolve the attributes for the component");
        component
            .resolve_attribute(
                &txn,
                &nats,
                veritech.clone(),
                encr_key,
                &tenancy,
                &visibility,
                &history_actor,
                prop,
                Some(serde_json::json!["voa"]),
                Some(array_attribute_resolver_id),
                None,
                UNSET_SYSTEM_ID,
            )
            .await
            .expect("cannot resolve the attributes for the component");
    }
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    // txn.commit().await.expect("cannot commit txn");
    assert_eq!(
        component_view.properties,
        serde_json::json![{"si": {"name": "tim maia" }, "domain": {"sammy_hagar": ["standing_hampton", "voa"]}}]
    );
}

#[test]
async fn complex_nested_array_of_objects() {
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
    let (
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_name_prop,
    ) = create_schema_with_nested_array_objects(&txn, &nats, veritech.clone(), encr_key).await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "An Integralist Doesn't Run, It Flies",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let (_, sammy_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &sammy_prop,
            Some(serde_json::json![[]]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve sammy prop");
    let (_, standing_hampton_album_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_object_prop,
            Some(serde_json::json![{}]),
            Some(sammy_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve album object prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_string_prop,
            Some(serde_json::json!["standing_hampton"]),
            Some(standing_hampton_album_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve album name for standing hampton");
    let (_, standing_hampton_songs_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &songs_array_prop,
            Some(serde_json::json![[]]),
            Some(standing_hampton_album_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve songs prop for standing hampton");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_name_prop,
            Some(serde_json::json!["fall in love again"]),
            Some(standing_hampton_songs_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve song for standing hampton");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_name_prop,
            Some(serde_json::json!["surrender"]),
            Some(standing_hampton_songs_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve song for standing hampton");
    let (_, voa_album_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_object_prop,
            Some(serde_json::json![{}]),
            Some(sammy_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve voa album object");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_string_prop,
            Some(serde_json::json!["voa"]),
            Some(voa_album_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve voa album name");
    let (_, voa_songs_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &songs_array_prop,
            Some(serde_json::json![[]]),
            Some(voa_album_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve voa songs array prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_name_prop,
            Some(serde_json::json!["eagles fly"]),
            Some(voa_songs_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("could not resolve eagles fly");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_name_prop,
            Some(serde_json::json!["can't drive 55"]),
            Some(voa_songs_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("could not resolve driving 55");
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    // txn.commit().await.expect("cannot commit txn");
    assert_eq_sorted!(
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
async fn simple_map() {
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
    let (schema_variant, album_prop, album_item_prop) =
        create_simple_map(&txn, &nats, veritech.clone(), encr_key).await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "E como isso afeta o Grêmio?",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let (_, album_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_prop,
            Some(serde_json::json![{}]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve sammy prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_item_prop,
            Some(serde_json::json!["nocturnal"]),
            Some(album_resolver_id),
            Some("black_dahlia".to_string()),
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve album object prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_item_prop,
            Some(serde_json::json!["destroy erase improve"]),
            Some(album_resolver_id),
            Some("meshuggah".to_string()),
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve album object prop");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    // txn.commit().await.expect("cannot commit txn");
    assert_eq_sorted!(
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
async fn complex_nested_array_of_objects_with_a_map() {
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
    let (
        schema_variant,
        sammy_prop,
        album_object_prop,
        album_string_prop,
        songs_array_prop,
        song_map_prop,
        song_map_item_prop,
    ) = create_schema_with_nested_array_objects_and_a_map(&txn, &nats, veritech.clone(), encr_key)
        .await;
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "E como isso afeta o Grêmio?",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");
    let (_, sammy_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &sammy_prop,
            Some(serde_json::json![[]]),
            None,
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve sammy prop");
    let (_, standing_hampton_album_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_object_prop,
            Some(serde_json::json![{}]),
            Some(sammy_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve album object prop");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &album_string_prop,
            Some(serde_json::json!["standing_hampton"]),
            Some(standing_hampton_album_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve album name for standing hampton");
    let (_, standing_hampton_songs_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &songs_array_prop,
            Some(serde_json::json![[]]),
            Some(standing_hampton_album_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve songs prop for standing hampton");
    let (_, standing_hampton_songs_first_map_resolver_id, _) = component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_map_prop,
            Some(serde_json::json![{}]),
            Some(standing_hampton_songs_resolver_id),
            None,
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve songs map prop for standing hampton");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_map_item_prop,
            Some(serde_json::json!["good"]),
            Some(standing_hampton_songs_first_map_resolver_id),
            Some("fall in love again".to_string()),
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve songs map prop for standing hampton");
    component
        .resolve_attribute(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            &song_map_item_prop,
            Some(serde_json::json!["ok"]),
            Some(standing_hampton_songs_first_map_resolver_id),
            Some("surrender".to_string()),
            UNSET_SYSTEM_ID,
        )
        .await
        .expect("cannot resolve songs map prop for standing hampton");

    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");

    // txn.commit().await.expect("cannot commit txn");
    assert_eq_sorted!(
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

#[test]
async fn cyclone_crypto_e2e() {
    test_setup!(
        _ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
    );
    let (tx, _rx) = mpsc::channel(64);
    let secret_value = "Beware Cuca will catch you";
    let secret = serde_json::to_string(&serde_json::json!({
        "key": secret_value,
    }))
    .expect("Secret serialization failed");
    let encoded = encr_key.encrypt_and_encode(&secret);
    let code = format!("function testE2ECrypto(component) {{ return component.data.properties.secret.message.key === '{secret_value}'; }}");
    let request = veritech::ResolverFunctionRequest {
        execution_id: "seujorge".to_owned(),
        handler: "testE2ECrypto".to_owned(),
        component: veritech::ResolverFunctionComponent {
            data: veritech::ComponentView {
                kind: veritech::ComponentKind::Credential,
                system: None,
                properties: serde_json::json!({
                    "secret": {
                        "name": "ufo",
                        "secret_kind": "dockerHub",
                        "object_type": "credential",
                        "message": {
                            "cycloneEncryptedDataMarker": true,
                            "encryptedSecret": encoded,
                        },
                    },
                }),
            },
            parents: Vec::new(),
        },
        code_base64: base64::encode(&code),
    };
    let result = veritech
        .execute_resolver_function(tx, &request)
        .await
        .expect("Veritech run failed");
    match result {
        veritech::FunctionResult::Success(result) => {
            assert_eq!(result.data, serde_json::Value::Bool(true))
        }
        veritech::FunctionResult::Failure(err) => panic!("Veritech run failed: {:?}", err),
    }
}
