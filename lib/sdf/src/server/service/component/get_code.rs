use axum::extract::Query;
use axum::Json;
use dal::system::UNSET_SYSTEM_ID;
use dal::{
    CodeLanguage, CodeView, Component, ComponentId, StandardModel, SystemId, Tenancy, Visibility,
    Workspace, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeResponse {
    pub code_views: Vec<CodeView>,
}

pub async fn get_code(
    mut txn: PgRoTxn,
    Query(request): Query<GetCodeRequest>,
    Authorization(claim): Authorization,
) -> ComponentResult<Json<GetCodeResponse>> {
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
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let system_id = request.system_id.unwrap_or(UNSET_SYSTEM_ID);
    let code_list = Component::list_code_generated_by_component_id(
        &txn,
        &tenancy,
        &request.visibility,
        request.component_id,
        system_id,
    )
    .await?;

    let code_views: Vec<CodeView> = code_list
        .into_iter()
        .map(|code_gen| CodeView::new(CodeLanguage::Yaml, code_gen.code))
        .collect();

    txn.commit().await?;
    Ok(Json(GetCodeResponse { code_views }))
}
