use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    ActorView,
    HistoryActor,
    HistoryEventMetadata,
};
use si_id::{
    ComponentId,
    PropId,
    SecretId,
};
use thiserror::Error;

use crate::{
    DalContext,
    Secret,
    SecretError,
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretViewError {
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::SiDbError),
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
    /// The list of component Ids connected to the secret [`Secret`].
    pub connected_components: Vec<ComponentId>,
    /// If the secret can be used on this workspace
    pub is_usable: bool,
}

impl SecretView {
    /// Assembles a [`view`](SecretView) for a given [`Secret`].
    pub async fn from_secret(
        ctx: &DalContext,
        secret: Secret,
        prefetched_secret_props: Option<&[PropId]>,
    ) -> SecretViewResult<Self> {
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

        let is_usable = secret.can_be_decrypted(ctx).await?;
        let connected_components = secret
            .clone()
            .find_connected_components(ctx, prefetched_secret_props)
            .await?;

        Ok(Self {
            id: secret.id,
            name: secret.name,
            definition: secret.definition,
            description: secret.description,
            created_info,
            updated_info,
            connected_components,
            is_usable,
        })
    }
}
