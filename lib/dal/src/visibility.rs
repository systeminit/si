use crate::{ChangeSetPk, DalContext, TransactionsError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use serde_aux::field_attributes::deserialize_number_from_string;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum VisibilityError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type VisibilityResult<T> = Result<T, VisibilityError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Visibility {
    #[serde(
        rename = "visibility_change_set_pk",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub change_set_pk: ChangeSetPk,
    #[serde(rename = "visibility_deleted_at")]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Visibility {
    #[instrument]
    pub fn new(change_set_pk: ChangeSetPk, deleted_at: Option<DateTime<Utc>>) -> Self {
        Visibility {
            change_set_pk,
            deleted_at,
        }
    }

    /// Constructs a new head [`Visibility`].
    #[instrument]
    pub fn new_head(deleted: bool) -> Self {
        let deleted_at = match deleted {
            true => Some(Utc::now()),
            false => None,
        };
        Visibility::new(ChangeSetPk::NONE, deleted_at)
    }

    pub fn to_deleted(&self) -> Self {
        let mut other = *self;
        other.deleted_at = Some(Utc::now());
        other
    }

    pub fn to_non_deleted(&self) -> Self {
        let mut other = *self;
        other.deleted_at = None;
        other
    }

    /// Converts this [`Visibility`] to a new head [`Visibility`].
    pub fn to_head(&self) -> Self {
        Self::new_head(self.deleted_at.is_some())
    }

    /// Determines if this [`Visibility`] is a head [`Visibility`].
    pub fn is_head(&self) -> bool {
        self.change_set_pk == ChangeSetPk::NONE
    }

    /// Constructs a new change set `Visibility`.
    #[instrument]
    pub fn new_change_set(change_set_pk: ChangeSetPk, deleted: bool) -> Self {
        let deleted_at = match deleted {
            true => Some(Utc::now()),
            false => None,
        };
        Visibility::new(change_set_pk, deleted_at)
    }

    /// Converts this `Visibility` to a new change set `Visibility`.
    pub fn to_change_set(&self) -> Self {
        Self::new_change_set(self.change_set_pk, self.deleted_at.is_some())
    }

    /// Returns true if this [`Visibility`] is in a working changeset (and not in head)
    #[instrument]
    pub fn in_change_set(&self) -> bool {
        self.change_set_pk != ChangeSetPk::NONE
    }

    #[instrument(skip(ctx))]
    pub async fn is_visible_to(
        &self,
        ctx: &DalContext,
        check_visibility: &Visibility,
    ) -> VisibilityResult<bool> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT is_visible_v1($1, $2, $3) AS result",
                &[&check_visibility, &self.change_set_pk, &self.deleted_at],
            )
            .await?;
        let result = row.try_get("result")?;
        Ok(result)
    }
}

impl postgres_types::ToSql for Visibility {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}
