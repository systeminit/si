use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::{Generator, Ulid};

#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("Mutex error: {0}")]
    Mutex(String),
    #[error("Ulid Monotonic Error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

// FIXME(nick): remove this in favor of the real one.
pub type ChangeSetId = Ulid;

// FIXME(nick): remove this in favor of the real one.
#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: ChangeSetId,
    #[serde(skip)]
    pub generator: Arc<Mutex<Generator>>,
}

impl ChangeSet {
    pub fn new() -> ChangeSetResult<Self> {
        let mut generator = Generator::new();
        let id = generator.generate()?;

        Ok(Self {
            id,
            generator: Arc::new(Mutex::new(generator)),
        })
    }

    pub fn generate_ulid(&self) -> ChangeSetResult<Ulid> {
        self.generator
            .lock()
            .map_err(|e| ChangeSetError::Mutex(e.to_string()))?
            .generate()
            .map_err(Into::into)
    }
}

impl std::fmt::Debug for ChangeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeSet")
            .field("id", &self.id.to_string())
            .finish()
    }
}
