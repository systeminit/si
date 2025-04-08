use std::str::FromStr;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::Path, Json};
use chrono::{DateTime, FixedOffset, Offset, Utc};
use module_index_types::ModuleDetailsResponse;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use si_id::SchemaId;
use telemetry::prelude::info;
use thiserror::Error;

use crate::app_state::AppState;
use crate::models::si_module::{make_module_details_response, SchemaIdReferenceLink};
use crate::routes::upsert_module_route::UpsertModuleError;
use crate::whoami::{is_systeminit_auth_token, WhoamiError};
use crate::{
    extract::{Authorization, DbConnection},
    models::si_module::{self, ModuleId},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum RejectModuleError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error(r#"Module "{0}" not found"#)]
    NotFound(ModuleId),
    #[error("error rejecting module: {0}")]
    RejectModule(#[from] UpsertModuleError),
    #[error("Ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("missing username for module rejection")]
    UserSupplied(),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

impl IntoResponse for RejectModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub type RejectModuleResult<T> = std::result::Result<T, RejectModuleError>;

pub async fn reject_module(
    Path(module_id): Path<ModuleId>,
    Authorization {
        user_claim: _user_claim,
        auth_token,
    }: Authorization,
    DbConnection(txn): DbConnection,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> RejectModuleResult<Json<Option<ModuleDetailsResponse>>> {
    if !is_systeminit_auth_token(state.auth_api_url(), &auth_token, state.token_emails()).await? {
        return Ok(Json(None));
    }

    info!("Reject module");
    let field = match multipart.next_field().await.unwrap() {
        Some(f) => f,
        None => return Err(RejectModuleError::UserSupplied()),
    };
    info!("Found multipart field");
    let data = dbg!(field.text().await.unwrap());
    info!("Got part data");

    let (module, linked_modules) = si_module::Entity::find_by_id(module_id)
        .find_with_linked(si_module::SchemaIdReferenceLink)
        .all(&txn)
        .await?
        .first()
        .cloned()
        .ok_or(RejectModuleError::NotFound(module_id))?;
    let active_module = si_module::ActiveModel {
        id: Set(module.id),
        name: Set(module.name),
        description: Set(module.description),
        owner_user_id: Set(module.owner_user_id),
        owner_display_name: Set(module.owner_display_name),
        metadata: Set(module.metadata),
        latest_hash: Set(module.latest_hash),
        latest_hash_created_at: Set(module.latest_hash_created_at),
        created_at: Set(module.created_at),
        rejected_at: Set(Some(DateTime::<FixedOffset>::from_naive_utc_and_offset(
            Utc::now().naive_utc(),
            Utc.fix(),
        ))),
        rejected_by_display_name: Set(Some(data)),
        kind: Set(module.kind),
        is_builtin_at: Set(module.is_builtin_at),
        is_builtin_at_by_display_name: Set(module.is_builtin_at_by_display_name),
        schema_id: Set(module.schema_id),
        ..Default::default() // all other attributes are `NotSet`
    };

    let updated_module: si_module::Model = dbg!(active_module.update(&txn).await)?;

    txn.commit().await?;

    let response = make_module_details_response(updated_module, linked_modules);

    Ok(Json(Some(response)))
}

pub async fn reject_other_modules_of_a_schema_id(
    data: String,
    schema_id: SchemaId,
    txn: &sea_orm::DatabaseTransaction,
) -> RejectModuleResult<()> {
    let query = si_module::Entity::find();

    // filters
    let query = query
        .find_with_linked(SchemaIdReferenceLink)
        .filter(si_module::Column::SchemaId.eq(schema_id));

    let modules: Vec<ModuleDetailsResponse> = query
        .all(txn)
        .await?
        .into_iter()
        .map(|(module, linked_modules)| make_module_details_response(module, linked_modules))
        .collect();

    for module in modules {
        let module_id = ModuleId::from_str(&module.id)?;
        let (module, _) = si_module::Entity::find_by_id(module_id)
            .find_with_linked(si_module::SchemaIdReferenceLink)
            .all(txn)
            .await?
            .first()
            .cloned()
            .ok_or(RejectModuleError::NotFound(module_id))?;

        let active_module = si_module::ActiveModel {
            id: Set(module.id),
            name: Set(module.name),
            description: Set(module.description),
            owner_user_id: Set(module.owner_user_id),
            owner_display_name: Set(module.owner_display_name),
            metadata: Set(module.metadata),
            latest_hash: Set(module.latest_hash),
            latest_hash_created_at: Set(module.latest_hash_created_at),
            created_at: Set(module.created_at),
            rejected_at: Set(Some(DateTime::<FixedOffset>::from_naive_utc_and_offset(
                Utc::now().naive_utc(),
                Utc.fix(),
            ))),
            rejected_by_display_name: Set(Some(data.clone())),
            kind: Set(module.kind),
            is_builtin_at: Set(module.is_builtin_at),
            is_builtin_at_by_display_name: Set(module.is_builtin_at_by_display_name),
            schema_id: Set(module.schema_id),
            ..Default::default() // all other attributes are `NotSet`
        };
        active_module.update(txn).await?;
    }

    Ok(())
}
