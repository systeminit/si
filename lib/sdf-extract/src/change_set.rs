use axum::{
    RequestPartsExt as _, async_trait,
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use dal::{ChangeSet, ChangeSetId, DalContext};
use derive_more::{Deref, Into};
use serde::Deserialize;

use sdf_core::app_state::AppState;

use super::{ErrorResponse, bad_request, internal_error, workspace::WorkspaceAuthorization};

///
/// Gets a DalContext pointed at the TargetChangeSet.
///
/// This ensures the user is authorized to access the workspace, has the correct role, and
/// that the change set is in fact a part of the workspace.
///
#[derive(Clone, derive_more::Deref, derive_more::Into)]
pub struct ChangeSetDalContext(pub DalContext);

#[async_trait]
impl FromRequestParts<AppState> for ChangeSetDalContext {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Get the workspace we are accessing (and authorized for)
        let WorkspaceAuthorization {
            mut ctx,
            workspace_id,
            ..
        } = parts.extract_with_state(state).await?;

        // Validate the change set id is within that workspace
        let TargetChangeSetId(change_set_id) = parts.extract().await?;
        let change_set = ChangeSet::find(&ctx, change_set_id)
            .await
            .map_err(internal_error)?;
        if change_set.is_none_or(|change_set| change_set.workspace_id != Some(workspace_id)) {
            return Err(internal_error("Change set not found for given workspace"));
        }

        // Update the DalContext to the given changeset.
        ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
            .await
            .map_err(internal_error)?;

        Ok(Self(ctx))
    }
}

/// The target change set id from the path.
///
/// *Not* checked to ensure it is in the workspace.
///
#[derive(Clone, Debug, Deref, Copy, Into)]
struct TargetChangeSetId(pub ChangeSetId);

impl TargetChangeSetId {
    fn set(parts: &mut Parts, change_set_id: ChangeSetId) -> Result<ChangeSetId, ErrorResponse> {
        // This must not be done twice.
        if parts.extensions.get::<TargetChangeSetId>().is_some() {
            return Err(internal_error("Must only specify workspace ID once"));
        }

        parts.extensions.insert(TargetChangeSetId(change_set_id));
        Ok(change_set_id)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for TargetChangeSetId {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(*parts
            .extensions
            .get::<TargetChangeSetId>()
            .ok_or_else(|| internal_error("No changeset ID. Endpoints must call an extractor like TargetChangeSetIdFromPath to get the change set ID."))?)
    }
}

#[derive(Deserialize, Clone, Debug, Deref, Copy, Into)]
pub struct TargetChangeSetIdFromPath {
    change_set_id: ChangeSetId,
}

#[async_trait]
impl<S> FromRequestParts<S> for TargetChangeSetIdFromPath {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Path(TargetChangeSetIdFromPath { change_set_id }) =
            parts.extract().await.map_err(bad_request)?;
        TargetChangeSetId::set(parts, change_set_id)?;
        Ok(TargetChangeSetIdFromPath { change_set_id })
    }
}
