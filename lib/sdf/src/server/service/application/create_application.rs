use super::{ApplicationError, ApplicationResult};
use crate::server::extract::{Authorization, EncryptionKey, NatsTxn, PgRwTxn, Veritech};
use axum::Json;
use dal::{
    Component, HistoryActor, Schema, StandardModel, Tenancy, Visibility, Workspace, WorkspaceId,
    WsEvent, WsPayload, NO_CHANGE_SET_PK,
};
use serde::{Deserialize, Serialize};
use si_data::PgTxn;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    pub name: String,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationResponse {
    pub application: Component,
}

pub async fn create_application(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Veritech(veritech): Veritech,
    EncryptionKey(encryption_key): EncryptionKey,
    Authorization(claim): Authorization,
    Json(request): Json<CreateApplicationRequest>,
) -> ApplicationResult<Json<CreateApplicationResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ApplicationError::InvalidRequest)?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    // You can only create applications directly to head? This feels wrong, but..
    let visibility = Visibility::new_head(false);

    let (application, _application_node) = Component::new_application_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
        &request.name,
    )
    .await?;

    // TODO(fnichol): we're going to create a service component here until we have "node add"
    // functionality in the frontend--then this extra code gets deleted
    create_service_with_node(
        &txn,
        &nats,
        veritech,
        &encryption_key,
        &tenancy,
        &visibility,
        &history_actor,
    )
    .await?;

    // When we create something intentionally on head, we need to fake that a change
    // set has been applied.
    WsEvent::new(
        billing_account_tenancy.billing_account_ids.clone(),
        history_actor.clone(),
        WsPayload::ChangeSetApplied(NO_CHANGE_SET_PK),
    )
    .publish(&nats)
    .await?;

    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(CreateApplicationResponse { application }))
}

async fn create_service_with_node(
    txn: &PgTxn<'_>,
    nats: &si_data::NatsTxn,
    veritech: veritech::Client,
    encryption_key: &veritech::EncryptionKey,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> ApplicationResult<()> {
    let universal_tenancy = Tenancy::new_universal();
    let schema_variant_id =
        Schema::default_schema_variant_id_for_name(txn, &universal_tenancy, visibility, "service")
            .await?;

    let (_component, _node) = Component::new_for_schema_variant_with_node(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        "whiskers",
        &schema_variant_id,
    )
    .await?;

    Ok(())
}
