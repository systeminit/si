use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};
use thiserror::Error;

use crate::si_storable::GlobalStorable;
use crate::{change_set_methods, SchemaVariant};
use crate::{ChangeSetError, PropKind, Resolver, ResolverBinding, ResolverError};

#[derive(Error, Debug)]
pub enum PropVariantError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("resolver error: {0}")]
    ResolverError(#[from] ResolverError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type PropVariantResult<T> = Result<T, PropVariantError>;

// TODO: This is still fucked up
const PROP_VARIANT_BY_ID: &str = include_str!("./queries/prop_by_id.sql");
const PROP_VARIANTS_GET_SCHEMA_VARIANTS: &str =
    include_str!("./queries/prop_variants_get_schema_variants.sql");
const PROP_VARIANT_PARENT: &str = include_str!("./queries/prop_variant_parent.sql");
const PROP_VARIANT_DESCENDANTS: &str = include_str!("./queries/prop_variant_descendants.sql");

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PropVariant {
    pub id: String,
    pub prop_id: String,
    pub name: String,
    pub description: String,
    pub kind: PropKind,
    pub si_storable: GlobalStorable,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PropLineage {
    pub id: String,
    pub parent_prop_variant_id: String,
    pub prop_variant: PropVariant,
    pub depth: u64,
    pub cycle: bool,
}

impl PropVariant {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        prop_id: impl AsRef<str>,
        name: impl AsRef<str>,
        description: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<Self> {
        let prop_id = prop_id.as_ref();
        let name = name.as_ref();
        let description = description.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let row = txn
            .query_one(
                "SELECT object FROM prop_variant_create_v1($1, $2, $3, $4, $5)",
                &[
                    &prop_id,
                    &name,
                    &description,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        let prop_variant_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&prop_variant_json).await?;
        let prop_variant: PropVariant = serde_json::from_value(prop_variant_json)?;

        let unset_resolver = Resolver::find_by_name(&txn, "si:unset").await?;
        let resolver_binding = ResolverBinding::new(
            &txn,
            &nats,
            &unset_resolver.id,
            crate::resolver::ResolverBackendKindBinding::EmptyObject,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
        resolver_binding.resolve(&txn, &nats).await?;

        Ok(prop_variant)
    }

    pub async fn get_by_id(txn: &PgTxn<'_>, id: impl AsRef<str>) -> PropVariantResult<Self> {
        let id = id.as_ref();
        let row = txn.query_one(PROP_VARIANT_BY_ID, &[&id]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn add_to_schema_variant(
        &self,
        txn: &PgTxn<'_>,
        schema_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<()> {
        let schema_variant_id = schema_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let row = txn
            .query_one(
                "SELECT prop_variant_add_to_schema_variant_v1($1, $2, $3, $4)",
                &[
                    &self.id,
                    &schema_variant_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn remove_from_schema_variant(
        &self,
        txn: &PgTxn<'_>,
        schema_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<()> {
        let schema_variant_id = schema_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let row = txn
            .query_one(
                "SELECT prop_variant_remove_from_schema_variant_v1($1, $2, $3, $4)",
                &[
                    &self.id,
                    &schema_variant_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn schema_variants(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> PropVariantResult<Vec<SchemaVariant>> {
        let mut results: Vec<SchemaVariant> = Vec::new();

        let rows = txn
            .query(
                PROP_VARIANTS_GET_SCHEMA_VARIANTS,
                &[&self.id, &change_set_id, &edit_session_id],
            )
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object: SchemaVariant = serde_json::from_value(json)?;
            results.push(object);
        }

        Ok(results)
    }

    pub async fn add_child(
        &self,
        txn: &PgTxn<'_>,
        child_prop_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<()> {
        let child_prop_variant_id = child_prop_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let _row = txn
            .query_one(
                "SELECT prop_variant_add_parent_v1($1, $2, $3, $4)",
                &[
                    &child_prop_variant_id,
                    &self.id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn remove_child(
        &self,
        txn: &PgTxn<'_>,
        child_prop_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<()> {
        let child_prop_variant_id = child_prop_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let _row = txn
            .query_one(
                "SELECT prop_variant_remove_parent_v1($1, $2, $3, $4)",
                &[
                    &child_prop_variant_id,
                    &self.id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn add_parent(
        &self,
        txn: &PgTxn<'_>,
        parent_prop_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<()> {
        let parent_prop_variant_id = parent_prop_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let _row = txn
            .query_one(
                "SELECT prop_variant_add_parent_v1($1, $2, $3, $4)",
                &[
                    &self.id,
                    &parent_prop_variant_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn remove_parent(
        &self,
        txn: &PgTxn<'_>,
        parent_prop_variant_id: impl AsRef<str>,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
    ) -> PropVariantResult<()> {
        let parent_prop_variant_id = parent_prop_variant_id.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();

        let _row = txn
            .query_one(
                "SELECT prop_variant_remove_parent_v1($1, $2, $3, $4)",
                &[
                    &self.id,
                    &parent_prop_variant_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn parents(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> PropVariantResult<Vec<PropVariant>> {
        let mut parents = Vec::new();
        let rows = txn
            .query(
                PROP_VARIANT_PARENT,
                &[
                    &self.id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let parent: PropVariant  = serde_json::from_value(json)?;
            parents.push(parent);
        }

        Ok(parents)
    }

    // TODO: Finish implementing the descendants
    pub async fn descendants(
        &self,
        txn: &PgTxn<'_>,
        change_set_id: Option<&String>,
        edit_session_id: Option<&String>,
    ) -> PropVariantResult<Vec<PropLineage>> {
        let mut descendants = Vec::new();
        let rows = txn
            .query(
                PROP_VARIANT_DESCENDANTS,
                &[
                    &self.id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        for row in rows.into_iter() {
            //let json: serde_json::Value = row.into();
            //let json: serde_json::Value = row.try_get("object")?;
            dbg!(row);
        }

        Ok(descendants)
    }

    change_set_methods!("prop_variants", PropVariantResult<PropVariant>);
}
