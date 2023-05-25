use axum::{
    extract::{Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetModuleDetailsRequest {
    pub foo: Option<bool>,
}

pub async fn get_module_details_route(
    Path(module_id): Path<Ulid>,
    Query(request): Query<GetModuleDetailsRequest>,
) -> Json<Value> {
    Json(json!({ "id": module_id, "foo": request.foo }))
}
