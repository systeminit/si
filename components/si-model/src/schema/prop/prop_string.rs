use crate::{schema::SchemaResult, MinimalStorable};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PropString {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub si_storable: MinimalStorable,
}

impl PropString {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        schema_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        path: Vec<String>,
    ) -> SchemaResult<Self> {
        let name = name.into();
        let description = description.into();
        let schema_id = schema_id.into();
        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5)",
                &[&name, &description, &"string", &path, &schema_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: PropString = serde_json::from_value(json)?;
        Ok(object)
    }
}
