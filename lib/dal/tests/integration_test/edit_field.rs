use crate::{dal::test, test_setup};
use dal::{
    component::Component,
    edit_field::{EditField, EditFieldAble, EditFieldDataType},
    test_harness::{
        create_prop_of_kind_and_set_parent_with_name, create_schema,
        create_schema_variant_with_root,
    },
    HistoryActor, PropKind, ReadTenancy, SchemaKind, StandardModel, Tenancy, Visibility,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn get_edit_fields_for_component() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encryption_key,
    );

    let tenancy = Tenancy::new_universal();
    let read_tenancy = ReadTenancy::try_from_tenancy(&txn, tenancy.clone())
        .await
        .expect("could not get ReadTenancy from Tenancy");
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

    // domain: Object
    // |- object: Object
    //    |- name: String
    //    |- value: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
        "object",
        root.domain_prop_id,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "name",
        *object_prop.id(),
    )
    .await;
    let _value_prop = create_prop_of_kind_and_set_parent_with_name(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
        "value",
        *object_prop.id(),
    )
    .await;

    let (component, _) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        "radahn",
        schema.id(),
    )
    .await
    .expect("cannot create component");

    let edit_fields = Component::get_edit_fields(&txn, &read_tenancy, &visibility, component.id())
        .await
        .expect("could not get edit fields");

    fn recursive_edit_fields(edit_fields: Vec<EditField>) -> Vec<EditField> {
        let mut temp: Vec<EditField> = Vec::new();
        for edit_field in &edit_fields {
            if let Some(children) = edit_field.get_child_edit_fields() {
                temp.append(&mut recursive_edit_fields(children));
            }
        }
        let mut combined = edit_fields;
        combined.append(&mut temp);
        combined
    }

    // FIXME(nick): use HashSet for automatic ordering.
    let edit_fields = recursive_edit_fields(edit_fields);
    let mut found: Vec<(&str, &EditFieldDataType)> = Vec::new();
    for edit_field in &edit_fields {
        found.push((edit_field.id(), edit_field.data_type()));
    }

    let mut expected = vec![
        ("properties.root", &EditFieldDataType::Object),
        ("properties.root.si", &EditFieldDataType::Object),
        ("properties.root.domain", &EditFieldDataType::Object),
        ("properties.root.si.name", &EditFieldDataType::String),
        ("properties.root.domain.object", &EditFieldDataType::Object),
        (
            "properties.root.domain.object.name",
            &EditFieldDataType::String,
        ),
        (
            "properties.root.domain.object.value",
            &EditFieldDataType::String,
        ),
    ];

    // FIXME(nick): use HashSet to avoid sorting.
    found.sort_by_key(|v| v.0);
    expected.sort_by_key(|v| v.0);
    assert_eq!(found, expected);
}
