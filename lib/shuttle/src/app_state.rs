use si_data_nats::{jetstream::Context, Subject};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) context: Context,
    pub(crate) destination_subject: Subject,
    pub(crate) token: CancellationToken,
}

impl AppState {
    pub(crate) fn new(
        context: Context,
        destination_subject: Subject,
        token: CancellationToken,
    ) -> Self {
        Self {
            context,
            destination_subject,
            token,
        }
    }
}
