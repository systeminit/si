use thiserror::Error;

use crate::{workspace::WorkspacePk, schema::SchemaPk, change_set::ChangeSetPk, dag::Conflict};

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
    #[error("cannot find an object by a pk")]
    CannotFindObjectByPk,
    #[error("object mismatch")]
    ObjectMismatch,
    #[error("vector clock not found")]
    VectorClockNotFound,
    #[error("mistmatched update object types")]
    MismatchedUpdateObject,
    #[error("must fix conflicts before merging: {0:?}")]
    MergeHasConflicts(Vec<Conflict>),
}

pub type DagResult<T> = Result<T, DagError>;
