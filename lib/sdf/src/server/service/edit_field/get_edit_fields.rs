use axum::extract::Query;
use axum::Json;
use dal::edit_field::EditField;
use dal::{
    edit_field::{EditFieldAble, EditFieldObjectKind},
    schema,
    socket::Socket,
    Component, Prop, QualificationCheck, Schema, StandardModel, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{EditFieldError, EditFieldResult};
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
    pub fields: Vec<EditField>,
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
            let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.id.into()).await?;
            let is_component_in_visibility = Component::get_by_id(&ctx, &request.id.into())
                .await?
                .is_some();
            if is_component_in_tenancy && !is_component_in_visibility {
                return Err(EditFieldError::InvalidVisibility);
            }
            Component::get_edit_fields(&ctx, &request.id.into()).await?
        }
        EditFieldObjectKind::ComponentProp => {
            let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.id.into()).await?;
            let is_component_in_visibility = Component::get_by_id(&ctx, &request.id.into())
                .await?
                .is_some();
            if is_component_in_tenancy && !is_component_in_visibility {
                return Err(EditFieldError::InvalidVisibility);
            }
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
