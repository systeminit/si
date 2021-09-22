use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};
use strum_macros::{Display, IntoStaticStr};
use thiserror::Error;

use crate::{Entity, MinimalStorable, SiStorable};

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid resolver response data; expected String and received {0}")]
    InvalidStringData(serde_json::Value),
}

pub type ResolverResult<T> = Result<T, ResolverError>;

#[derive(Deserialize, Serialize, Debug, Display, IntoStaticStr, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResolverBackendKind {
    String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverBackendKindBinding {
    String(ResolverBackendKindStringBinding),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindStringBinding {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Display, IntoStaticStr, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResolverOutputKind {
    String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resolver {
    pub id: String,
    pub backend: ResolverBackendKind,
    pub name: String,
    pub description: String,
    pub output_kind: ResolverOutputKind,
    pub si_storable: MinimalStorable,
}

impl Resolver {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        description: impl Into<String>,
        backend: ResolverBackendKind,
        output_kind: ResolverOutputKind,
    ) -> ResolverResult<Self> {
        let name = name.into();
        let description = description.into();
        let backend: &str = backend.into();
        let output_kind: &str = output_kind.into();
        let row = txn
            .query_one(
                "SELECT object FROM resolver_create_v1($1, $2, $3, $4)",
                &[&name, &description, &backend, &output_kind],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Resolver = serde_json::from_value(json)?;
        Ok(object)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverArgKind {
    String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverArgKindBinding {
    String(ResolverArgKindBindingString),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverArgKindBindingString {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverArg {
    pub id: String,
    pub name: String,
    pub kind: ResolverArgKind,
    pub description: String,
    pub si_storable: MinimalStorable,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBinding {
    pub id: String,
    pub resolver_id: String,
    pub entity_id: String,
    pub schema_id: Option<String>,
    pub prop_id: Option<String>,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
    pub system_id: Option<String>,
    pub backend_binding: ResolverBackendKindBinding,
    pub si_storable: MinimalStorable,
}

impl ResolverBinding {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        resolver_id: impl Into<String>,
        backend_binding: ResolverBackendKindBinding,
        schema_id: Option<String>,
        prop_id: Option<String>,
        entity_id: Option<String>,
        system_id: Option<String>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
    ) -> ResolverResult<Self> {
        let resolver_id = resolver_id.into();

        let backend_binding = serde_json::to_value(&backend_binding)?;
        let row = txn
            .query_one(
                "SELECT object FROM resolver_binding_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &resolver_id,
                    &schema_id,
                    &prop_id,
                    &entity_id,
                    &backend_binding,
                    &system_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ResolverBinding = serde_json::from_value(json)?;
        Ok(object)
    }

    // Select all the resolver bindings for the schema + entity + context
    //   - Order by the schema root first, then based on prop id path
    // Run the resolver binding for the schema id
    //   - it generates an empty object by default
    //   - or it could return a full object
    //
    // { # schema "fancypants"
    //   foo: "bar"
    // }
    //
    // ResolverBinding schema "fancypants" -> {}
    // ResolverBinding schema "fancypants" prop "foo" ({}) -> { foo: "bar" }
    //
    // { # schema "frobnob"
    //   foo: {
    //      bar: "baz"
    //   }
    // }
    //
    // let mut acc = {};
    // ResolverBinding schema "frobnob" -> {} | acc = {}
    // ResolverBinding schema "frobnob" prop "foo" ({}) -> { foo: {} }  | acc = { foo: {} }
    // ResolverBinding schema "frobnob" prop "bar" path ["foo"] ({}) -> { bar: "baz" } | acc = {
    // foo: { bar: "baz" }
    // return acc
    //
    // { # schema "frobnob"
    //   foo: { }
    // }
    //
    // let mut acc = {};
    // ResolverBinding schema "frobnob" -> {} | acc = {}
    // ResolverBinding schema "frobnob" prop "foo" ({}) -> { foo: {} }  | acc = { foo: {} }
    //   ResolverBinding schema "frobnob" prop "bar" path ["foo"] ({}) -> null | acc = {}
    //   ResolverBinding schema "frobnob" prop "bar" path ["foo"] ({}) -> null | acc = {}
    //   null
    //
    // foo: {}
    // return acc
    //
    //
    //
    //

    pub async fn resolve(&self) -> ResolverResult<serde_json::Value> {
        // Resolve arguments by looking up the ResolverArgBindings
        //
        // Dispatch to the backend
        let result = match &self.backend_binding {
            ResolverBackendKindBinding::String(context) => {
                let result = serde_json::to_value(&context.value)?;
                // You can be damn sure this is a string, really - because
                // the inner type there is a string. But hey - better safe
                // than sorry!
                if !result.is_string() {
                    return Err(ResolverError::InvalidStringData(result));
                }
                result
            }
        };

        Ok(result)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverArgBinding {
    pub id: String,
    pub resolve_id: String,
    pub resolver_binding_id: String,
    pub resolver_arg_id: String,
    pub entity_id: String,
    pub system_id: String,
    pub prop_id: String,
    pub binding: ResolverArgKindBinding,
    pub si_storable: SiStorable,
}

//impl DefaultStringResolver {
//    async fn resolve(obj: serde_json::Value, args: serde_json::Value, context: serde_json::Value) {}
//}
