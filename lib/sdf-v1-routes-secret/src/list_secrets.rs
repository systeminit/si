use std::collections::HashMap;

use axum::{Json, extract::Query};
use dal::{Secret, SecretDefinitionView, SecretView, Visibility};
use serde::{Deserialize, Serialize};

use crate::{SecretError, SecretResult};
use sdf_extract::{HandlerContext, v1::AccessBuilder};

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

    let prefetched_secret_props = Secret::list_all_secret_prop_ids(&ctx).await?;
    for secret in Secret::list(&ctx).await? {
        hash_map
            .get_mut(secret.definition())
            .ok_or(SecretError::SecretWithInvalidDefinition(secret.id()))?
            .secrets
            .push(
                SecretView::from_secret(&ctx, secret, Some(prefetched_secret_props.as_slice()))
                    .await?,
            );
    }

    Ok(Json(hash_map))
}
