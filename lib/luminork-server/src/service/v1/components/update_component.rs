use std::collections::HashMap;

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    AttributeValue,
    Component,
    Prop,
    PropId,
    SchemaVariant,
    SchemaVariantId,
    WsEvent,
    attribute::attributes::AttributeSources,
    prop::{
        PROP_PATH_SEPARATOR,
        PropPath,
        PropResult,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentV1RequestPath,
    ComponentViewV1,
    ComponentsResult,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::{
        ComponentsError,
        components::get_component::into_front_end_type,
    },
};

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Update a component",
    request_body = UpdateComponentV1Request,
    responses(
        (status = 200, description = "Component updated successfully", body = UpdateComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 412, description = "Precondition failed - Duplicate component name"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn update_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
    payload: Result<Json<UpdateComponentV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<UpdateComponentV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let component = Component::get_by_id(ctx, component_id).await?;

    let old_name = component.name(ctx).await?;

    if let Some(name) = payload.name {
        component.set_name(ctx, name.as_str()).await?;

        tracker.track(
            ctx,
            "api_component_renamed",
            json!({
                "component_id": component_id,
                "old_name": old_name,
                "new_name": name.clone(),
            }),
        );

        ctx.write_audit_log(
            AuditLogKind::RenameComponent {
                component_id,
                old_name,
                new_name: name.clone(),
            },
            name.clone(),
        )
        .await?;
    }

    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id;
    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, variant_id).await?;

    if !payload.attributes.is_empty() {
        dal::update_attributes(ctx, component_id, payload.attributes.clone()).await?;
    }

    if !is_secret_defining && !payload.secrets.is_empty() {
        return Err(ComponentsError::NotSecretDefiningComponent(component_id));
    }

    // For secrets, if the user needs to attach a secret to the component, the attributes API would require them to know what the secretId
    // is, so we create a convenience method that allows them to attach a secret definition directly by name. Ideally, this would be
    // guarded so that we could only do this on secret defining components.
    if is_secret_defining {
        for (key, value) in payload.secrets.clone().into_iter() {
            let prop_id = key.prop_id(ctx, variant_id).await?;

            let secret_id = resolve_secret_id(ctx, &value).await?;

            let attribute_value_id =
                Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
            dal::Secret::attach_for_attribute_value(ctx, attribute_value_id, Some(secret_id))
                .await?;
        }
    }

    if let Some(resource_id) = payload.resource_id {
        let resource_prop_path = ["root", "si", "resourceId"];
        let resource_prop_id =
            Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(resource_prop_path)).await?;

        let av_for_resource_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), resource_prop_id).await?;

        AttributeValue::update(
            ctx,
            av_for_resource_id,
            Some(serde_json::to_value(resource_id)?),
        )
        .await?;
    }

    // Send a websocket event about the component update
    let updated_component = Component::get_by_id(ctx, component_id).await?;
    let new_name = updated_component.name(ctx).await?;
    WsEvent::component_updated(
        ctx,
        into_front_end_type(ctx, updated_component.clone()).await?,
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    ctx.write_audit_log(
        AuditLogKind::UpdateComponent {
            component_id: updated_component.id(),
            component_name: new_name.clone(),
        },
        new_name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_update_component",
        json!({
            "component_id": component.id(),
            "component_name": new_name.clone(),
        }),
    );

    ctx.commit().await?;

    Ok(Json(UpdateComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentV1Request {
    #[schema(example = "MyUpdatedComponentName")]
    pub name: Option<String>,

    #[schema(example = "i-12345678")]
    pub resource_id: Option<String>,

    #[serde(default)]
    #[schema(example = json!({"secretDefinitionName": "secretId", "secretDefinitionName": "secretName"}))]
    pub secrets: HashMap<SecretPropKey, serde_json::Value>,

    #[serde(default)]
    #[schema(
        value_type = std::collections::BTreeMap<String, serde_json::Value>,
        example = json!({
            "/domain/VpcId": {
                "$source": {
                    "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
                    "path": "/resource_value/VpcId"
                }
            },
            "/domain/SubnetId": {
                "$source": {
                    "component": "01K0WRC69ZPEMD6SMTKC84FBWD",
                    "path": "/resource_value/SubnetId"
                }
            },
            "/domain/Version": "1.2.3",
            "/domain/Version": {
                "$source": null
            }
        })
    )]
    pub attributes: AttributeSources,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentV1Response {
    #[schema(example = json!({
        "id": "01H9ZQD35JPMBGHH69BT0Q79AA",
        "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
        "domainProps": [],
        "resourceProps": [],
        "name": "My EC2 Instance",
        "resourceId": "i-1234567890abcdef0",
        "toDelete": false,
        "canBeUpgraded": true,
        "connections": [],
        "views": [
            {
                "id": "01HAXYZF3GC9CYA6ZVSM3E4YEE",
                "name": "Default View",
                "isDefault": true
            }
        ],
        "sources": [
            ["/domain/RouteTableId", {
                "$source": {
                    "component": "demo-component",
                    "path": "/resource_value/RouteTableId"
                }
            }],
            ["/domain/region", {
                "value": "us-east-1"
            }]
        ]
    }))]
    pub component: ComponentViewV1,
}

/// Resolves a secret value (ID or name) to a SecretId
pub(super) async fn resolve_secret_id(
    ctx: &dal::DalContext,
    value: &serde_json::Value,
) -> ComponentsResult<dal::SecretId> {
    match value {
        serde_json::Value::String(value_str) => {
            if let Ok(id) = value_str.parse() {
                if dal::Secret::get_by_id(ctx, id).await.is_ok() {
                    Ok(id)
                } else {
                    let secrets = dal::Secret::list(ctx).await?;
                    let found_secret = secrets
                        .into_iter()
                        .find(|s| s.name() == value_str)
                        .ok_or_else(|| {
                            ComponentsError::SecretNotFound(format!(
                                "Secret '{value_str}' not found"
                            ))
                        })?;
                    Ok(found_secret.id())
                }
            } else {
                let secrets = dal::Secret::list(ctx).await?;
                let found_secret = secrets
                    .into_iter()
                    .find(|s| s.name() == value_str)
                    .ok_or_else(|| {
                        ComponentsError::SecretNotFound(format!("Secret '{value_str}' not found"))
                    })?;
                Ok(found_secret.id())
            }
        }
        _ => Err(ComponentsError::InvalidSecretValue(format!(
            "Secret value must be a string containing ID or name, got: {value}"
        ))),
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(untagged)]
pub enum SecretPropKey {
    #[schema(value_type = String)]
    PropId(PropId),
    PropPath(SecretPropPath),
}

impl SecretPropKey {
    pub async fn prop_id(
        &self,
        ctx: &dal::DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<PropId> {
        match self {
            SecretPropKey::PropId(prop_id) => Ok(*prop_id),
            SecretPropKey::PropPath(path) => {
                dal::Prop::find_prop_id_by_path(ctx, schema_variant_id, &path.to_prop_path()).await
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct SecretPropPath(pub String);

impl SecretPropPath {
    pub fn to_prop_path(&self) -> PropPath {
        PropPath::new(["root", "secrets"]).join(&self.0.replace("/", PROP_PATH_SEPARATOR).into())
    }
}
