use serde::{Deserialize, Serialize};
use si_data::{PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{ChangeSetPk, EditSessionPk, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK};
use serde_aux::field_attributes::{deserialize_bool_from_anything, deserialize_number_from_string};

#[derive(Error, Debug)]
pub enum VisibilityError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type VisibilityResult<T> = Result<T, VisibilityError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Visibility {
    #[serde(
        rename = "visibility_change_set_pk",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub change_set_pk: ChangeSetPk,
    #[serde(
        rename = "visibility_edit_session_pk",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub edit_session_pk: EditSessionPk,
    #[serde(
        rename = "visibility_deleted",
        deserialize_with = "deserialize_bool_from_anything"
    )]
    pub deleted: bool,
}

impl Visibility {
    #[instrument]
    pub fn new(change_set_pk: ChangeSetPk, edit_session_pk: EditSessionPk, deleted: bool) -> Self {
        Visibility {
            change_set_pk,
            edit_session_pk,
            deleted,
        }
    }

    /// Constructs a new head `Visibility`.
    #[instrument]
    pub fn new_head(deleted: bool) -> Self {
        Visibility::new(NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK, deleted)
    }

    /// Converts this `Visibility` to a new head `Visibility`.
    pub fn to_head(&self) -> Self {
        Self::new_head(self.deleted)
    }

    /// Constructs a new change set `Visibility`.
    #[instrument]
    pub fn new_change_set(change_set_pk: ChangeSetPk, deleted: bool) -> Self {
        Visibility::new(change_set_pk, NO_EDIT_SESSION_PK, deleted)
    }

    /// Converts this `Visibility` to a new change set `Visibility`.
    pub fn to_change_set(&self) -> Self {
        Self::new_change_set(self.change_set_pk, self.deleted)
    }

    /// Constructs a new edit session `Visibility`.
    #[instrument]
    pub fn new_edit_session(
        change_set_pk: ChangeSetPk,
        edit_session_pk: EditSessionPk,
        deleted: bool,
    ) -> Self {
        Visibility::new(change_set_pk, edit_session_pk, deleted)
    }

    #[instrument]
    pub fn in_edit_session(&self) -> bool {
        self.edit_session_pk != NO_EDIT_SESSION_PK
    }

    #[instrument]
    pub fn in_change_set(&self) -> bool {
        self.change_set_pk != NO_CHANGE_SET_PK
    }

    #[instrument(skip(txn))]
    pub async fn is_visible_to(
        &self,
        txn: &PgTxn<'_>,
        check_visibility: &Visibility,
    ) -> VisibilityResult<bool> {
        let row = txn
            .query_one(
                "SELECT result FROM is_visible_v1($1, $2, $3, $4)",
                &[
                    &check_visibility,
                    &self.change_set_pk,
                    &self.edit_session_pk,
                    &self.deleted,
                ],
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
