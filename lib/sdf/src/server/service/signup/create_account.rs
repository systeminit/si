use super::SignupResult;
use crate::server::extract::HandlerContext;
use axum::Json;
use dal::{BillingAccount, HistoryActor, ReadTenancy, WriteTenancy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub billing_account_name: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountResponse {
    pub success: bool,
}

pub async fn create_account(
    HandlerContext(builder, mut txns): HandlerContext,
    Json(request): Json<CreateAccountRequest>,
) -> SignupResult<Json<CreateAccountResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(
        dal::context::AccessBuilder::new(
            ReadTenancy::new_universal(),
            WriteTenancy::new_universal(),
            HistoryActor::SystemInit,
        )
        .build_head(),
        &txns,
    );

    let _result = BillingAccount::signup(
        &ctx,
        &request.billing_account_name,
        &request.user_name,
        &request.user_email,
        &request.user_password,
    )
    .await?;

    txns.commit().await?;
    Ok(Json(CreateAccountResponse { success: true }))
}
