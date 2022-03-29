use axum::extract::Query;
use axum::Json;
use dal::{
    edit_field::{EditFieldAble, EditFieldObjectKind, EditFields},
    schema,
    socket::Socket,
    Component, Prop, QualificationCheck, Schema, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::EditFieldResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetEditFieldsRequest>,
) -> EditFieldResult<Json<GetEditFieldsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let edit_fields = match request.object_kind {
        EditFieldObjectKind::Component => {
            Component::get_edit_fields(&ctx, &request.id.into()).await?
        }
        EditFieldObjectKind::ComponentProp => {
            Component::get_edit_fields(&ctx, &request.id.into()).await?
        }
        EditFieldObjectKind::Prop => Prop::get_edit_fields(&ctx, &request.id.into()).await?,
        EditFieldObjectKind::QualificationCheck => {
            QualificationCheck::get_edit_fields(&ctx, &request.id.into()).await?
        }
        EditFieldObjectKind::Schema => Schema::get_edit_fields(&ctx, &request.id.into()).await?,
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::get_edit_fields(&ctx, &request.id.into()).await?
        }
        EditFieldObjectKind::SchemaVariant => {
            schema::SchemaVariant::get_edit_fields(&ctx, &request.id.into()).await?
        }
        EditFieldObjectKind::Socket => Socket::get_edit_fields(&ctx, &request.id.into()).await?,
    };

    txns.commit().await?;

    Ok(Json(GetEditFieldsResponse {
        fields: edit_fields,
    }))
}
