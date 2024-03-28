use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::history_event::HistoryEventMetadata;
use crate::StandardModelError;
use crate::{ActorView, DalContext, HistoryActor, Secret, SecretId};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretViewError {
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

#[allow(missing_docs)]

pub type SecretViewResult<T> = Result<T, SecretViewError>;

/// A [`view`](SecretView) of a corresponding [`Secret`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretView {
    /// The [`id`](SecretId) of a [`Secret`].
    pub id: SecretId,
    /// The name of a [`Secret`].
    pub name: String,
    /// The definition of a [`Secret`].
    pub definition: String,
    /// The description of a [`Secret`].
    pub description: Option<String>,
    /// The "creation" information for a [`Secret`].
    pub created_info: HistoryEventMetadata,
    /// The "updated" information for a [`Secret`].
    pub updated_info: Option<HistoryEventMetadata>,
}

impl SecretView {
    /// Assembles a [`view`](SecretView) for a given [`Secret`].
    pub async fn from_secret(ctx: &DalContext, secret: Secret) -> SecretViewResult<Self> {
        let created_info = {
            let actor = match secret.created_by {
                None => HistoryActor::SystemInit,
                Some(user_pk) => HistoryActor::from(user_pk),
            };

            let view = ActorView::from_history_actor(ctx, actor).await?;

            HistoryEventMetadata {
                actor: view,
                timestamp: secret.timestamp.created_at,
            }
        };

        let updated_info = {
            let actor = match secret.updated_by {
                None => HistoryActor::SystemInit,
                Some(user_pk) => HistoryActor::from(user_pk),
            };

            let view = ActorView::from_history_actor(ctx, actor).await?;

            if secret.timestamp.created_at == secret.timestamp.updated_at {
                None
            } else {
                Some(HistoryEventMetadata {
                    actor: view,
                    timestamp: secret.timestamp.updated_at,
                })
            }
        };

        Ok(Self {
            id: secret.id,
            name: secret.name,
            definition: secret.definition,
            description: secret.description,
            created_info,
            updated_info,
        })
    }
}
