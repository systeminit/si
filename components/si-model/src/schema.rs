pub mod prop;

pub use prop::Prop;

use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};
use thiserror::Error;

use crate::{MinimalStorable, Resolver, ResolverBinding, ResolverError};
use std::option::Option::None;

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
}

pub type SchemaResult<T> = Result<T, SchemaError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entity_type: String,
    pub si_storable: MinimalStorable,
}

impl Schema {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        entity_type: impl Into<String>,
        description: impl Into<String>,
    ) -> SchemaResult<Self> {
        let name = name.into();
        let entity_type = entity_type.into();
        let description = description.into();
        let row = txn
            .query_one(
                "SELECT object FROM schema_create_v1($1, $2, $3)",
                &[&name, &entity_type, &description],
            )
            .await?;
        let schema_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&schema_json).await?;
        let schema: Schema = serde_json::from_value(schema_json)?;

        let empty_object_resolver = Resolver::get_by_name(&txn, "si:setEmptyObject").await?;
        let _binding = ResolverBinding::new(
            &txn,
            &nats,
            &empty_object_resolver.id,
            crate::resolver::ResolverBackendKindBinding::EmptyObject,
            schema.id.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(schema)
    }
}
