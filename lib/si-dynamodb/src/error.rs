use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::create_table::CreateTableError;
use aws_sdk_dynamodb::operation::delete_table::DeleteTableError;
use aws_smithy_http::body::SdkBody;
use http::response::Response;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum DynamoError {
    #[error("aws sdk dynamodb error: {0}")]
    Client(#[from] aws_sdk_dynamodb::Error),
    #[error("aws sdk create table error: {0}")]
    CreateTable(#[from] SdkError<CreateTableError, Response<SdkBody>>),
    #[error("aws sdk delete table error: {0}")]
    DeleteTable(#[from] SdkError<DeleteTableError, Response<SdkBody>>),
}

#[allow(missing_docs)]
pub type DynamoResult<T> = Result<T, DynamoError>;
