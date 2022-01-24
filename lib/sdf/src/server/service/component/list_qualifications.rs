use axum::extract::Query;
use axum::Json;
use dal::{
    qualification::{QualificationError, QualificationResult, QualificationView},
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationResponse {
    pub qualification: QualificationView,
}

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
    let _tenancy = Tenancy::new_workspace(vec![*workspace.id()]);
    let qualification = QualificationView {
        message: "I don't think this field needs to exist".to_owned(),
        // If you run the beast catches you, if you stay the beast eats you
        title: Some("Se correr o bicho pega, se ficar o bicho come".to_owned()),
        link: Some("https://www.youtube.com/watch?v=Kw-6mm6Uu7c".to_owned()),
        result: Some(QualificationResult {
            success: false,
            errors: vec![
                QualificationError {
                    message: "🥸".to_owned(),
                },
                QualificationError {
                    message: "Run to the hills".to_owned(),
                },
            ],
        }),
        ..QualificationView::default()
    };
    txn.commit().await?;
    Ok(Json(QualificationResponse { qualification }))
}
