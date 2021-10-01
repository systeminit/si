use crate::schema::prop::PropArray;
use crate::{Prop, PropBoolean, PropMap, PropNumber, PropObject, PropString, Schema};
use names::{Generator, Name};
use si_data::{NatsTxn, PgTxn};

pub async fn create_new_schema(txn: &PgTxn<'_>, nats: &NatsTxn) -> Schema {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    Schema::new(&txn, &nats, &name, &name, &name)
        .await
        .expect("cannot create schema")
}

pub async fn create_new_prop_string(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
) -> PropString {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    create_new_prop_string_with_name(txn, nats, schema, parent_id, name).await
}

pub async fn create_new_prop_string_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
    name: String,
) -> PropString {
    PropString::new(&txn, &nats, &schema.id, &name, &name, parent_id, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_number(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
) -> PropNumber {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    PropNumber::new(&txn, &nats, &schema.id, &name, &name, None, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_boolean(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
) -> PropBoolean {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    create_new_prop_boolean_with_name(txn, nats, schema, parent_id, name).await
}

pub async fn create_new_prop_boolean_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
    name: String,
) -> PropBoolean {
    PropBoolean::new(&txn, &nats, &schema.id, &name, &name, parent_id, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_map(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
) -> PropMap {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    create_new_prop_map_with_name(&txn, &nats, &schema, parent_id, name).await
}

pub async fn create_new_prop_map_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
    name: String,
) -> PropMap {
    PropMap::new(&txn, &nats, &schema.id, &name, &name, parent_id, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_array(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
) -> PropArray {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    PropArray::new(&txn, &nats, &schema.id, &name, &name, parent_id, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_array_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
    name: String,
) -> PropArray {
    PropArray::new(&txn, &nats, &schema.id, &name, &name, parent_id, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_object(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
) -> PropObject {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    create_new_prop_object_with_name(&txn, &nats, &schema, parent_id, name).await
}

pub async fn create_new_prop_object_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent_id: Option<String>,
    name: String,
) -> PropObject {
    PropObject::new(&txn, &nats, &schema.id, &name, &name, parent_id, false)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_string_with_parent(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    schema: &Schema,
    parent: &Prop,
) -> PropString {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    PropString::new(
        &txn,
        &nats,
        &schema.id,
        &name,
        &name,
        Some(parent.id().to_string()),
        false,
    )
    .await
    .expect("cannot create prop with parent")
}
