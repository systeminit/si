use axum::{
    RequestPartsExt as _,
    async_trait,
    extract::{
        FromRequestParts,
        Path,
    },
    http::request::Parts,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    DalContext,
    WorkspacePk,
};
use derive_more::{
    Deref,
    Into,
};
use sdf_core::app_state::AppState;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::User;
use si_jwt_public_key::SiJwtClaimRole;

use super::{
    ErrorResponse,
    bad_request,
    internal_error,
    workspace::WorkspaceAuthorization,
};

///
/// Gets a DalContext pointed at the TargetChangeSet, with the snapshot preloaded.
///
/// - Authenticates the user via token (via ChangeSetAuthorization)
/// - Authorizes the user to the endpoint (role), workspace, and change set (via ChangeSetAuthorization)
/// - Loads the snapshot
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
        let ChangeSetAuthorization {
            mut ctx_without_snapshot,
            ..
        } = parts.extract_with_state(state).await?;

        ctx_without_snapshot
            .update_snapshot_to_visibility()
            .await
            .map_err(internal_error)?;

        Ok(Self(ctx_without_snapshot))
    }
}

///
/// Handles the whole endpoint authorization (checking if the user has access to the target
/// workspace with the desired role, *and* that the user is a member of the workspace), and
/// checks that the TargetChangeSetIdent is within the given workspace.
///
/// - Authenticates the user via token (via WorkspaceAuthorization)
/// - Authorizes the user to the endpoint (role) and workspace (via WorkspaceAuthorization)
/// - Validates that the change set is in the workspace
///
/// This extractor is cached and may be called multiple times without redoing the work.
///
#[derive(Clone)]
pub struct ChangeSetAuthorization {
    /// The DalContext used to talk to the DB. This has the correct workspace and changeset,
    /// but does NOT have the snapshot loaded. ChangeSetDalContext gives you that.
    pub ctx_without_snapshot: DalContext,
    pub user: User,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub authorized_role: SiJwtClaimRole,
}

#[async_trait]
impl FromRequestParts<AppState> for ChangeSetAuthorization {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(result) = parts.extensions.get::<Self>() {
            return Ok(result.clone());
        }

        // Get the workspace we are accessing (and authorized for)
        let WorkspaceAuthorization {
            mut ctx_without_snapshot,
            user,
            workspace_id,
            authorized_role,
        } = parts.extract_with_state(state).await?;

        // Validate the change set id is within that workspace
        let TargetChangeSetIdent(change_set_ident) = parts.extract().await?;
        let change_set_id = change_set_ident.resolve(&ctx_without_snapshot).await?;
        let change_set = ChangeSet::find(&ctx_without_snapshot, change_set_id)
            .await
            .map_err(internal_error)?;
        if change_set.is_none_or(|change_set| change_set.workspace_id != Some(workspace_id)) {
            return Err(internal_error("Change set not found for given workspace"));
        }

        // Update the DalContext to the given changeset, but do not load the snapshot.
        // As long as we *do* expose a DalContext, we should make sure it has the right visibility,
        // because callers look at it.
        ctx_without_snapshot.update_visibility_deprecated(change_set_id.into());

        Ok(Self {
            ctx_without_snapshot,
            user,
            workspace_id,
            change_set_id,
            authorized_role,
        })
    }
}

/// The target change set id from the path.
///
/// *Not* checked to ensure it is in the workspace.
///
#[derive(Clone, Debug, Deref, Into)]
struct TargetChangeSetIdent(pub ChangeSetIdent);

impl TargetChangeSetIdent {
    fn set(
        parts: &mut Parts,
        change_set_ident: ChangeSetIdent,
    ) -> Result<ChangeSetIdent, ErrorResponse> {
        // This must not be done twice.
        if parts.extensions.get::<TargetChangeSetIdent>().is_some() {
            return Err(internal_error("Must only specify workspace ID once"));
        }

        parts
            .extensions
            .insert(TargetChangeSetIdent(change_set_ident.clone()));
        Ok(change_set_ident)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for TargetChangeSetIdent {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<TargetChangeSetIdent>()
            .ok_or_else(|| internal_error("No changeset ID. Endpoints must call an extractor like TargetChangeSetIdentFromPath to get the change set ID."))?
            .clone())
    }
}

#[derive(Deserialize, Clone, Debug, Deref, Into)]
pub struct TargetChangeSetIdentFromPath {
    change_set_id: ChangeSetIdent,
}

#[async_trait]
impl<S> FromRequestParts<S> for TargetChangeSetIdentFromPath {
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Path(TargetChangeSetIdentFromPath { change_set_id }) =
            parts.extract().await.map_err(bad_request)?;
        TargetChangeSetIdent::set(parts, change_set_id.clone())?;
        Ok(TargetChangeSetIdentFromPath { change_set_id })
    }
}

/// String identifier for a changeset within a workspace.
/// Supports either ChangeSetId (ULID) or a "HEAD" (case-insensitive).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetIdent(String);

impl From<ChangeSetId> for ChangeSetIdent {
    fn from(id: ChangeSetId) -> Self {
        Self(id.to_string())
    }
}

impl ChangeSetIdent {
    /// Create a ChangeSetIdent with the string "HEAD"
    pub fn head() -> Self {
        Self("HEAD".to_string())
    }

    /// Get the ChangeSetId for this ChangeSetIdent. If it is HEAD, it will get the HEAD
    /// changeset from the workspace.
    pub async fn resolve(&self, ctx: &DalContext) -> Result<ChangeSetId, ErrorResponse> {
        if self.0.eq_ignore_ascii_case("HEAD") {
            ctx.get_workspace_default_change_set_id()
                .await
                .map_err(internal_error)
        } else {
            self.0.parse().map_err(bad_request)
        }
    }
}
