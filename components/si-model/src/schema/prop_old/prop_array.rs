use crate::{schema::SchemaResult, MinimalStorable, Resolver, ResolverBinding};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};
use std::collections::HashMap;
use std::option::Option::None;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropArray {
    pub id: String,
    pub name: String,
    pub description: String,
    pub schemas: Vec<String>,
    pub parents: HashMap<String, String>,
    pub si_storable: MinimalStorable,
}

impl PropArray {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        schema_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        parent_id: Option<String>,
    ) -> SchemaResult<Self> {
        let name = name.into();
        let description = description.into();
        let schema_id = schema_id.into();
        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5)",
                &[&name, &description, &"array", &parent_id, &schema_id],
            )
            .await?;
        let prop_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&prop_json).await?;
        let prop_array: PropArray = serde_json::from_value(prop_json)?;

        let unset_resolver = Resolver::find_by_name(&txn, "si:unset").await?;
        let _resolver_binding = ResolverBinding::new(
            &txn,
            &nats,
            &unset_resolver.id,
            crate::resolver::ResolverBackendKindBinding::Unset,
            schema_id.clone(),
            Some(prop_array.id.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(prop_array)
    }
}
