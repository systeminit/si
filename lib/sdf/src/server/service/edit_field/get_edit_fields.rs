use axum::extract::Query;
use super::EditFieldResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use axum::Json;
use chrono::Utc;
use dal::{EditSession, HistoryActor, Tenancy, edit_field::{EditFieldObjectKind, EditFields}, Visibility, Schema, schema};
use serde::{Deserialize, Serialize};
use dal::edit_field::EditFieldAble;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEditFieldsRequest {
    pub object_kind: EditFieldObjectKind,
    pub id: i64,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEditFieldsResponse {
    pub fields: EditFields,
}

pub async fn get_edit_fields(
    mut txn: PgRwTxn,
    Authorization(claim): Authorization,
    Query(request): Query<GetEditFieldsRequest>,
) -> EditFieldResult<Json<GetEditFieldsResponse>> {
    let txn = txn.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);

    let edit_fields = match request.object_kind {
        EditFieldObjectKind::Schema => {
            Schema::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into()).await?
        }
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into()).await?
        }

    };

    txn.commit().await?;

    Ok(Json(GetEditFieldsResponse { fields: edit_fields }))
}
