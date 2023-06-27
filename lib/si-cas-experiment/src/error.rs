use thiserror::Error;

#[derive(Error, Debug)]
pub enum CasError {
    #[error("merged two vector clocks with different ids")]
    WrongMergeId,
    #[error("cannot increment a vector clock without a who")]
    NoWho,
    #[error("cannot find function with content hash")]
    NoContentHash,
}

pub type CasResult<T> = Result<T, CasError>;
