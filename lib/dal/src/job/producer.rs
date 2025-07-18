use thiserror::Error;

pub type BlockingJobResult = Result<(), BlockingJobError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum BlockingJobError {
    #[error("error during job execution: {0}")]
    JobExecution(String),
    #[error("job queue processor error: {0}")]
    JobQueueProcessor(String),
    #[error("stream create error: {0}")]
    JsCreateStreamError(String),
    #[error("missing required workspace_pk")]
    MissingWorkspacePk,
    #[error("A nats error occurred: {0}")]
    Nats(String),
    #[error("no access builder found in job info")]
    NoAccessBuilder,
    #[error("pinga client error: {0}")]
    PingaClient(#[from] Box<pinga_client::ClientError>),
    #[error("serde error: {0}")]
    Serde(String),
    #[error("A transactions error occurred: {0}")]
    Transactions(String),
}

impl From<pinga_client::ClientError> for BlockingJobError {
    fn from(value: pinga_client::ClientError) -> Self {
        Box::new(value).into()
    }
}
