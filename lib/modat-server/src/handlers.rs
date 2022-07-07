use axum::Json;
use serde::{Deserialize, Serialize};

use crate::AppResult;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreetRequest {
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GreetResponse {
    greeting: String,
}

pub async fn greet(Json(request): Json<GreetRequest>) -> AppResult<Json<GreetResponse>> {
    let response = format!("Well hello there, {}", request.name);

    Ok(Json(GreetResponse { greeting: response }))
}
