use crate::change_set_methods;
use crate::si_storable::GlobalStorable;
use crate::{ChangeSetError, Resolver, ResolverBinding, ResolverError};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("resolver error: {0}")]
    Resolver(#[from] ResolverError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariant {
    pub id: String,
    pub schema_id: String,
    pub name: String,
    pub description: String,
    pub root_prop_variant_id: Option<String>,
    pub si_storable: GlobalStorable,
}

impl SchemaVariant {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        schema_id: impl AsRef<str>,
        name: impl AsRef<str>,
        description: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> SchemaVariantResult<Self> {
        let schema_id = schema_id.as_ref();
        let name = name.as_ref();
        let description = description.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM schema_variant_create_v1($1, $2, $3, $4, $5)",
                &[
                    &schema_id,
                    &name,
                    &description,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        let schema_variant_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&schema_variant_json).await?;
        let schema_variant: SchemaVariant = serde_json::from_value(schema_variant_json)?;

        Ok(schema_variant)
    }

    pub async fn new_global(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        schema_id: impl AsRef<str>,
        name: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> SchemaVariantResult<Self> {
        let schema_id = schema_id.as_ref();
        let name = name.as_ref();
        let description = description.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM schema_variant_create_global_v1($1, $2, $3)",
                &[&schema_id, &name, &description],
            )
            .await?;
        let schema_variant_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&schema_variant_json).await?;
        let schema_variant: SchemaVariant = serde_json::from_value(schema_variant_json)?;

        Ok(schema_variant)
    }

    pub async fn set_root_prop_variant_id(
        &mut self,
        txn: &PgTxn<'_>,
        root_prop_variant_id: impl Into<String>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> SchemaVariantResult<()> {
        let root_prop_variant_id = root_prop_variant_id.into();
        self.root_prop_variant_id = Some(root_prop_variant_id.clone());
        self.save_for_edit_session(&txn, &change_set_id, &edit_session_id)
            .await?;
        self.add_prop_variant(&txn, &root_prop_variant_id, &change_set_id, &edit_session_id).await?;
        Ok(())
    }

    pub async fn add_prop_variant(
        &self,
        txn: &PgTxn<'_>,
        prop_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> SchemaVariantResult<()> {
        let prop_variant_id = prop_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let row = txn
            .query_one(
                "SELECT prop_variant_add_to_schema_variant_v1($1, $2, $3, $4)",
                &[
                    &prop_variant_id,
                    &self.id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn remove_prop_variant(
        &self,
        txn: &PgTxn<'_>,
        prop_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> SchemaVariantResult<()> {
        let prop_variant_id = prop_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let row = txn
            .query_one(
                "SELECT prop_variant_remove_from_schema_variant_v1($1, $2, $3, $4)",
                &[
                    &prop_variant_id,
                    &self.id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }


    pub async fn save_for_edit_session(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> SchemaVariantResult<()> {
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let json = serde_json::to_value(self)?;
        let _row = txn
            .query_one(
                "SELECT true FROM schema_variant_save_for_edit_session_v1($1, $2, $3)",
                &[&json, &change_set_id, &edit_session_id],
            )
            .await?;
        Ok(())
    }

    change_set_methods!("schema_variants", SchemaVariantResult<SchemaVariant>);
}
