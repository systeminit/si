use crate::{
    data::{Connection, Db},
    models::{
        {insert_model, KeyPairError, ModelError, SiStorable, SiStorableError},
    },
};
use serde::{Deserialize, Serialize};

use strum_macros::{Display, EnumString};
use thiserror::Error;
use tracing::error;

macro_rules! enum_impls {
    ($ty:ty) => {
        impl From<$ty> for String {
            fn from(value: $ty) -> Self {
                value.to_string()
            }
        }

        impl std::convert::TryFrom<&str> for $ty {
            type Error = strum::ParseError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                <Self as std::str::FromStr>::from_str(value)
            }
        }

        impl std::convert::TryFrom<String> for $ty {
            type Error = strum::ParseError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                <Self as std::str::FromStr>::from_str(value.as_str())
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("error when decrypting crypted client")]
    DecryptionFailed,
    #[error("failed to deserialize decrypted message as json: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("error in key pair: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("client is not found")]
    NotFound,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: String,
    pub object_type: ClientObjectType,
    pub kind: ClientKind,
    pub version: ClientVersion,
    pub organization_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: Client,
}

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum ClientObjectType {
    Api,
}

enum_impls!(ClientObjectType);

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum ClientKind {
    Api,
}

enum_impls!(ClientKind);

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum ClientVersion {
    V1,
}

enum_impls!(ClientVersion);

impl Default for ClientVersion {
    fn default() -> Self {
        Self::V1
    }
}

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum ClientAlgorithm {
    Sealedbox,
}

enum_impls!(ClientAlgorithm);

impl Default for ClientAlgorithm {
    fn default() -> Self {
        Self::Sealedbox
    }
}

/// A reference to a database-persisted encrypted client.
///
/// This type does not contain any encypted information nor any encryption metadata and is
/// therefore safe to expose via external API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: String,
    pub name: String,
    pub object_type: ClientObjectType,
    pub kind: ClientKind,
    pub si_storable: SiStorable,
}

impl Client {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        object_type: ClientObjectType,
        kind: ClientKind,
        version: ClientVersion,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> ClientResult<Self> {
        Ok(NewClient::new(
            db,
            nats,
            name,
            object_type,
            kind,
            version,
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?
        .into())
    }
}

/// A database-persisted client.
///
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewClient {
    pub id: String,
    pub name: String,
    pub object_type: ClientObjectType,
    pub kind: ClientKind,
    version: ClientVersion,
    pub si_storable: SiStorable,
}

impl NewClient {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        object_type: ClientObjectType,
        kind: ClientKind,
        version: ClientVersion,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> ClientResult<Self> {
        let name = name.into();

        let si_storable = SiStorable::new(
            db,
            "client",
            billing_account_id,
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;
        let id = si_storable.object_id.clone();
        let model = Self {
            id,
            name,
            object_type,
            kind,
            version,
            si_storable,
        };
        insert_model(db, nats, &model.id, &model).await?;
        Ok(model)
    }
}

impl From<NewClient> for Client {
    fn from(value: NewClient) -> Self {
        Self {
            id: value.id,
            name: value.name,
            object_type: value.object_type,
            kind: value.kind,
            si_storable: value.si_storable,
        }
    }
}
