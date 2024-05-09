use std::num::ParseIntError;

use buck2_resources::Buck2ResourcesError;
use si_std::CanonicalFileError;
use thiserror::Error;

use crate::config::ConfigBuilderError;

// SI Full Stack errors.
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SiFullStackError {
    #[error("buck2 resources error: {0}")]
    Buck2Resources(#[from] Buck2ResourcesError),
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("config builder error: {0}")]
    ConfigBuilder(#[from] ConfigBuilderError),
    #[error("error parsing integer: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("full stack server panic: {0}")]
    ServerPanic(String),
    #[error("sodium oxide failed to initialize")]
    SodiumOxideInit,
    #[error("Tokio IO Error: {0}")]
    TokioIo(#[from] tokio::io::Error),
    #[error("Veritech Config Error: {0}")]
    VeritechConfig(#[from] veritech_server::ConfigError),
    #[error("Veritech server error: {0}")]
    VeritechServer(#[from] veritech_server::ServerError),
}

pub(crate) type SiFullStackResult<T> = Result<T, SiFullStackError>;
