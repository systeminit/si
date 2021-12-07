use axum::Json;
use dal::{
    edit_field::{EditFieldAble, EditFieldObjectKind},
    schema::{self, SchemaVariant},
    socket::Socket,
    HistoryActor, Schema, Tenancy, Visibility,
};
use serde::{Deserialize, Serialize};

use super::EditFieldResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFromEditFieldRequest {
    pub object_kind: EditFieldObjectKind,
    pub object_id: i64,
    pub edit_field_id: String,
    pub value: Option<serde_json::Value>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFromEditFieldResponse {
    pub success: bool,
}

pub async fn update_from_edit_field(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<UpdateFromEditFieldRequest>,
) -> EditFieldResult<Json<UpdateFromEditFieldResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let mut tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    tenancy.universal = true;
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);

    match request.object_kind {
        EditFieldObjectKind::Schema => {
            Schema::update_from_edit_field(
                &txn,
                &nats,
                &tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::update_from_edit_field(
                &txn,
                &nats,
                &tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::SchemaVariant => {
            SchemaVariant::update_from_edit_field(
                &txn,
                &nats,
                &tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::Socket => {
            Socket::update_from_edit_field(
                &txn,
                &nats,
                &tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
    };

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(UpdateFromEditFieldResponse { success: true }))
}
