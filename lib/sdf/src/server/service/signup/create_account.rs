use axum::Json;
use serde::{Deserialize, Serialize};

use dal::{
    BillingAccount, Component, HistoryActor, NodePosition, ReadTenancy, Schema, SchemaError,
    SchematicKind, StandardModel, WriteTenancy,
};
use telemetry::prelude::*;

use crate::{
    server::extract::{HandlerContext, SignupSecret},
    service::signup::SignupError,
};

use super::SignupResult;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub billing_account_name: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
    pub signup_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountResponse {
    pub success: bool,
}

pub async fn create_account(
    HandlerContext(builder, mut txns): HandlerContext,
    SignupSecret(signup_secret): SignupSecret,
    Json(request): Json<CreateAccountRequest>,
) -> SignupResult<Json<CreateAccountResponse>> {
    if signup_secret.as_str() != request.signup_secret.as_str() {
        warn!("invalid signup secret provided when signing up new billing account");
        return Err(SignupError::InvalidSignupSecret);
    }

    let txns = txns.start().await?;
    let mut ctx = builder.build(
        dal::context::AccessBuilder::new(
            ReadTenancy::new_universal(),
            WriteTenancy::new_universal(),
            HistoryActor::SystemInit,
            None,
        )
        .build_head(),
        &txns,
    );

    let billing_acct = BillingAccount::signup(
        &ctx,
        &request.billing_account_name,
        &request.user_name,
        &request.user_email,
        &request.user_password,
    )
    .await?;

    ctx.update_tenancies(
        ReadTenancy::new_workspace(
            txns.pg(),
            vec![*billing_acct.workspace.id()],
            ctx.visibility(),
        )
        .await?,
        WriteTenancy::new_workspace(*billing_acct.workspace.id()),
    );

    // FIXME(nick,victor): create application and service upon creating a billing account. This is a temporary measure
    // ensure both concepts are removed from the codebase.
    let application_and_service_name = "default";
    let service_schema_name = "service".to_string();

    // Create the application.
    let (_, app_node) =
        Component::new_application_with_node(&ctx, application_and_service_name).await?;
    ctx.update_application_node_id(Some(*app_node.id()));

    // Create the service.
    let schema = Schema::find_by_attr(&ctx, "name", &service_schema_name)
        .await?
        .pop()
        .ok_or(SchemaError::NotFoundByName(service_schema_name))?;
    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or_else(|| SchemaError::NoDefaultVariant(*schema.id()))?;
    let (_, deployment_node) = Component::new_for_schema_variant_with_node(
        &ctx,
        application_and_service_name,
        schema_variant_id,
    )
    .await?;

    // Create the (deployment) node for the service.
    let position_deployment_panel =
        NodePosition::new(&ctx, SchematicKind::Deployment, None, None, "0", "0").await?;
    position_deployment_panel
        .set_node(&ctx, deployment_node.id())
        .await?;
    let position_component_panel = NodePosition::new(
        &ctx,
        SchematicKind::Component,
        None,
        Some(*deployment_node.id()),
        "0",
        "0",
    )
    .await?;

    position_component_panel
        .set_node(&ctx, deployment_node.id())
        .await?;

    txns.commit().await?;
    Ok(Json(CreateAccountResponse { success: true }))
}
