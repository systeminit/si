use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesRequest {
    pub foo: Option<bool>,
}

pub async fn list_module_route(Query(request): Query<ListModulesRequest>) -> Json<Value> {
    Json(json!({ "ok": true }))
}
