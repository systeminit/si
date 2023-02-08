use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{
    component::ComponentKind,
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionError as DalSchemaVariantDefinitionError,
        SchemaVariantDefinitionId,
    },
    StandardModel, StandardModelError, TransactionsError, Visibility, WriteTenancyError,
    WsEventError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaVariantDefinitionError {
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("write tenancy error: {0}")]
    WriteTenancy(#[from] WriteTenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] DalSchemaVariantDefinitionError),
    #[error("Schema Variant Definition {0} not found")]
    VariantDefnitionNotFound(SchemaVariantDefinitionId),
}

pub type SchemaVariantDefinitionResult<T> = Result<T, SchemaVariantDefinitionError>;

impl IntoResponse for SchemaVariantDefinitionError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantDefsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListedVariantDef {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantDefsResponse {
    pub variant_defs: Vec<ListedVariantDef>,
}

pub async fn list_variant_defs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListVariantDefsRequest>,
) -> SchemaVariantDefinitionResult<Json<ListVariantDefsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_defs: Vec<ListedVariantDef> = SchemaVariantDefinition::list(&ctx)
        .await?
        .iter()
        .map(|def| ListedVariantDef {
            id: def.id().to_owned(),
            name: def.name().to_owned(),
            menu_name: def.menu_name().map(|menu_name| menu_name.to_owned()),
            category: def.category().to_owned(),
            color: def.color().to_owned(),
        })
        .collect();

    Ok(Json(ListVariantDefsResponse { variant_defs }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefResponse {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub component_kind: ComponentKind,
    pub link: Option<String>,
    pub definition: String,
}

impl From<SchemaVariantDefinition> for GetVariantDefResponse {
    fn from(variant: SchemaVariantDefinition) -> Self {
        GetVariantDefResponse {
            id: *variant.id(),
            name: variant.name().to_string(),
            menu_name: variant.menu_name().map(|menu_name| menu_name.to_string()),
            category: variant.category().to_string(),
            color: variant.color().to_string(),
            component_kind: *variant.component_kind(),
            link: variant.link().map(|link| link.to_string()),
            definition: variant.definition().to_string(),
        }
    }
}

pub async fn get_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<GetVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefnitionNotFound(
            request.id,
        ))?;

    Ok(Json(variant_def.into()))
}

pub fn routes() -> Router {
    Router::new()
        .route("/list_variant_defs", get(list_variant_defs))
        .route("/get_variant_def", get(get_variant_def))
}
