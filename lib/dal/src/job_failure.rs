use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor_ro, DalContext, PgPoolError,
    StandardModelError, Tenancy, Timestamp, TransactionsError, Visibility,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum JobFailureError {
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type JobFailureResult<T, E = JobFailureError> = Result<T, E>;

pk!(JobFailurePk);
pk!(JobFailureId);

/// Background tasks, enqueued on the dynamic transport layer, executed by `pinga` may fail.
/// And not all failure can be reported to the user by the regular path, so
/// any fatal failure, like a panic or an `Err` propagated from the `dal will
/// be stored in the database in the form of a `JobFailure`
///
/// The failure will be set to the user's tenancy and visibility, with the user
/// as the actor. For now we don't support drastic failures that happen before
/// `pinga` can obtain this metadata, they will only be logged
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct JobFailure {
    pk: JobFailurePk,
    id: JobFailureId,
    kind: String,
    message: String,
    solved: bool,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: JobFailure,
    pk: JobFailurePk,
    id: JobFailureId,
    table_name: "job_failures",
    history_event_label_base: "job_failure",
    history_event_message_name: "JobFailure"
}

impl JobFailure {
    pub async fn new(
        ctx: &DalContext,
        kind: impl AsRef<str>,
        message: impl AsRef<str>,
    ) -> JobFailureResult<Self> {
        let kind = kind.as_ref();
        let message = message.as_ref();

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM job_failure_create_v1($1, $2, $3, $4)",
                &[ctx.tenancy(), ctx.visibility(), &kind, &message],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor_ro!(kind, String);
    standard_model_accessor_ro!(message, String);
}
