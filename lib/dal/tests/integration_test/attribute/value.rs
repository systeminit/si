use crate::{dal::test, test_setup};

use dal::{
    component::view::ComponentView,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeContext, AttributeReadContext, AttributeValue, Component, HistoryActor, PropKind,
    ReadTenancy, SchemaKind, StandardModel, Tenancy, Visibility,
};
use pretty_assertions_sorted::assert_eq_sorted;

#[test]
async fn update_for_context_simple() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    // "name": String
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

    let name_prop = create_prop_of_kind_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "name_prop",
    )
    .await;
    name_prop
        .set_parent_prop(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            root.domain_prop_id,
        )
        .await
        .expect("cannot set parent of name_prop");

    let (component, _) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "Basic component",
        schema.id(),
    )
    .await
    .expect("Unable to create component");

    let read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let read_tenancy = ReadTenancy::try_from_tenancy(&txn, tenancy.clone())
        .await
        .expect("could not convert tenancy to read tenancy");
    let component_view = ComponentView::for_context(&txn, &read_tenancy, &visibility, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {},
            }
        ],
        component_view.properties,
    );

    let domain_value_id = *AttributeValue::find_for_context(
        &txn,
        &read_tenancy,
        &visibility,
        AttributeReadContext {
            prop_id: Some(root.domain_prop_id),
            component_id: Some(*component.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get domain AttributeValue")
    .pop()
    .expect("domain AttributeValue not found")
    .id();
    let base_name_value =
        AttributeValue::find_for_prop(&txn, &read_tenancy, &visibility, *name_prop.id())
            .await
            .expect("cannot get base prop value");

    let update_context = AttributeContext::builder()
        .set_prop_id(*name_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let (_, name_value_id) = AttributeValue::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        *base_name_value.id(),
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Miles".to_string()).expect("cannot create new Value")),
        None,
    )
    .await
    .expect("cannot set value for context");

    let component_view = ComponentView::for_context(&txn, &read_tenancy, &visibility, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "name_prop": "Miles",
                },
            }
        ],
        component_view.properties,
    );

    AttributeValue::update_for_context(
        &txn,
        &nats,
        veritech.clone(),
        encr_key,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        name_value_id,
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Iria".to_string()).expect("cannot create new value")),
        None,
    )
    .await
    .expect("cannot update value for context");

    let component_view = ComponentView::for_context(&txn, &read_tenancy, &visibility, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "name_prop": "Iria",
                },
            }
        ],
        component_view.properties,
    );
}
