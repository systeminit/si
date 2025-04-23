use std::collections::HashMap;

use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use si_frontend_types::object::FrontendObject;

use super::{
    AccessBuilder,
    IndexResult,
};
use crate::extract::{
    FriggStore,
    HandlerContext,
};

pub async fn get_workspace_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    Path(workspace_pk): Path<WorkspacePk>,
) -> IndexResult<Json<HashMap<ChangeSetId, Option<FrontendObject>>>> {
    let ctx = builder.build_head(access_builder).await?;

    let mut indexes = HashMap::new();
    for change_set in ChangeSet::list_active(&ctx).await? {
        let maybe_index = frigg.get_index(workspace_pk, change_set.id).await?;
        indexes.insert(change_set.id, maybe_index.map(|i| i.0));
    }

    Ok(Json(indexes))
}
