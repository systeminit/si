use std::{
    fmt,
    ops::Deref,
};

use axum::{
    Json,
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use hyper::StatusCode;
use s3::{
    Bucket as S3Bucket,
    Region as AwsRegion,
    error::S3Error,
};
use sea_orm::{
    DatabaseTransaction,
    TransactionTrait,
};
use si_jwt_public_key::{
    SiJwtClaimRole,
    SiJwtClaims,
};

use super::app_state::AppState;

pub struct ExtractedS3Bucket {
    pub s3_bucket: S3Bucket,
    pub cloudfront_domain: Option<String>,
}

impl ExtractedS3Bucket {
    pub async fn url_for_module(self, module_hash: String) -> Result<String, S3Error> {
        let object_key = format!("{}.{}", module_hash, "sipkg");
        self.get_url(object_key).await
    }

    pub async fn url_for_export(self, module_hash: String) -> Result<String, S3Error> {
        let object_key = format!("{}.{}", module_hash, "workspace_export");
        self.get_url(object_key).await
    }

    async fn get_url(self, object_key: String) -> Result<String, S3Error> {
        let download_url = if let Some(domain) = self.cloudfront_domain {
            format!("https://{domain}/{object_key}")
        } else {
            self.s3_bucket.presign_get(object_key, 60 * 5, None).await?
        };

        Ok(download_url)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for ExtractedS3Bucket {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let region = match state.s3_config().region.parse::<AwsRegion>() {
            Ok(region) => region,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": {
                            "message": err.to_string(),
                            "code": 42,
                            "statusCode": 500
                        }
                    })),
                ));
            }
        };

        let s3_bucket = S3Bucket::new(&state.s3_config().bucket, region, state.aws_creds().clone())
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": {
                            "message": err.to_string(),
                            "code": 42,
                            "statusCode": 500
                        }
                    })),
                )
            })?;

        let cloudfront_domain = state.s3_config().cloudfront_domain.clone();
        Ok(ExtractedS3Bucket {
            s3_bucket,
            cloudfront_domain,
        })
    }
}

pub struct DbConnection(pub DatabaseTransaction);

impl Deref for DbConnection {
    type Target = DatabaseTransaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl FromRequestParts<AppState> for DbConnection {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match state.pg_pool().begin().await {
            Ok(txn) => Ok(DbConnection(txn)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": err.to_string(),
                        "code": 42,
                        "statusCode": 500
                    }
                })),
            )),
        }
    }
}

pub struct Authorization {
    pub user_claim: SiJwtClaims,
    pub auth_token: String,
}

#[async_trait]
impl FromRequestParts<AppState> for Authorization {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jwt_public_signing_key = state.jwt_public_signing_key();

        let headers = &parts.headers;
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(|| unauthorized_error("No Authorization header"))?;
        let auth_token = authorization_header_value
            .to_str()
            .map_err(internal_error)?;
        let user_claim =
            si_jwt_public_key::validate_bearer_token(jwt_public_signing_key.clone(), auth_token)
                .await
                .map_err(unauthorized_error)?
                .custom;
        if !user_claim.authorized_for(SiJwtClaimRole::Web) {
            return Err(unauthorized_error("Not authorized for web role"));
        }

        Ok(Self {
            user_claim,
            auth_token: auth_token.into(),
        })
    }
}

fn internal_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

fn unauthorized_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::UNAUTHORIZED;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}
