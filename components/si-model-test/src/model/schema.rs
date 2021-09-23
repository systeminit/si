use names::{Generator, Name};
use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use si_model::{Prop, PropString, Schema};

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
) -> PropString {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    PropString::new(&txn, &nats, &schema.id, &name, &name, None)
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
