use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    ActionPrototypeId,
    Component,
    ComponentId,
    Func,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
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

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::{
        ComponentV1RequestPath,
        ComponentsError,
    },
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/action",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    request_body = AddActionV1Request,
    summary = "Queue action for a component",
    responses(
        (status = 200, description = "Action successfully queued", body = AddActionV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component or function not found"),
        (status = 409, description = "action already enqueued", body = crate::service::v1::common::ApiError),
        (status = 412, description = "Precondition Failed - View not found or duplicate function name", body = crate::service::v1::common::ApiError),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn add_action(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
    payload: Result<Json<AddActionV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<AddActionV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let action_prototype_id =
        resolve_action_function_reference(ctx, component_id, &payload.action).await?;
    let prototype = ActionPrototype::get_by_id(ctx, action_prototype_id).await?;

    match prototype.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Update | ActionKind::Refresh => {
            let maybe_duplicate = Action::find_for_component_id(ctx, component_id).await?;
            if !maybe_duplicate.is_empty() {
                return Err(ComponentsError::ActionAlreadyEnqueued(prototype.id));
            }
        }
        ActionKind::Manual => {}
    }

    let func_id = ActionPrototype::func_id(ctx, prototype.id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;

    Action::new(ctx, action_prototype_id, component_id.into()).await?;

    tracker.track(
        ctx,
        "api_queue_action",
        json!({
            "component_id": component_id,
            "action_func_name": func.name.clone(),
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::AddAction {
            prototype_id: prototype.id,
            action_kind: prototype.kind.into(),
            func_id,
            func_display_name: func.display_name,
            func_name: func.name.clone(),
        },
        func.name,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(AddActionV1Response { success: true }))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
/// Reference to a management function by either name or ID.
/// This allows clients to use the more human-friendly name approach
/// or the more precise ID approach when working with actions.
// #[schema(example = json!({"function": "Create Asset"}))]
// #[schema(example = json!({"action_prototype_id": "01H9ZQD35JPMBGHH69BT0Q79VY"}))]
pub enum ActionReference {
    ByName {
        #[schema(example = "Create Asset")]
        function: String,
    },
    #[serde(rename_all = "camelCase")]
    ById {
        #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
        action_prototype_id: ActionPrototypeId,
    },
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "action": {"actionPrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VY"}
}))]
#[schema(example = json!({
    "action": {"function": "Create Asset"}
}))]
pub struct AddActionV1Request {
    #[schema(example = json!({"function": "CreateVpc"}), required = true)]
    pub action: ActionReference,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "success": true
}))]
pub struct AddActionV1Response {
    #[schema(example = true)]
    pub success: bool,
}

async fn resolve_action_function_reference(
    ctx: &dal::DalContext,
    component_id: ComponentId,
    function_ref: &ActionReference,
) -> Result<ActionPrototypeId, ComponentsError> {
    match function_ref {
        ActionReference::ById {
            action_prototype_id,
        } => Ok(*action_prototype_id),
        ActionReference::ByName { function } => {
            let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
            let prototypes = ActionPrototype::for_variant(ctx, schema_variant_id).await?;

            let mut matching_prototypes: Vec<_> = vec![];

            for prototype in prototypes {
                let func_id = ActionPrototype::func_id(ctx, prototype.id).await?;
                let func = Func::get_by_id(ctx, func_id).await?;
                if func.name == *function
                    || func
                        .display_name
                        .as_ref()
                        .map_or_else(|| false, |display_name| display_name == function)
                {
                    matching_prototypes.push(prototype);
                }
            }

            match matching_prototypes.len() {
                0 => Err(ComponentsError::ActionFunctionNotFound(function.clone())),
                1 => Ok(matching_prototypes[0].id),
                _ => Err(ComponentsError::DuplicateActionFunctionName(
                    function.clone(),
                )),
            }
        }
    }
}
