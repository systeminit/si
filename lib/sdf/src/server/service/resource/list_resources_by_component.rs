use axum::extract::Query;
use axum::Json;
use dal::{Component, ComponentId, StandardModel, SystemId, Visibility};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::resource::{ResourceError, ResourceResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesByComponentRequest {
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesByComponentResponse {
    components: Vec<MockComponent>,
}

// Mock Data Structs

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MockComponent {
    id: ComponentId,
    name: String,
    schema: String,
    resources: Vec<MockResource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MockResource {
    id: i64,
    name: String,
    kind: String,
    health: MockHealth,
    status: MockStatus,
    confirmations: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
enum MockHealth {
    Ok,
    Warning,
    Error,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
enum MockStatus {
    Pending,
    InProgress,
    Created,
    Failed,
    Deleted,
}

// Mock Data Generation Functions

fn mock_default(id: ComponentId, name: String, schema: String) -> MockComponent {
    // Create confirmation
    let mut confirmations = Vec::new();
    let confirmation = serde_json::json![{
        "title": "fake confirmation",
        "health": "error",
        "link": "none",
        "description": "this is fake",
        "output": [],
    }];
    confirmations.push(confirmation);

    // Create resource
    let mut resources = Vec::new();
    let resouce = MockResource {
        id: 1,
        name: "unknown".to_string(),
        kind: "unknown".to_string(),
        health: MockHealth::Unknown,
        status: MockStatus::Pending,
        confirmations,
    };
    resources.push(resouce);

    // Return component
    MockComponent {
        name,
        id,
        resources,
        schema,
    }
}

fn mock_docker(id: ComponentId, name: String, schema: String) -> MockComponent {
    // Create confirmation
    let mut confirmations = Vec::new();
    let confirmation = serde_json::json![{
        "title": "Does the resource exist?",
        "health": "ok",
        "link": "none",
        "description": "Checks if the resource actually exists.",
        "output": [],
    }];
    confirmations.push(confirmation);

    // Create resource
    let mut resources = Vec::new();
    let resouce = MockResource {
        id: 1,
        name: "whiskers".to_string(),
        kind: "docker image".to_string(),
        health: MockHealth::Ok,
        status: MockStatus::Created,
        confirmations,
    };
    resources.push(resouce);

    // Return component
    MockComponent {
        name,
        id,
        resources,
        schema,
    }
}

pub async fn list_resources_by_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListResourcesByComponentRequest>,
) -> ResourceResult<Json<ListResourcesByComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut components: Vec<MockComponent> = Vec::new();

    for component in Component::list(&ctx).await? {
        let component_id = *component.id();
        let component_name = component
            .find_value_by_json_pointer::<String>(&ctx, "/root/si/name")
            .await?
            .ok_or(ResourceError::ComponentNameNotFound(component_id))?;
        let component_schema = component.schema(&ctx).await?;

        let value = match component_schema {
            Some(schema) => match schema.name() {
                "Docker Image" => {
                    mock_docker(component_id, component_name, schema.name().to_string())
                }
                _ => mock_default(component_id, component_name, schema.name().to_string()),
            },
            None => mock_default(component_id, component_name, "unknown".to_string()),
        };

        components.push(value);
    }

    Ok(Json(ListResourcesByComponentResponse { components }))
}
