use axum::{
    extract::{multipart::MultipartError, Multipart},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, FixedOffset, Offset, Utc};
use hyper::StatusCode;
use module_index_types::{ExtraMetadata, FuncMetadata, ModuleDetailsResponse};
use module_index_types::{
    MODULE_BASED_ON_HASH_FIELD_NAME, MODULE_BUNDLE_FIELD_NAME, MODULE_SCHEMA_ID_FIELD_NAME,
};
use s3::error::S3Error;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, Set};
use serde::{Deserialize, Serialize};
use si_pkg::{SiPkg, SiPkgError, SiPkgKind};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    extract::{Authorization, DbConnection, ExtractedS3Bucket},
    models::si_module::{self, make_module_details_response, ModuleId, ModuleKind, SchemaId},
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpsertModuleRequest {
    pub foo: Option<bool>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum UpsertModuleError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("file upload error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("multipart decode error: {0}")]
    Multipart(#[from] MultipartError),
    #[error("module with {0} could not be found after insert!")]
    NotFoundAfterInsert(ModuleId),
    #[error("s3 error: {0}")]
    S3Error(#[from] S3Error),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("module parsing error: {0}")]
    SiPkgError(#[from] SiPkgError),
    #[error("Ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("upload is required")]
    UploadRequiredError,
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for UpsertModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        error!("upsert error: {}", &error_message);

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

// #[debug_handler]
pub async fn upsert_module_route(
    Authorization { user_claim, .. }: Authorization,
    ExtractedS3Bucket(s3_bucket): ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
    mut multipart: Multipart,
) -> Result<Json<ModuleDetailsResponse>, UpsertModuleError> {
    let mut module_data = None;
    let mut module_based_on_hash = None;
    let mut module_schema_id = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some(MODULE_BUNDLE_FIELD_NAME) => {
                module_data = Some(field.bytes().await?);
            }
            Some(MODULE_BASED_ON_HASH_FIELD_NAME) => {
                module_based_on_hash = Some(field.text().await?);
            }
            Some(MODULE_SCHEMA_ID_FIELD_NAME) => {
                module_schema_id = Some(field.text().await?);
            }
            _ => debug!("Unknown multipart form field on module upload, skipping..."),
        }
    }

    let data = module_data.ok_or(UpsertModuleError::UploadRequiredError)?;

    // SiPkg using old term "package" but we are dealing with a "module"
    let loaded_module = SiPkg::load_from_bytes(data.to_vec())?;
    let module_metadata = loaded_module.metadata()?;

    info!(
        "upserting module: {:?} based on hash: {:?} with provided schema id of {:?}",
        &module_metadata, &module_based_on_hash, &module_schema_id
    );

    let version = module_metadata.version().to_owned();
    let module_kind = match module_metadata.kind() {
        SiPkgKind::WorkspaceBackup => ModuleKind::WorkspaceBackup,
        SiPkgKind::Module => ModuleKind::Module,
    };

    let new_schema_id = Some(SchemaId(Ulid::new()));
    let schema_id = match module_kind {
        ModuleKind::WorkspaceBackup => None,
        ModuleKind::Module => match module_schema_id {
            Some(schema_id_string) => Some(SchemaId(Ulid::from_string(&schema_id_string)?)),
            None => match module_based_on_hash {
                None => new_schema_id,
                Some(based_on_hash) => {
                    match si_module::Entity::find()
                        .filter(si_module::Column::Kind.eq(ModuleKind::Module))
                        .filter(si_module::Column::LatestHash.eq(based_on_hash))
                        .limit(1)
                        .all(&txn)
                        .await?
                        .first()
                    {
                        None => new_schema_id,
                        Some(module) => match module.schema_id {
                            some @ Some(_) => some,
                            None => {
                                // If we found matching past hash but it has no schema id, backfill it to match the one we're generating
                                let mut active: si_module::ActiveModel = module.to_owned().into();
                                active.schema_id = Set(new_schema_id);
                                active.update(&txn).await?;

                                new_schema_id
                            }
                        },
                    }
                }
            },
        },
    };

    if let Some(schema_id) = schema_id {
        info!("module gets schema id: {}", schema_id.0);
    }

    let schemas: Vec<String> = loaded_module
        .schemas()?
        .iter()
        .map(|s| s.name().to_owned())
        .collect();
    let funcs: Vec<FuncMetadata> = loaded_module
        .funcs()?
        .iter()
        .map(|f| FuncMetadata {
            name: f.name().to_owned(),
            display_name: f.display_name().map(|d| d.to_owned()),
            description: f.description().map(|d| d.to_owned()),
        })
        .collect();

    let new_module = si_module::ActiveModel {
        name: Set(module_metadata.name().to_owned()),
        description: Set(Some(module_metadata.description().to_owned())),
        owner_user_id: Set(user_claim.user_pk.to_string()),
        owner_display_name: Set(Some(module_metadata.created_by().to_owned())),
        latest_hash: Set(module_metadata.hash().to_string()),
        // maybe use db's `CLOCK_TIMESTAMP()`?
        latest_hash_created_at: Set(DateTime::<FixedOffset>::from_naive_utc_and_offset(
            Utc::now().naive_utc(),
            Utc.fix(),
        )),
        metadata: Set(serde_json::to_value(ExtraMetadata {
            version,
            schemas,
            funcs,
        })?),
        kind: Set(module_kind),
        schema_id: Set(schema_id),
        ..Default::default() // all other attributes are `NotSet`
    };

    // TODO: put below
    // upload to s3
    s3_bucket
        .put_object(format!("{}.sipkg", module_metadata.hash()), &data)
        .await?;

    let new_module: si_module::Model = new_module.insert(&txn).await?;
    let (module, linked_modules) = si_module::Entity::find_by_id(new_module.id)
        .find_with_linked(si_module::SchemaIdReferenceLink)
        .all(&txn)
        .await?
        .first()
        .cloned()
        .ok_or(UpsertModuleError::NotFoundAfterInsert(new_module.id))?;

    txn.commit().await?;

    Ok(Json(make_module_details_response(module, linked_modules)))
}
