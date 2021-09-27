use crate::{schema::SchemaResult, MinimalStorable, Resolver, ResolverBinding};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropObject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub schema_id: String,
    pub is_item: bool,
    pub si_storable: MinimalStorable,
}

impl PropObject {
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
                    &"object",
                    &parent_id,
                    &schema_id,
                    &is_item,
                ],
            )
            .await?;
        let prop_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&prop_json).await?;
        let prop_object: PropObject = serde_json::from_value(prop_json)?;

        let unset_resolver = Resolver::get_by_name(&txn, "si:unset").await?;
        let _binding = ResolverBinding::new(
            &txn,
            &nats,
            &unset_resolver.id,
            crate::resolver::ResolverBackendKindBinding::Unset,
            schema_id.clone(),
            Some(prop_object.id.clone()),
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(prop_object)
    }
}
