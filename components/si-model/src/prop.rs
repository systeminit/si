use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};

use crate::si_storable::GlobalStorable;
use crate::{PropVariant, PropVariantError, ChangeSetError};
use crate::change_set_methods;

#[derive(Error, Debug)]
pub enum PropError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("prop variant error: {0}")]
    PropVariant(#[from] PropVariantError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type PropResult<T> = Result<T, PropError>;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PropKind {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Map,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Prop {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub description: String,
    pub kind: PropKind,
    pub si_storable: GlobalStorable,
}

impl Prop {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        description: impl AsRef<str>,
        kind: PropKind,
        change_set_id: impl AsRef<str>,
        edit_session_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
        organization_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> PropResult<(Self, PropVariant)> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        let description = description.as_ref();
        let change_set_id = change_set_id.as_ref();
        let edit_session_id = edit_session_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let organization_id = organization_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let kind_string = kind.to_string();

        let row = txn
            .query_one(
                "SELECT object FROM prop_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &namespace,
                    &name,
                    &description,
                    &kind_string,
                    &billing_account_id,
                    &organization_id,
                    &workspace_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        let prop_json: serde_json::Value = row.try_get("object")?;
        nats.publish(&prop_json).await?;
        let prop: Prop = serde_json::from_value(prop_json)?;
        let default_variant = PropVariant::new(
            &txn,
            &nats,
            &prop.id,
            "default",
            "default",
            &change_set_id,
            &edit_session_id,
        )
        .await?;

        Ok((prop, default_variant))
    }

    change_set_methods!("props", PropResult<Prop>);
}
