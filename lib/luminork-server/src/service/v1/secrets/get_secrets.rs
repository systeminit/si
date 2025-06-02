use std::collections::HashMap;

use axum::response::Json;
use dal::{
    Secret,
    SecretDefinitionView,
};
use serde_json::json;
use utoipa::{
    self,
};

use super::{
    SecretsError,
    SecretsResult,
};
use crate::{
    api_types::secrets::v1::{
        SecretDefinitionV1,
        SecretFormDataV1,
        SecretV1,
        SecretViewV1,
    },
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/secrets",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "secrets",
    summary = "List all secrets",
    responses(
        (status = 200, description = "Secrets retrieved successfully", body = GetSecretsV1Response,
          example = json!({
            "aws_credentials": {
              "definition": {
                "secretDefinition": "aws_credentials",
                "formData": [
                  {
                    "name": "access_key_id",
                    "kind": "string"
                  },
                  {
                    "name": "secret_access_key",
                    "kind": "password"
                  },
                  {
                    "name": "region",
                    "kind": "string"
                  },
                  {
                    "name": "default_output",
                    "kind": "string"
                  }
                ]
              },
              "secrets": [
                {
                  "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
                  "name": "Production AWS Key",
                  "definition": "aws_credentials",
                  "description": "AWS credentials for production environment"
                },
                {
                  "id": "01HAXYZF3GC9CYA6ZVSM3E4YHI",
                  "name": "Development AWS Key",
                  "definition": "aws_credentials",
                  "description": "AWS credentials for development environment"
                }
              ]
            },
            "docker_registry": {
              "definition": {
                "secretDefinition": "docker_registry",
                "formData": [
                  {
                    "name": "username",
                    "kind": "string"
                  },
                  {
                    "name": "password",
                    "kind": "password"
                  },
                  {
                    "name": "registry_url",
                    "kind": "string"
                  }
                ]
              },
              "secrets": [
                {
                  "id": "01HAXYZF3GC9CYA6ZVSM3E4YHJ",
                  "name": "DockerHub Access",
                  "definition": "docker_registry",
                  "description": "DockerHub registry credentials"
                }
              ]
            }
          })
        ),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_secrets(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> SecretsResult<Json<GetSecretsV1Response>> {
    tracker.track(ctx, "api_get_secrets", json!({}));

    let definitions = SecretDefinitionView::list(ctx).await?;
    let mut result_map: HashMap<String, SecretViewV1> = definitions
        .iter()
        .map(|def| {
            let secret_definition = def.secret_definition.clone();
            let view = SecretViewV1 {
                definition: create_definition_v1(def),
                secrets: vec![],
            };
            (secret_definition, view)
        })
        .collect();

    let definition_lookup: HashMap<String, &SecretDefinitionView> = definitions
        .iter()
        .map(|def| (def.secret_definition.clone(), def))
        .collect();

    let secrets = Secret::list(ctx).await?;
    for secret in secrets {
        if let Some(view) = result_map.get_mut(secret.definition()) {
            view.secrets.push(create_secret_v1(&secret));
        } else {
            match definition_lookup.get(secret.definition()) {
                Some(def) => {
                    let new_view = SecretViewV1 {
                        definition: create_definition_v1(def),
                        secrets: vec![create_secret_v1(&secret)],
                    };
                    result_map.insert(secret.definition().to_string(), new_view);
                }
                _ => return Err(SecretsError::SecretWithInvalidDefinition(secret.id())),
            }
        }
    }

    Ok(Json(result_map))
}

pub type GetSecretsV1Response = HashMap<String, SecretViewV1>;

fn create_definition_v1(def: &SecretDefinitionView) -> SecretDefinitionV1 {
    let form_data = def
        .form_data
        .iter()
        .map(|fd| SecretFormDataV1 {
            name: fd.name.clone(),
            kind: fd.kind.clone(),
        })
        .collect();

    SecretDefinitionV1 {
        secret_definition: def.secret_definition.clone(),
        form_data,
    }
}

fn create_secret_v1(secret: &Secret) -> SecretV1 {
    SecretV1 {
        id: secret.id(),
        name: secret.name().to_string(),
        definition: secret.definition().to_string(),
        description: secret.description().clone(),
    }
}
