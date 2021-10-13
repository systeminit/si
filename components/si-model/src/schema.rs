//pub mod prop;
//pub mod prop;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};

use crate::schema_variant::SchemaVariant;
use crate::si_storable::GlobalStorable;
use crate::{Prop, PropVariant, SchemaVariantError, ChangeSetError};
use crate::{Resolver, ResolverBinding, ResolverError};
use crate::change_set_methods;

const SCHEMA_HEAD_BY_NAMESPACE_AND_NAME: &str =
    include_str!("./queries/schema_head_by_namespace_and_name.sql");

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("resolver error: {0}")]
    Resolver(#[from] ResolverError),
    #[error("schema variant error: {0:?}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub description: String,
    pub entity_type: String,
    pub si_storable: GlobalStorable,
}

impl Schema {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        entity_type: impl AsRef<str>,
        description: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
        organization_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> SchemaResult<(Self, SchemaVariant)> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        let entity_type = entity_type.as_ref();
        let description = description.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let organization_id = organization_id.as_ref();
        let workspace_id = workspace_id.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM schema_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &namespace,
                    &name,
                    &entity_type,
                    &description,
                    &billing_account_id,
                    &organization_id,
                    &workspace_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        let schema_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&schema_json).await?;
        let schema: Schema = serde_json::from_value(schema_json)?;
        let default_variant: SchemaVariant = SchemaVariant::new(
            &txn,
            &nats,
            &schema.id,
            "default",
            "default",
            &change_set_id,
            &edit_session_id,
        )
        .await?;

        Ok((schema, default_variant))
    }

    pub async fn new_global(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        entity_type: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> SchemaResult<(Self, SchemaVariant)> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        let entity_type = entity_type.as_ref();
        let description = description.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM schema_create_global_v1($1, $2, $3, $4)",
                &[&namespace, &name, &entity_type, &description],
            )
            .await?;
        let schema_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&schema_json).await?;
        let schema: Schema = serde_json::from_value(schema_json)?;

        let default_variant: SchemaVariant =
            SchemaVariant::new_global(&txn, &nats, &schema.id, "default", "default").await?;

        Ok((schema, default_variant))
    }

    pub async fn find_head_by_namespace_and_name(
        txn: &PgTxn<'_>,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> SchemaResult<Schema> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        let row = txn
            .query_one(SCHEMA_HEAD_BY_NAMESPACE_AND_NAME, &[&namespace, &name])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn find_or_create_global(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        entity_type: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> SchemaResult<Self> {
        let schema = match Schema::find_head_by_namespace_and_name(&txn, &namespace, &name).await {
            Ok(schema) => schema,
            Err(_) => {
                let (schema, _schema_variant) =
                    Schema::new_global(&txn, &nats, &namespace, &name, &entity_type, &description)
                        .await?;
                schema
            }
        };
        Ok(schema)
    }

    change_set_methods!("schemas", SchemaResult<Schema>);
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMap(HashMap<String, PropVariant>);

impl Deref for SchemaMap {
    type Target = HashMap<String, PropVariant>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SchemaMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SchemaMap {
    pub fn new() -> SchemaMap {
        SchemaMap(HashMap::new())
    }

    pub fn find_prop_by_name(
        &self,
        schema_id: impl AsRef<str>,
        parent_id: Option<&str>,
        name: impl AsRef<str>,
    ) -> Option<&PropVariant> {
        let schema_id = schema_id.as_ref();
        let name = name.as_ref();

        // TODO: This should get tis functionality back
        //self.values()
        //    .find(|p| p.parent_id(schema_id) == parent_id && p.name() == name)
        None
    }

    pub fn find_item_prop_for_parent(
        &self,
        schema_id: impl AsRef<str>,
        parent_id: impl AsRef<str>,
    ) -> Option<&PropVariant> {
        let schema_id = schema_id.as_ref();
        let parent_id = parent_id.as_ref();
        None
        //self.values()
        //    .find(|p| p.parent_id(schema_id) == Some(parent_id))
    }
}
