use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, DalContext,
    HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError,
    Visibility,
};

pub mod asset;
pub use asset::*;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InstalledPkgError {
    #[error("error decoding code_base64: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("Installed package asset {0} was expected to be {1} but was {2}")]
    InstalledPkgKindMismatch(
        InstalledPkgAssetId,
        InstalledPkgAssetKind,
        InstalledPkgAssetKind,
    ),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("error decoding ulid: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
}

pub type InstalledPkgResult<T> = Result<T, InstalledPkgError>;

pk!(InstalledPkgPk);
pk!(InstalledPkgId);

/// An `InstalledPkg` is a record of the installation of a package. It tracks the
/// package installation and can be used to prevent duplicate installations and
/// to remove packages after installation.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InstalledPkg {
    pk: InstalledPkgPk,
    id: InstalledPkgId,
    name: String,
    root_hash: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: InstalledPkg,
    pk: InstalledPkgPk,
    id: InstalledPkgId,
    table_name: "installed_pkgs",
    history_event_label_base: "installed_pkg",
    history_event_message_name: "Installed Pkg"
}

impl InstalledPkg {
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        root_hash: impl AsRef<str>,
    ) -> InstalledPkgResult<Self> {
        let name = name.as_ref();
        let root_hash = root_hash.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM installed_pkg_create_v1($1, $2, $3, $4)",
                &[ctx.tenancy(), ctx.visibility(), &name, &root_hash],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, InstalledPkgResult);
    standard_model_accessor!(root_hash, String, InstalledPkgResult);

    pub async fn find_by_hash(ctx: &DalContext, hash: &str) -> InstalledPkgResult<Option<Self>> {
        Ok(Self::find_by_attr(ctx, "root_hash", &hash).await?.pop())
    }
}
