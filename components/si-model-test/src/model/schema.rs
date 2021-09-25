use names::{Generator, Name};
use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use si_model::schema::prop::PropArray;
use si_model::{Prop, PropBoolean, PropMap, PropNumber, PropObject, PropString, Schema};

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
    PropString::new(&txn, &nats, &schema.id, &name, &name, parent_id)
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
    PropNumber::new(&txn, &nats, &schema.id, &name, &name, None)
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
    PropBoolean::new(&txn, &nats, &schema.id, &name, &name, parent_id)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_map(txn: &PgTxn<'_>, nats: &NatsTxn, schema: &Schema) -> PropMap {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    PropMap::new(&txn, &nats, &schema.id, &name, &name, None)
        .await
        .expect("cannot create prop")
}

pub async fn create_new_prop_array(txn: &PgTxn<'_>, nats: &NatsTxn, schema: &Schema) -> PropArray {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    PropArray::new(&txn, &nats, &schema.id, &name, &name, None)
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
    PropObject::new(&txn, &nats, &schema.id, &name, &name, parent_id)
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
    )
    .await
    .expect("cannot create prop with parent")
}
