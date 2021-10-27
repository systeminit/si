use std::option::Option::None;

use names::{Generator, Name};

use si_data::{NatsTxn, PgTxn};

use crate::prop_variant::PropVariant;
use crate::schema_variant::SchemaVariant;
use crate::test::NewBillingAccount;
use crate::{ChangeSet, EditSession, Prop, PropKind, Schema};

pub async fn create_new_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> (Schema, SchemaVariant) {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    Schema::new(
        &txn,
        &nats,
        "si",
        &name,
        &name,
        &name,
        &change_set.id,
        &edit_session.id,
        &nba.billing_account.id,
        &nba.organization.id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create schema")
}

pub async fn create_new_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    kind: PropKind,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> (Prop, PropVariant) {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    create_new_prop_with_name(&txn, &nats, &name, kind, &nba, &change_set, &edit_session).await
}

pub async fn create_new_prop_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    name: impl AsRef<str>,
    kind: PropKind,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> (Prop, PropVariant) {
    let name = name.as_ref();
    Prop::new(
        &txn,
        &nats,
        "si",
        &name,
        &name,
        kind,
        &change_set.id,
        &edit_session.id,
        &nba.billing_account.id,
        &nba.organization.id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create prop")
}

// pub async fn create_new_prop_string(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
// ) -> PropString {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     create_new_prop_string_with_name(txn, nats, schema, parent_id, name).await
// }
//
// pub async fn create_new_prop_string_with_name(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
//     name: String,
// ) -> PropString {
//     PropString::new(&txn, &nats, &schema.id, &name, &name, parent_id)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_number(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
// ) -> PropNumber {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     PropNumber::new(&txn, &nats, &schema.id, &name, &name, None)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_boolean(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
// ) -> PropBoolean {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     create_new_prop_boolean_with_name(txn, nats, schema, parent_id, name).await
// }
//
// pub async fn create_new_prop_boolean_with_name(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
//     name: String,
// ) -> PropBoolean {
//     PropBoolean::new(&txn, &nats, &schema.id, &name, &name, parent_id)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_map(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
// ) -> PropMap {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     create_new_prop_map_with_name(&txn, &nats, &schema, parent_id, name).await
// }
//
// pub async fn create_new_prop_map_with_name(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
//     name: String,
// ) -> PropMap {
//     PropMap::new(&txn, &nats, &schema.id, &name, &name, parent_id)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_array(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
// ) -> PropArray {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     PropArray::new(&txn, &nats, &schema.id, &name, &name, parent_id)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_array_with_name(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
//     name: String,
// ) -> PropArray {
//     PropArray::new(&txn, &nats, &schema.id, &name, &name, parent_id)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_object(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
// ) -> PropObject {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     create_new_prop_object_with_name(&txn, &nats, &schema, parent_id, name).await
// }
//
// pub async fn create_new_prop_object_with_name(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent_id: Option<String>,
//     name: String,
// ) -> PropObject {
//     PropObject::new(&txn, &nats, &schema.id, &name, &name, parent_id)
//         .await
//         .expect("cannot create prop")
// }
//
// pub async fn create_new_prop_string_with_parent(
//     txn: &PgTxn<'_>,
//     nats: &NatsTxn,
//     schema: &Schema,
//     parent: &Prop,
// ) -> PropString {
//     let mut generator = Generator::with_naming(Name::Numbered);
//     let name = generator.next().unwrap();
//     PropString::new(
//         &txn,
//         &nats,
//         &schema.id,
//         &name,
//         &name,
//         Some(parent.id().to_string()),
//     )
//     .await
//     .expect("cannot create prop with parent")
// }
//
