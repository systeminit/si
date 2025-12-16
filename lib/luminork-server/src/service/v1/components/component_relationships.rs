use axum::response::Json;
use std::collections::HashMap;
use dal::{
    Component,
    ComponentId,
    diagram::Diagram,
    action::Action,
    attribute::value::AttributeValue,
};
use si_id::{ActionId, FuncRunId};
use si_events::ActionState;
use serde::Serialize;
use serde_json::json;
use utoipa::{self, ToSchema};

use super::ComponentsError;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationshipsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub include_functions: Option<bool>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationshipsV1Response {
    #[schema(value_type = std::collections::BTreeMap<String, Vec<ComponentRelationshipV1>>)]
    pub relationships_by_component: std::collections::BTreeMap<String, Vec<ComponentRelationshipV1>>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationshipV1 {
    #[schema(value_type = Option<String>)]
    pub to_component_id: Option<ComponentId>,
    pub to_component_name: String,
    pub relationship_type: RelationshipTypeV1,
    pub from_path: Option<String>,
    pub to_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_status: Option<FunctionExecutionStatusV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionExecutionStatusV1 {
    pub state: String,
    pub has_active_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub func_run_id: Option<si_id::FuncRunId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]  
    pub action_id: Option<si_id::ActionId>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum RelationshipTypeV1 {
    Subscription,
    Manages,
    ActionFunction,
    ManagementFunction,
    QualificationFunction,
}

async fn get_all_component_relationships(
    ctx: &dal::DalContext,
    include_functions: bool,
) -> Result<Vec<(ComponentId, ComponentRelationshipV1)>, ComponentsError> {
    let mut relationships = Vec::new();
    
    // Use the Diagram assembly to get all subscription relationships 
    // This is the same method used by the UI to render the visual connections
    let diagram = Diagram::assemble(ctx, None).await?;
    
    // Add subscription relationships from the diagram
    for edge in diagram.attribute_subscription_edges {
        let _from_name = Component::name_by_id(ctx, edge.from_component_id).await?;
        let to_name = Component::name_by_id(ctx, edge.to_component_id).await?;
        
        // Try to get the current value being subscribed to
        let current_value = AttributeValue::view(ctx, edge.to_attribute_value_id).await?;
        
        relationships.push((edge.from_component_id, ComponentRelationshipV1 {
            to_component_id: Some(edge.to_component_id),
            to_component_name: to_name,
            relationship_type: RelationshipTypeV1::Subscription,
            from_path: Some(edge.from_attribute_path),
            to_path: Some(edge.to_attribute_path),
            execution_status: None,
            current_value,
        }));
    }
    
    // Add management relationships  
    let component_ids = Component::list_ids(ctx).await?;
    for component_id in &component_ids {
        let component = Component::get_by_id(ctx, *component_id).await?;
        let managed_components = component.get_managed(ctx).await?;
        
        for managed_id in managed_components {
            let _from_name = Component::name_by_id(ctx, *component_id).await?;
            let to_name = Component::name_by_id(ctx, managed_id).await?;
            
            relationships.push((*component_id, ComponentRelationshipV1 {
                to_component_id: Some(managed_id),
                to_component_name: to_name,
                relationship_type: RelationshipTypeV1::Manages,
                from_path: None,
                to_path: None,
                execution_status: None,
                current_value: None,
            }));
        }
    }

    // Add function relationships if requested  
    if include_functions {
        // Simple approach: get actions per component as needed
        
        for component_id in &component_ids {
            // Use the existing helper function to get component functions
            let (management_functions, action_functions) = super::get_component_functions(ctx, *component_id).await?;
            
            // Add action function relationships with simplified state checking
            for action_func in action_functions {
                // Quick check: look for any action with this prototype in common states
                let has_queued = Action::find_equivalent(ctx, action_func.prototype_id, Some(*component_id)).await?.is_some();
                
                let execution_status = if has_queued {
                    // If action exists, get its actual state
                    if let Some(action_id) = Action::find_equivalent(ctx, action_func.prototype_id, Some(*component_id)).await? {
                        let action = Action::get_by_id(ctx, action_id).await?;
                        let func_run_id = Action::last_func_run_id_for_id_opt(ctx, action_id).await?;
                        
                        match action.state() {
                            ActionState::Running | ActionState::Dispatched => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Running".to_string(),
                                    has_active_run: true,
                                    func_run_id,
                                    action_id: Some(action_id),
                                })
                            }
                            ActionState::Queued => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Queued".to_string(),
                                    has_active_run: true,
                                    func_run_id,
                                    action_id: Some(action_id),
                                })
                            }
                            ActionState::OnHold => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "OnHold".to_string(),
                                    has_active_run: true,
                                    func_run_id,
                                    action_id: Some(action_id),
                                })
                            }
                            ActionState::Failed => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Failed".to_string(),
                                    has_active_run: false,
                                    func_run_id,
                                    action_id: Some(action_id),
                                })
                            }
                        }
                    } else {
                        Some(FunctionExecutionStatusV1 {
                            state: "Idle".to_string(),
                            has_active_run: false,
                            func_run_id: None,
                            action_id: None,
                        })
                    }
                } else {
                    Some(FunctionExecutionStatusV1 {
                        state: "Idle".to_string(),
                        has_active_run: false,
                        func_run_id: None,
                        action_id: None,
                    })
                };
                
                relationships.push((*component_id, ComponentRelationshipV1 {
                    to_component_id: None,
                    to_component_name: action_func.func_name,
                    relationship_type: RelationshipTypeV1::ActionFunction,
                    from_path: None,
                    to_path: None,
                    execution_status,
                    current_value: None,
                }));
            }
            
            // Add management function relationships
            for mgmt_func in management_functions {
                relationships.push((*component_id, ComponentRelationshipV1 {
                    to_component_id: None,
                    to_component_name: mgmt_func.func_name,
                    relationship_type: RelationshipTypeV1::ManagementFunction,
                    from_path: None,
                    to_path: None,
                    execution_status: Some(FunctionExecutionStatusV1 {
                        state: "Available".to_string(),
                        has_active_run: false,
                        func_run_id: None,
                        action_id: None,
                    }),
                    current_value: None,
                }));
            }
            
            // Add qualification function relationships
            let qualifications = Component::list_qualifications(ctx, *component_id).await?;
            for qualification in qualifications {
                // Simple qualification status
                let execution_status = if qualification.finalized {
                    if let Some(result) = &qualification.result {
                        match result.status {
                            dal::qualification::QualificationSubCheckStatus::Success => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Succeeded".to_string(),
                                    has_active_run: false,
                                    func_run_id: None, // TODO: Get qualification func run ID
                                    action_id: None,
                                })
                            }
                            dal::qualification::QualificationSubCheckStatus::Failure => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Failed".to_string(),
                                    has_active_run: false,
                                    func_run_id: None, // TODO: Get qualification func run ID
                                    action_id: None,
                                })
                            }
                            _ => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Completed".to_string(),
                                    has_active_run: false,
                                    func_run_id: None, // TODO: Get qualification func run ID
                                    action_id: None,
                                })
                            }
                        }
                    } else {
                        Some(FunctionExecutionStatusV1 {
                            state: "Completed".to_string(),
                            has_active_run: false,
                            func_run_id: None, // TODO: Get qualification func run ID
                            action_id: None,
                        })
                    }
                } else {
                    Some(FunctionExecutionStatusV1 {
                        state: "Running".to_string(),
                        has_active_run: true,
                        func_run_id: None, // TODO: Get qualification func run ID
                        action_id: None,
                    })
                };
                
                relationships.push((*component_id, ComponentRelationshipV1 {
                    to_component_id: None,
                    to_component_name: qualification.qualification_name,
                    relationship_type: RelationshipTypeV1::QualificationFunction,
                    from_path: None,
                    to_path: None,
                    execution_status,
                    current_value: None,
                }));
            }
        }
    }

    Ok(relationships)
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/relationships",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination"),
        ("includeFunctions" = Option<bool>, Query, description = "Include function relationships (action and management functions)"),
    ),
    summary = "List all component relationships",
    tag = "components",
    responses(
        (status = 200, description = "Component relationships retrieved successfully", body = ComponentRelationshipsV1Response, example = json!({
                    "relationshipsByComponent": {
                        "01H9ZQD35JPMBGHH69BT0Q79AA": [
                            {
                                "toComponentId": "01H9ZQD35JPMBGHH69BT0Q79BB",
                                "toComponentName": "subnet-component",
                                "relationshipType": "Subscription",
                                "fromPath": "/domain/example",
                                "toPath": "/domain/consume-me"
                            }
                        ],
                        "01H9ZQD35JPMBGHH69BT0Q79EE": [
                            {
                                "toComponentId": null,
                                "toComponentName": "Deploy Function",
                                "relationshipType": "ActionFunction",
                                "executionStatus": {
                                    "state": "Scheduled",
                                    "hasActiveRun": true
                                }
                            },
                            {
                                "toComponentId": null,
                                "toComponentName": "Template Validation",
                                "relationshipType": "QualificationFunction",
                                "executionStatus": {
                                    "state": "Succeeded",
                                    "hasActiveRun": false
                                }
                            }
                        ]
                    },
                    "nextCursor": null
                })),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn component_relationships(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    axum::extract::Query(params): axum::extract::Query<ComponentRelationshipsParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ComponentRelationshipsV1Response>, ComponentsError> {
    // Set default limit and enforce a max limit  
    let _limit = params.limit.unwrap_or(50).min(300) as usize;
    let _cursor = params.cursor;

    // Check if functions should be included
    let include_functions = params.include_functions.unwrap_or(false);
    
    // Get all relationships using the enhanced method
    let all_relationships = get_all_component_relationships(ctx, include_functions).await?;

    // Group relationships by component
    let mut relationships_by_component: HashMap<String, Vec<ComponentRelationshipV1>> = HashMap::new();
    
    for (from_component_id, relationship) in all_relationships {
        let component_id_str = from_component_id.to_string();
        relationships_by_component
            .entry(component_id_str)
            .or_insert_with(Vec::new)
            .push(relationship);
    }

    // Sort relationships within each component group for deterministic ordering
    for relationships in relationships_by_component.values_mut() {
        relationships.sort_by(|a, b| {
            // First sort by relationship type (Subscription, Manages, ActionFunction, etc.)
            match a.relationship_type.cmp(&b.relationship_type) {
                std::cmp::Ordering::Equal => {
                    // Then sort by target component name for consistent ordering
                    a.to_component_name.cmp(&b.to_component_name)
                }
                other => other,
            }
        });
    }
    
    // Convert to a sorted structure for deterministic component ordering
    use std::collections::BTreeMap;
    let mut sorted_relationships: BTreeMap<String, Vec<ComponentRelationshipV1>> = BTreeMap::new();
    for (component_id, relationships) in relationships_by_component {
        sorted_relationships.insert(component_id, relationships);
    }

    // For now, skip complex pagination with grouped data and return all
    // TODO: Implement component-level pagination if needed
    let next_cursor = None;

    tracker.track(ctx, "api_component_relationships", json!({}));

    Ok(Json(ComponentRelationshipsV1Response {
        relationships_by_component: sorted_relationships,
        next_cursor,
    }))
}