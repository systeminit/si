use thiserror::Error;

#[derive(Error, Debug)]
pub enum CasError {
    #[error("poop")]
    Poop,
}

pub type CasResult<T> = Result<T, CasError>;
