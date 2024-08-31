use std::collections::HashMap;

use axum::{extract::Query, Json};
use dal::{Secret, SecretDefinitionView, SecretView, Visibility};
use serde::{Deserialize, Serialize};

use super::SecretResult;
use crate::{
    extract::{AccessBuilder, HandlerContext},
    service::secret::SecretError,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SecretDefinitionViewWithSecrets {
    pub definition: SecretDefinitionView,
    pub secrets: Vec<SecretView>,
}

pub type ListSecretResponse = HashMap<String, SecretDefinitionViewWithSecrets>;

pub async fn list_secrets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSecretRequest>,
) -> SecretResult<Json<ListSecretResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let definitions = SecretDefinitionView::list(&ctx).await?;

    let mut hash_map = definitions
        .into_iter()
        .map(|def| {
            let secret_definition = def.secret_definition.clone();

            let view = SecretDefinitionViewWithSecrets {
                definition: def,
                secrets: vec![],
            };

            (secret_definition, view)
        })
        .collect::<HashMap<_, _>>();

    for secret in Secret::list(&ctx).await? {
        hash_map
            .get_mut(secret.definition())
            .ok_or(SecretError::SecretWithInvalidDefinition(secret.id()))?
            .secrets
            .push(SecretView::from_secret(&ctx, secret).await?);
    }

    Ok(Json(hash_map))
}
