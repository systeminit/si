//! An [`ActorView`] is an augmented, display type which represents a user, system, or other actor
//! entitiy which affects a change in the system. Highly related to a [`User`] and [`HistoryActor`]
//! types, this type is able to ship a displayable label suitable for the front end to use when
//! displaying "who did this?"-style changes/updates/mutations.

#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use serde::{Deserialize, Serialize};

use crate::{DalContext, HistoryActor, StandardModelError, User, UserPk};

/// The actor entitiy that initiates an activitiy--this could represent be a person, service, etc.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ActorView {
    /// Represents a human by their [`UserPk`]
    User {
        /// A user's ID
        pk: UserPk,
        /// A display label
        label: String,
    },
    /// Represents a system-generated activity
    System {
        /// A display label
        #[serde(default = "ActorView::system_label")]
        label: String,
    },
}

impl ActorView {
    fn system_label() -> String {
        "system".to_string()
    }

    /// Converts a [`HistoryActor`] and returns an `ActorView`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if a user cannot be determined given a user pk or if there is a aconnection
    /// issue with the database.
    pub async fn from_history_actor(
        ctx: &DalContext,
        history_actor: HistoryActor,
    ) -> Result<Self, StandardModelError> {
        match history_actor {
            HistoryActor::User(user_pk) => {
                let user = User::get_by_pk(ctx, user_pk).await?.ok_or(StandardModelError::UserNotFound(user_pk))?;
                Ok(Self::User {
                    pk: user.pk(),
                    label: user.name().to_string(),
                })
            }
            HistoryActor::SystemInit => Ok(Self::System {
                label: Self::system_label(),
            }),
        }
    }
}

impl postgres_types::ToSql for ActorView {
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
