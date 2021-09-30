use crate::{schema::SchemaResult, MinimalStorable, Resolver, ResolverBinding};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};
use std::option::Option::None;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropBoolean {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub is_item: bool,
    pub schema_id: String,
    pub si_storable: MinimalStorable,
}

impl PropBoolean {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        schema_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        parent_id: Option<String>,
        is_item: bool,
    ) -> SchemaResult<Self> {
        let name = name.into();
        let description = description.into();
        let schema_id = schema_id.into();
        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &name,
                    &description,
                    &"boolean",
                    &parent_id,
                    &schema_id,
                    &is_item,
                ],
            )
            .await?;
        let prop_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&prop_json).await?;
        let prop_number: PropBoolean = serde_json::from_value(prop_json)?;

        let unset_resolver = Resolver::find_by_name(&txn, "si:unset").await?;
        let _resolver_binding = ResolverBinding::new(
            &txn,
            &nats,
            &unset_resolver.id,
            crate::resolver::ResolverBackendKindBinding::Unset,
            schema_id.clone(),
            Some(prop_number.id.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(prop_number)
    }
}
