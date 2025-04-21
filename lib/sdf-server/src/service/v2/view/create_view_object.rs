use axum::{
    Json,
    extract::{Host, OriginalUri, Path},
};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
    track,
};
use dal::{
    ChangeSet, ChangeSetId, WorkspacePk, WsEvent,
    diagram::{
        geometry::Geometry,
        view::{View, ViewId},
    },
};
use si_frontend_types::RawGeometry;

use super::{ViewError, ViewResult};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum CreateComponentSchemaType {
    Installed,
    Uninstalled,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub view_object_id: ViewId,
    pub x: String,
    pub y: String,
    pub radius: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub view_object_id: ViewId,
}

pub async fn create_view_object(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, container_view_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ViewId,
    )>,
    Json(request): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<Response>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let object_view = View::get_by_id(&ctx, request.view_object_id).await?;

    let _geometry: Geometry;
    let (Ok(x), Ok(y), Ok(radius)) = (
        request.x.clone().parse::<isize>(),
        request.y.clone().parse::<isize>(),
        request
            .radius
            .map(|w| w.clone().parse::<isize>())
            .transpose(),
    ) else {
        ctx.rollback().await?;
        return Err(ViewError::InvalidRequest(
            "geometry unable to be parsed from create view object request".into(),
        ));
    };

    let geometry = RawGeometry {
        x,
        y,
        width: radius,
        height: radius,
    };
    View::add_to_another_view(
        &ctx,
        request.view_object_id,
        container_view_id,
        geometry.clone(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_view_object",
        serde_json::json!({
            "how": "/view/create_view_object",
            "object_view_id": object_view.id(),
            "object_view_name": object_view.name(),
            "change_set_id": ctx.change_set_id(),
            "container_view_id": container_view_id,
        }),
    );

    WsEvent::view_object_created(&ctx, container_view_id, request.view_object_id, geometry)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        Response {
            view_object_id: object_view.id(),
        },
    ))
}
