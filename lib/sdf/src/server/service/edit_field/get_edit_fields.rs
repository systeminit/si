use axum::extract::Query;
use axum::Json;
use dal::{
    edit_field::{EditFieldAble, EditFieldObjectKind, EditFields},
    schema,
    socket::Socket,
    Component, Prop, QualificationCheck, Schema, Tenancy, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::EditFieldResult;
use crate::server::extract::{Authorization, PgRwTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEditFieldsRequest {
    pub object_kind: EditFieldObjectKind,
    pub id: i64,
    pub workspace_id: Option<WorkspaceId>,
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

    let mut tenancy = match request.workspace_id {
        Some(workspace_id) => Tenancy::new_workspace(vec![workspace_id]),
        None => Tenancy::new_billing_account(vec![claim.billing_account_id]),
    };
    tenancy.universal = true;

    let edit_fields = match request.object_kind {
        EditFieldObjectKind::Component => {
            Component::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into())
                .await?
        }
        EditFieldObjectKind::ComponentProp => {
            Component::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into())
                .await?
        }
        EditFieldObjectKind::Prop => {
            Prop::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into()).await?
        }
        EditFieldObjectKind::QualificationCheck => {
            QualificationCheck::get_edit_fields(
                &txn,
                &tenancy,
                &request.visibility,
                &request.id.into(),
            )
            .await?
        }
        EditFieldObjectKind::Schema => {
            Schema::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into()).await?
        }
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into())
                .await?
        }
        EditFieldObjectKind::SchemaVariant => {
            schema::SchemaVariant::get_edit_fields(
                &txn,
                &tenancy,
                &request.visibility,
                &request.id.into(),
            )
            .await?
        }
        EditFieldObjectKind::Socket => {
            Socket::get_edit_fields(&txn, &tenancy, &request.visibility, &request.id.into()).await?
        }
    };

    txn.commit().await?;

    Ok(Json(GetEditFieldsResponse {
        fields: edit_fields,
    }))
}
