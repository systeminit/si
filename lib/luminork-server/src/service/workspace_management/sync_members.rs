use std::str::FromStr;

use dal::{
    AccessBuilder,
    DalContextBuilder,
    WorkspacePk,
};
use permissions::{
    ObjectType,
    Relation,
    RelationBuilder,
};
use si_db::User;
use si_id::WorkspaceId;

use super::{
    Member,
    WorkspaceManagementError,
    WorkspaceManagementResult,
};
use crate::AppState;

/// Syncs workspace members from auth-api to the DAL and SpiceDB
///
/// This function:
/// 1. Builds a DalContext for the target workspace
/// 2. Syncs approver roles to SpiceDB (if SpiceDB client available)
/// 3. Removes users from DAL who are no longer in auth-api member list
/// 4. This function will only run after we establish that the user has access to the workspace via Auth API
pub(super) async fn sync_members(
    builder: &DalContextBuilder,
    state: &AppState,
    workspace_id: &WorkspaceId,
    validated_token: &sdf_extract::request::ValidatedToken,
    request_ulid: Option<ulid::Ulid>,
    auth_api_members: &[Member],
    tracker: &crate::extract::PosthogEventTracker,
) -> WorkspaceManagementResult<()> {
    let workspace_pk = WorkspacePk::from_str(&workspace_id.to_string())
        .map_err(|e| WorkspaceManagementError::Validation(format!("Invalid workspace_id: {e}")))?;

    let authentication_method = validated_token
        .authentication_method()
        .map_err(|e| WorkspaceManagementError::Validation(format!("Invalid token: {e}")))?;

    let access_builder = AccessBuilder::new(
        workspace_pk.into(),
        validated_token.0.custom.user_id().into(),
        request_ulid,
        authentication_method,
    );

    let ctx = builder.build_head_without_snapshot(access_builder).await?;

    // Sync approvers to SpiceDB if client is available
    if let Some(mut client) = state.spicedb_client_clone() {
        // Extract approver user IDs from auth-api members
        let new_approver_ids: Vec<String> = auth_api_members
            .iter()
            .filter(|m| m.role.eq_ignore_ascii_case("APPROVER"))
            .map(|m| m.user_id.clone())
            .collect();

        let existing_approvers = RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_pk.to_string())
            .relation(Relation::Approver)
            .read(&mut client)
            .await?;

        let existing_approver_ids: Vec<String> = existing_approvers
            .into_iter()
            .map(|w| w.subject().id().to_string())
            .collect();

        let to_add: Vec<String> = new_approver_ids
            .iter()
            .filter(|id| !existing_approver_ids.contains(id))
            .cloned()
            .collect();

        let to_remove: Vec<String> = existing_approver_ids
            .iter()
            .filter(|id| !new_approver_ids.contains(id))
            .cloned()
            .collect();

        // Track the sync operation
        tracker.track(
            &ctx,
            "api_sync_workspace_approvers",
            serde_json::json!({
                "to_add": to_add.clone(),
                "to_remove": to_remove.clone(),
                "existing_approver_ids": existing_approver_ids,
            }),
        );

        for user_id in to_add {
            RelationBuilder::new()
                .object(ObjectType::Workspace, workspace_pk.to_string())
                .relation(Relation::Approver)
                .subject(ObjectType::User, user_id.clone())
                .create(&mut client)
                .await?;

            tracker.track(
                &ctx,
                "api_add_approver",
                serde_json::json!({
                    "user_id": user_id,
                }),
            );
        }

        for user_id in to_remove {
            RelationBuilder::new()
                .object(ObjectType::Workspace, workspace_pk.to_string())
                .relation(Relation::Approver)
                .subject(ObjectType::User, user_id.clone())
                .delete(&mut client)
                .await?;

            tracker.track(
                &ctx,
                "api_remove_approver",
                serde_json::json!({
                    "user_id": user_id,
                }),
            );
        }
    }

    let dal_members = User::list_members_for_workspace(&ctx, workspace_pk.to_string()).await?;

    let auth_api_user_ids: Vec<&str> = auth_api_members
        .iter()
        .map(|m| m.user_id.as_str())
        .collect();

    let users_to_remove: Vec<_> = dal_members
        .into_iter()
        .filter(|u| !auth_api_user_ids.contains(&u.pk().to_string().as_str()))
        .collect();

    for user in users_to_remove {
        User::delete_user_from_workspace(&ctx, user.pk(), workspace_pk.to_string()).await?;
    }

    ctx.commit().await?;

    Ok(())
}
