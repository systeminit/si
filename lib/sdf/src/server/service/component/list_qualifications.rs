use axum::extract::Query;
use axum::Json;
use dal::{
    qualification::QualificationView, qualification_resolver::UNSET_ID_VALUE, Component,
    ComponentId, StandardModel, Tenancy, Visibility, Workspace, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQualificationsRequest {
    pub component_id: ComponentId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type QualificationResponse = Vec<QualificationView>;

pub async fn list_qualifications(
    mut txn: PgRoTxn,
    Query(request): Query<ListQualificationsRequest>,
    Authorization(claim): Authorization,
) -> ComponentResult<Json<QualificationResponse>> {
    let txn = txn.start().await?;
    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ComponentError::InvalidRequest)?;

    // This is a "read tenancy" that includes schemas.
    let mut tenancy = Tenancy::new_workspace(vec![*workspace.id()]);
    tenancy.universal = true;

    let qualifications = Component::list_qualifications_by_component_id(
        &txn,
        &tenancy,
        &request.visibility,
        request.component_id,
        UNSET_ID_VALUE.into(),
    )
    .await?;

    txn.commit().await?;
    Ok(Json(qualifications))
}
