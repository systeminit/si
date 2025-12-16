use axum::response::Json;
use std::collections::HashMap;
use dal::{
    Component,
    ComponentId,
    diagram::Diagram,
    attribute::value::AttributeValue,
    action::Action,
};
use si_id::{ActionId, FuncRunId};
use si_events::ActionState;
use serde::Serialize;
use serde_json::json;
use utoipa::{self, ToSchema};
use telemetry::prelude::*;
use std::time::Instant;

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
    let _start_time = Instant::now();
    info!("🚀 [PERF] Starting component relationships processing...");
    
    let mut relationships = Vec::new();
    
    // Use the Diagram assembly to get all subscription relationships 
    // This is the same method used by the UI to render the visual connections
    let diagram_start = Instant::now();
    let diagram = Diagram::assemble(ctx, None).await?;
    info!("📊 [PERF] Diagram assembly completed in {}ms", diagram_start.elapsed().as_millis());
    
    // Add subscription relationships from the diagram
    let subscription_start = Instant::now();
    let subscription_count = diagram.attribute_subscription_edges.len();
    for edge in &diagram.attribute_subscription_edges {
        let _from_name = Component::name_by_id(ctx, edge.from_component_id).await?;
        let to_name = Component::name_by_id(ctx, edge.to_component_id).await?;
        
        // Try to get the current value being subscribed to
        let current_value = AttributeValue::view(ctx, edge.to_attribute_value_id).await?;
        
        relationships.push((edge.from_component_id, ComponentRelationshipV1 {
            to_component_id: Some(edge.to_component_id),
            to_component_name: to_name,
            relationship_type: RelationshipTypeV1::Subscription,
            from_path: Some(edge.from_attribute_path.clone()),
            to_path: Some(edge.to_attribute_path.clone()),
            execution_status: None,
            current_value,
        }));
    }
    info!("🔗 [PERF] Subscription relationships processed in {}ms (count: {})", 
        subscription_start.elapsed().as_millis(), subscription_count);
    
    // Add management relationships  
    let mgmt_start = Instant::now();
    let component_ids = Component::list_ids(ctx).await?;
    info!("📋 [PERF] Component list fetched in {}ms (count: {})", 
        mgmt_start.elapsed().as_millis(), component_ids.len());
    
    let mgmt_rel_start = Instant::now();
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
    info!("👑 [PERF] Management relationships processed in {}ms", mgmt_rel_start.elapsed().as_millis());

    // Add function relationships with REAL-TIME states (OPTIMIZED)
    if include_functions {
        let functions_start = Instant::now();
        info!("⚙️ [PERF] Starting OPTIMIZED function relationships with real-time states...");
        
        // OPTIMIZATION: Batch all expensive action queries upfront
        let batch_start = Instant::now();
        let all_action_ids = Action::list_topologically(ctx).await?;
        info!("📊 [PERF] Got {} total actions in {}ms", all_action_ids.len(), batch_start.elapsed().as_millis());
        
        // Build super-fast lookup map
        let lookup_start = Instant::now();
        let mut action_lookup: HashMap<(ComponentId, dal::ActionPrototypeId), (ActionState, Option<si_id::FuncRunId>, si_id::ActionId)> = HashMap::new();
        
        for action_id in all_action_ids {
            if let (Ok(Some(comp_id)), Ok(proto_id), Ok(action)) = (
                Action::component_id(ctx, action_id).await,
                Action::prototype_id(ctx, action_id).await,
                Action::get_by_id(ctx, action_id).await
            ) {
                let func_run_id = Action::last_func_run_id_for_id_opt(ctx, action_id).await?;
                action_lookup.insert((comp_id, proto_id), (action.state(), func_run_id, action_id));
            }
        }
        info!("🗂️ [PERF] Built action lookup map in {}ms ({} entries)", 
            lookup_start.elapsed().as_millis(), action_lookup.len());
        
        for component_id in &component_ids {
            let comp_start = Instant::now();
            // Get component functions (1 query per component)
            let (management_functions, action_functions) = super::get_component_functions(ctx, *component_id).await?;
            
            // Add action functions with FAST lookup-based state checking
            for action_func in action_functions {
                let execution_status = if let Some((state, func_run_id, action_id)) = action_lookup.get(&(*component_id, action_func.prototype_id)) {
                    match state {
                        ActionState::Running | ActionState::Dispatched => {
                            Some(FunctionExecutionStatusV1 {
                                state: "Running".to_string(),
                                has_active_run: true,
                                func_run_id: *func_run_id,
                                action_id: Some(*action_id),
                            })
                        }
                        ActionState::Queued => {
                            Some(FunctionExecutionStatusV1 {
                                state: "Queued".to_string(),
                                has_active_run: true,
                                func_run_id: *func_run_id,
                                action_id: Some(*action_id),
                            })
                        }
                        ActionState::OnHold => {
                            Some(FunctionExecutionStatusV1 {
                                state: "OnHold".to_string(),
                                has_active_run: true,
                                func_run_id: *func_run_id,
                                action_id: Some(*action_id),
                            })
                        }
                        ActionState::Failed => {
                            Some(FunctionExecutionStatusV1 {
                                state: "Failed".to_string(),
                                has_active_run: false,
                                func_run_id: *func_run_id,
                                action_id: Some(*action_id),
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
            
            info!("🔧 [PERF] Component {} processed in {}ms", component_id, comp_start.elapsed().as_millis());
            
            // Add management functions
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
            
            // Add qualification functions with REAL status (optimized approach)
            let qual_start = Instant::now();
            
            // Use qualification AVs and get actual qualification status efficiently
            let qualification_avs = Component::list_qualification_avs(ctx, *component_id).await?;
            let qual_count = qualification_avs.len();
            
            for qualification_av in &qualification_avs {
                // Get qualification view with actual status (but just for this specific AV)
                if let Ok(Some(qualification)) = dal::qualification::QualificationView::new(ctx, qualification_av.id()).await {
                    // Get the func run ID efficiently
                    let qual_func_run = ctx
                        .layer_db()
                        .func_run()
                        .get_last_qualification_for_attribute_value_id(
                            ctx.events_tenancy().workspace_pk,
                            qualification_av.id(),
                        )
                        .await?;
                    
                    let func_run_id = qual_func_run.map(|run| run.id());
                    
                    let execution_status = if qualification.finalized {
                        if let Some(result) = &qualification.result {
                            match result.status {
                                dal::qualification::QualificationSubCheckStatus::Success => {
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Succeeded".to_string(),
                                        has_active_run: false,
                                        func_run_id,
                                        action_id: None,
                                    })
                                }
                                dal::qualification::QualificationSubCheckStatus::Failure => {
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Failed".to_string(),
                                        has_active_run: false,
                                        func_run_id,
                                        action_id: None,
                                    })
                                }
                                _ => {
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Completed".to_string(),
                                        has_active_run: false,
                                        func_run_id,
                                        action_id: None,
                                    })
                                }
                            }
                        } else {
                            Some(FunctionExecutionStatusV1 {
                                state: "Completed".to_string(),
                                has_active_run: false,
                                func_run_id,
                                action_id: None,
                            })
                        }
                    } else {
                        Some(FunctionExecutionStatusV1 {
                            state: "Running".to_string(),
                            has_active_run: true,
                            func_run_id,
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
            
            info!("🧪 [PERF] Component {} qualifications processed in {}ms (real status, count: {})", 
                component_id, qual_start.elapsed().as_millis(), qual_count);
        }
        
        info!("⚙️ [PERF] All function relationships completed in {}ms", functions_start.elapsed().as_millis());
    }

    info!("🏁 [PERF] Total component relationships processing completed in {}ms", _start_time.elapsed().as_millis());
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
    let limit = params.limit.unwrap_or(50).min(300) as usize;
    let cursor = params.cursor;

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

    // Handle pagination at the component level
    let component_ids: Vec<String> = sorted_relationships.keys().cloned().collect();
    
    // Find pagination start index
    let start_index = if let Some(ref cursor_str) = cursor {
        component_ids
            .iter()
            .position(|comp_id| comp_id == cursor_str)
            .map(|idx| idx + 1) // Start after the cursor
            .unwrap_or(0)
    } else {
        0
    };
    
    // Calculate end index and paginate components
    let end_index = (start_index + limit).min(component_ids.len());
    let paginated_component_ids = &component_ids[start_index..end_index];
    
    // Build paginated response with only the selected components
    let mut paginated_relationships: BTreeMap<String, Vec<ComponentRelationshipV1>> = BTreeMap::new();
    for component_id in paginated_component_ids {
        if let Some(relationships) = sorted_relationships.remove(component_id) {
            paginated_relationships.insert(component_id.clone(), relationships);
        }
    }
    
    // Generate next cursor (last component ID in current page)
    let next_cursor = if end_index < component_ids.len() && !paginated_component_ids.is_empty() {
        paginated_component_ids.last().cloned()
    } else {
        None
    };

    tracker.track(ctx, "api_component_relationships", json!({}));

    Ok(Json(ComponentRelationshipsV1Response {
        relationships_by_component: paginated_relationships,
        next_cursor,
    }))
}