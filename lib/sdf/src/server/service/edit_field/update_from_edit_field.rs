use axum::Json;
use dal::{
    edit_field::{EditFieldAble, EditFieldBaggage, EditFieldObjectKind},
    schema::{self, SchemaVariant},
    socket::Socket,
    AttributeContext, Component, Prop, QualificationCheck, Schema, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{EditFieldError, EditFieldResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFromEditFieldRequest {
    pub object_kind: EditFieldObjectKind,
    pub object_id: i64,
    pub edit_field_id: String,
    pub value: Option<serde_json::Value>,
    pub baggage: Option<EditFieldBaggage>,
    pub workspace_id: Option<WorkspaceId>,
    pub attribute_context: Option<AttributeContext>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFromEditFieldResponse {
    pub success: bool,
}

pub async fn update_from_edit_field(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<UpdateFromEditFieldRequest>,
) -> EditFieldResult<Json<UpdateFromEditFieldResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    let async_tasks = match request.object_kind {
        EditFieldObjectKind::Component => {
            Component::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
        EditFieldObjectKind::ComponentProp => {
            let baggage = request.baggage.ok_or(EditFieldError::MissingBaggage)?;
            let attribute_context = request
                .attribute_context
                .ok_or(EditFieldError::MissingAttributeContext)?;
            Component::update_from_edit_field_with_baggage(
                &ctx,
                request.value,
                attribute_context,
                baggage,
            )
            .await?
        }
        EditFieldObjectKind::Prop => {
            Prop::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
        EditFieldObjectKind::QualificationCheck => {
            QualificationCheck::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
        EditFieldObjectKind::Schema => {
            Schema::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
        EditFieldObjectKind::SchemaVariant => {
            SchemaVariant::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
        EditFieldObjectKind::Socket => {
            Socket::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?;
            None
        }
    };

    txns.commit().await?;

    if let Some(async_tasks) = async_tasks {
        tokio::task::spawn(async move {
            if let Err(err) = async_tasks
                .run(request_ctx, request.visibility, &builder)
                .await
            {
                error!("Component async task execution failed: {err}");
            }
        });
    }

    Ok(Json(UpdateFromEditFieldResponse { success: true }))
}
