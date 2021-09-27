use crate::{schema::SchemaResult, MinimalStorable, Prop, Resolver, ResolverBinding};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};

// TODO: Maps have one child prop, which is the valueProp from the old days.
// To add an entry to a map, you bind to the valueProp, but must also specify
// the index you want to write the value to. So when the rollup comes around,
// the parent looks at the child to determine what index it should be in, and
// viola! magic.
// You're also allowed to have multiple binding values to maps!
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropMap {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub schema_id: String,
    pub is_item: bool,
    pub si_storable: MinimalStorable,
}

impl PropMap {
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
        //let options_json = serde_json::to_value(options)?;
        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &name,
                    &description,
                    &"map",
                    &parent_id,
                    &schema_id,
                    &is_item,
                ],
            )
            .await?;
        let prop_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&prop_json).await?;
        let prop_map: PropMap = serde_json::from_value(prop_json)?;

        let unset_resolver = Resolver::get_by_name(&txn, "si:unset").await?;
        let _binding = ResolverBinding::new(
            &txn,
            &nats,
            &unset_resolver.id,
            crate::resolver::ResolverBackendKindBinding::Unset,
            schema_id.clone(),
            Some(prop_map.id.clone()),
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(prop_map)
    }
}
