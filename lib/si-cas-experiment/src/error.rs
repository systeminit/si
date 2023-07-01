use thiserror::Error;

use crate::{workspace::WorkspacePk, schema::SchemaPk, change_set::ChangeSetPk};

#[derive(Error, Debug, PartialEq)]
pub enum DagError {
    #[error("the workspace was not found {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("the schema was not found {0}")]
    SchemaNotFound(SchemaPk),
    #[error("the change set was not found {0}")]
    ChangeSetNotFound(ChangeSetPk),
    #[error("the change set name was not found {0}")]
    ChangeSetNameNotFound(String),
    #[error("the change set must be rebased before merging")]
    MustRebase,
    #[error("you tried to merge a vector clock and they were for different objects")]
    CannotMergeVectorClocksForDifferentObjects,
    #[error("missing node weight")]
    MissingNodeWeight,
}

pub type DagResult<T> = Result<T, DagError>;
