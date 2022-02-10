use dal::{
    system::UNSET_SYSTEM_ID,
    test_harness::{
        create_component_for_schema_variant, create_prop_of_kind_with_name, create_schema,
        create_schema_variant,
    },
    ComponentView, HistoryActor, PropKind, SchemaKind, SchemaVariant, StandardModel, Tenancy,
    Visibility,
};
use si_data::{NatsTxn, PgTxn};

use crate::test_setup;

/// Create a schema that looks like this:
/// ```json
/// { "queen": { "bohemian_rhapsody": "", "killer_queen": ""} }
/// ```
pub async fn create_schema_with_object_and_string_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
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
    let schema_variant =
        create_schema_variant(txn, nats, &tenancy, &visibility, &history_actor).await;
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
    let schema_variant =
        create_schema_variant(txn, nats, &tenancy, &visibility, &history_actor).await;
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

    schema_variant
}

/// Create a schema that looks like this:
/// ```json
/// { "bohemian_rhapsody": "", "killer_queen": "" }
/// ```
pub async fn create_schema_with_string_props(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
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
    let schema_variant =
        create_schema_variant(txn, nats, &tenancy, &visibility, &history_actor).await;
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

    let killer_prop = create_prop_of_kind_with_name(
        txn,
        nats,
        veritech,
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
    schema_variant
}

#[tokio::test]
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
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_variant = create_schema_with_string_props(&txn, &nats, veritech.clone()).await;
    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;
    let props = schema_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props for schema_variant");
    for prop in props.iter() {
        component
            .resolve_attribute(
                &txn,
                &nats,
                veritech.clone(),
                &tenancy,
                &visibility,
                &history_actor,
                prop,
                Some(serde_json::json!["woohoo"]),
            )
            .await
            .expect("cannot resolve the attributes for the component");
    }
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    txn.commit().await.expect("cannot commit txn");
    assert_eq!(component_view.name, component.name());
    assert_eq!(
        component_view.properties,
        serde_json::json![{"bohemian_rhapsody": "woohoo", "killer_queen": "woohoo"}]
    );
}

#[tokio::test]
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
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_variant =
        create_schema_with_object_and_string_prop(&txn, &nats, veritech.clone()).await;
    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;
    let props = schema_variant
        .all_props(&txn, &visibility)
        .await
        .expect("cannot get all props");
    for prop in props.iter() {
        // TODO: This should happen automatically when required
        if prop.name() == "queen" {
            component
                .resolve_attribute(
                    &txn,
                    &nats,
                    veritech.clone(),
                    &tenancy,
                    &visibility,
                    &history_actor,
                    prop,
                    Some(serde_json::json![{}]),
                )
                .await
                .expect("cannot resolve object attribute to empty object");
        } else {
            component
                .resolve_attribute(
                    &txn,
                    &nats,
                    veritech.clone(),
                    &tenancy,
                    &visibility,
                    &history_actor,
                    prop,
                    Some(serde_json::json!["woohoo"]),
                )
                .await
                .expect("cannot resolve the attributes for the component");
        }
    }
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    txn.commit().await.expect("cannot commit txn");
    assert_eq!(component_view.name, component.name());
    assert_eq!(
        component_view.properties,
        serde_json::json![{"queen": {"bohemian_rhapsody": "woohoo", "killer_queen": "woohoo"}}]
    );
}

#[tokio::test]
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
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_variant =
        create_schema_with_nested_objects_and_string_prop(&txn, &nats, veritech.clone()).await;
    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;
    let props = schema_variant
        .all_props(&txn, &visibility)
        .await
        .expect("cannot get all props");
    for prop in props.iter() {
        // TODO: This should happen automatically when required
        if prop.name() == "queen" || prop.name() == "under_pressure" {
            component
                .resolve_attribute(
                    &txn,
                    &nats,
                    veritech.clone(),
                    &tenancy,
                    &visibility,
                    &history_actor,
                    prop,
                    Some(serde_json::json![{}]),
                )
                .await
                .expect("cannot resolve object attribute to empty object");
        } else {
            component
                .resolve_attribute(
                    &txn,
                    &nats,
                    veritech.clone(),
                    &tenancy,
                    &visibility,
                    &history_actor,
                    prop,
                    Some(serde_json::json!["woohoo"]),
                )
                .await
                .expect("cannot resolve the attributes for the component");
        }
    }
    let component_view = ComponentView::for_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        UNSET_SYSTEM_ID,
    )
    .await
    .expect("cannot get component view");
    txn.commit().await.expect("cannot commit txn");
    assert_eq!(component_view.name, component.name());
    assert_eq!(
        component_view.properties,
        serde_json::json![{"queen": {"bohemian_rhapsody": "woohoo", "killer_queen": "woohoo", "under_pressure": { "another_one_bites_the_dust": "woohoo"}}}]
    );
}
